//! gRPC client implementation for testing echo service

use grpc_performance_rs::{echo::{echo_service_client::EchoServiceClient, EchoRequest}, crypto::{
    crypto_service_client::CryptoServiceClient,
    SignRequest, PublicKeyRequest, KeyType, SigningAlgorithm
}, current_timestamp_millis, AppResult, DEFAULT_SERVER_ADDR, DEFAULT_LOG_LEVEL, transport::{TransportConfig}, create_transport_channel};
use log::{info, error, debug};
use std::env;
use std::str::FromStr;
use tonic::transport::{Channel};

/// Create a sample echo request
fn create_echo_request(payload: &str) -> EchoRequest {
    EchoRequest {
        payload: payload.to_string(),
        timestamp: current_timestamp_millis(),
    }
}


/// Connect to the gRPC server for echo service
async fn connect_to_echo_server(transport_config: &TransportConfig) -> AppResult<EchoServiceClient<Channel>> {
    info!("Connecting to gRPC echo service at {}", transport_config);

    let channel = create_transport_channel(transport_config).await?;
    Ok(EchoServiceClient::new(channel))
}

/// Connect to the gRPC server for crypto service
async fn connect_to_crypto_server(transport_config: &TransportConfig) -> AppResult<CryptoServiceClient<Channel>> {
    info!("Connecting to gRPC crypto service at {}", transport_config);

    let channel = create_transport_channel(transport_config).await?;
    Ok(CryptoServiceClient::new(channel))
}

/// Send echo requests and measure performance
async fn run_echo_test(client: &mut EchoServiceClient<Channel>) -> AppResult<()> {
    let test_payloads = vec![
        "Hello, gRPC!",
        "This is a test message",
        "Performance testing with minimal latency",
        "🚀 High-performance echo service",
    ];

    for (i, payload) in test_payloads.iter().enumerate() {
        let request = create_echo_request(payload);
        let request_time = request.timestamp;

        debug!("Sending echo request {}: '{}'", i + 1, payload);

        match client.echo(request).await {
            Ok(response) => {
                let resp = response.into_inner();
                let total_latency = current_timestamp_millis() - request_time;
                let server_latency = resp.response_timestamp - resp.request_timestamp;

                info!(
                    "Echo response {}: payload='{}', server_latency={}ms, total_latency={}ms",
                    i + 1, resp.payload, server_latency, total_latency
                );

                // Verify the echo worked correctly
                if resp.payload != *payload {
                    error!("Echo mismatch! Expected: '{}', Got: '{}'", payload, resp.payload);
                }
            }
            Err(e) => {
                error!("Echo request {} failed: {}", i + 1, e);
                return Err(e.into());
            }
        }
    }

    Ok(())
}

/// Test crypto service operations
async fn run_crypto_test(client: &mut CryptoServiceClient<Channel>) -> AppResult<()> {
    info!("Starting crypto service tests");

    let test_data = b"Hello, crypto world!";

    // Test RSA PKCS#1 signing
    let rsa_pkcs1_request = SignRequest {
        data: test_data.to_vec(),
        key_type: KeyType::Rsa as i32,
        algorithm: SigningAlgorithm::RsaPkcs1Sha256 as i32,
        timestamp: current_timestamp_millis(),
    };

    debug!("Testing RSA PKCS#1 signing");
    match client.sign(rsa_pkcs1_request).await {
        Ok(response) => {
            let resp = response.into_inner();
            let total_latency = current_timestamp_millis() - resp.request_timestamp;
            let server_latency = resp.response_timestamp - resp.request_timestamp;

            info!(
                "RSA PKCS#1 sign response: signature_len={}, server_latency={}ms, total_latency={}ms",
                resp.signature.len(), server_latency, total_latency
            );
        }
        Err(e) => {
            error!("RSA PKCS#1 signing failed: {}", e);
            return Err(e.into());
        }
    }

    // Test ECC P-256 signing
    let ecc_request = SignRequest {
        data: test_data.to_vec(),
        key_type: KeyType::Ecc as i32,
        algorithm: SigningAlgorithm::EcdsaP256Sha256 as i32,
        timestamp: current_timestamp_millis(),
    };

    debug!("Testing ECC P-256 signing");
    match client.sign(ecc_request).await {
        Ok(response) => {
            let resp = response.into_inner();
            let total_latency = current_timestamp_millis() - resp.request_timestamp;
            let server_latency = resp.response_timestamp - resp.request_timestamp;

            info!(
                "ECC P-256 sign response: signature_len={}, server_latency={}ms, total_latency={}ms",
                resp.signature.len(), server_latency, total_latency
            );
        }
        Err(e) => {
            error!("ECC P-256 signing failed: {}", e);
            return Err(e.into());
        }
    }

    // Test public key retrieval
    let pubkey_request = PublicKeyRequest {
        key_type: KeyType::Rsa as i32,
        timestamp: current_timestamp_millis(),
    };

    debug!("Testing RSA public key retrieval");
    match client.get_public_key(pubkey_request).await {
        Ok(response) => {
            let resp = response.into_inner();
            let total_latency = current_timestamp_millis() - resp.request_timestamp;
            let server_latency = resp.response_timestamp - resp.request_timestamp;

            info!(
                "RSA public key response: key_len={}, server_latency={}ms, total_latency={}ms",
                resp.public_key_der.len(), server_latency, total_latency
            );
        }
        Err(e) => {
            error!("RSA public key retrieval failed: {}", e);
            return Err(e.into());
        }
    }

    Ok(())
}

#[tokio::main]
async fn main() -> AppResult<()> {
    // Initialize logging
    let log_level = env::var("RUST_LOG").unwrap_or_else(|_| DEFAULT_LOG_LEVEL.to_string());
    env::set_var("RUST_LOG", log_level);
    env_logger::init();

    // Parse server address from environment or use default
    let addr_str = env::var("SERVER_ADDR")
        .unwrap_or_else(|_| DEFAULT_SERVER_ADDR.to_string());

    let transport_config = TransportConfig::from_str(&addr_str)
        .map_err(|e| {
            error!("Invalid server address '{}': {}", addr_str, e);
            std::io::Error::new(std::io::ErrorKind::InvalidInput, e)
        })?;

    info!("Starting gRPC client, connecting to {} ({})",
          transport_config,
          if transport_config.is_tcp() { "TCP" } else { "VSOCK" });

    // Connect to echo service
    let mut echo_client = connect_to_echo_server(&transport_config).await?;

    // Run echo tests
    match run_echo_test(&mut echo_client).await {
        Ok(_) => {
            info!("All echo tests completed successfully");
        }
        Err(e) => {
            error!("Echo test failed: {}", e);
            return Err(e);
        }
    }

    // Connect to crypto service
    let mut crypto_client = connect_to_crypto_server(&transport_config).await?;

    // Run crypto tests
    match run_crypto_test(&mut crypto_client).await {
        Ok(_) => {
            info!("All crypto tests completed successfully");
            Ok(())
        }
        Err(e) => {
            error!("Crypto test failed: {}", e);
            Err(e)
        }
    }
}