//! Integration tests for the gRPC echo service

use grpc_performance_rs::{
    echo::{echo_service_client::EchoServiceClient, echo_service_server::{EchoService, EchoServiceServer}, EchoRequest},
    crypto::{
        crypto_service_client::CryptoServiceClient,
        crypto_service_server::{CryptoService, CryptoServiceServer},
        SignRequest, PublicKeyRequest, KeyType, SigningAlgorithm
    },
    current_timestamp_millis, CryptoKeys, AppResult,
};
use std::time::Duration;
use tokio::time::sleep;
use tonic::transport::{Channel, Server};

/// Echo service implementation for testing
#[derive(Debug, Default)]
pub struct TestEchoService;

/// Crypto service implementation for testing
#[derive(Debug)]
pub struct TestCryptoService {
    crypto_keys: CryptoKeys,
}

impl TestCryptoService {
    pub fn new() -> AppResult<Self> {
        let crypto_keys = CryptoKeys::generate()?;
        Ok(TestCryptoService { crypto_keys })
    }
}

#[tonic::async_trait]
impl EchoService for TestEchoService {
    async fn echo(
        &self,
        request: tonic::Request<EchoRequest>,
    ) -> Result<tonic::Response<grpc_performance_rs::echo::EchoResponse>, tonic::Status> {
        let req = request.into_inner();
        let response_timestamp = current_timestamp_millis();
        
        let response = grpc_performance_rs::echo::EchoResponse {
            payload: req.payload,
            request_timestamp: req.timestamp,
            response_timestamp,
        };

        Ok(tonic::Response::new(response))
    }
}

#[tonic::async_trait]
impl CryptoService for TestCryptoService {
    async fn sign(
        &self,
        request: tonic::Request<SignRequest>,
    ) -> Result<tonic::Response<grpc_performance_rs::crypto::SignResponse>, tonic::Status> {
        let req = request.into_inner();
        let response_timestamp = current_timestamp_millis();
        
        // Perform signing based on key type and algorithm
        let signature = match (KeyType::from_i32(req.key_type), SigningAlgorithm::from_i32(req.algorithm)) {
            (Some(KeyType::Rsa), Some(SigningAlgorithm::RsaPkcs1Sha256)) => {
                self.crypto_keys.sign_rsa_pkcs1_sha256(&req.data)
                    .map_err(|e| tonic::Status::internal(format!("RSA PKCS#1 signing failed: {}", e)))?
            }
            (Some(KeyType::Rsa), Some(SigningAlgorithm::RsaPssSha256)) => {
                self.crypto_keys.sign_rsa_pss_sha256(&req.data)
                    .map_err(|e| tonic::Status::internal(format!("RSA PSS signing failed: {}", e)))?
            }
            (Some(KeyType::Ecc), Some(SigningAlgorithm::EcdsaP256Sha256)) => {
                self.crypto_keys.sign_ecdsa_p256_sha256(&req.data)
                    .map_err(|e| tonic::Status::internal(format!("ECDSA P-256 signing failed: {}", e)))?
            }
            (Some(KeyType::Ecc), Some(SigningAlgorithm::EcdsaP384Sha256)) => {
                self.crypto_keys.sign_ecdsa_p384_sha384(&req.data)
                    .map_err(|e| tonic::Status::internal(format!("ECDSA P-384 signing failed: {}", e)))?
            }
            _ => {
                return Err(tonic::Status::invalid_argument(
                    format!("Unsupported key type {:?} or algorithm {:?}", req.key_type, req.algorithm)
                ));
            }
        };

        let response = grpc_performance_rs::crypto::SignResponse {
            signature,
            key_type: req.key_type,
            algorithm: req.algorithm,
            request_timestamp: req.timestamp,
            response_timestamp,
        };

        Ok(tonic::Response::new(response))
    }

    async fn get_public_key(
        &self,
        request: tonic::Request<PublicKeyRequest>,
    ) -> Result<tonic::Response<grpc_performance_rs::crypto::PublicKeyResponse>, tonic::Status> {
        let req = request.into_inner();
        let response_timestamp = current_timestamp_millis();
        
        // Get public key based on key type
        let public_key_der = match KeyType::from_i32(req.key_type) {
            Some(KeyType::Rsa) => {
                self.crypto_keys.get_rsa_public_key_der()
                    .map_err(|e| tonic::Status::internal(format!("RSA public key retrieval failed: {}", e)))?
            }
            Some(KeyType::Ecc) => {
                self.crypto_keys.get_ecc_p256_public_key_der()
                    .map_err(|e| tonic::Status::internal(format!("ECC public key retrieval failed: {}", e)))?
            }
            _ => {
                return Err(tonic::Status::invalid_argument(
                    format!("Unsupported key type: {:?}", req.key_type)
                ));
            }
        };

        let response = grpc_performance_rs::crypto::PublicKeyResponse {
            public_key_der,
            key_type: req.key_type,
            request_timestamp: req.timestamp,
            response_timestamp,
        };

        Ok(tonic::Response::new(response))
    }
}

/// Start a test server in the background with both echo and crypto services
async fn start_test_server(port: u16) -> Result<(), Box<dyn std::error::Error>> {
    let addr = format!("127.0.0.1:{}", port).parse()?;
    let echo_service = TestEchoService::default();
    let crypto_service = TestCryptoService::new().map_err(|e| format!("Failed to create crypto service: {}", e))?;

    tokio::spawn(async move {
        Server::builder()
            .add_service(EchoServiceServer::new(echo_service))
            .add_service(CryptoServiceServer::new(crypto_service))
            .serve(addr)
            .await
            .expect("Test server failed to start");
    });

    // Give the server time to start
    sleep(Duration::from_millis(100)).await;
    Ok(())
}

/// Connect to the test server for echo service
async fn connect_test_echo_client(port: u16) -> Result<EchoServiceClient<Channel>, Box<dyn std::error::Error>> {
    let addr = format!("127.0.0.1:{}", port);
    let channel = Channel::from_shared(format!("http://{}", addr))?
        .connect()
        .await?;
    
    Ok(EchoServiceClient::new(channel))
}

/// Connect to the test server for crypto service
async fn connect_test_crypto_client(port: u16) -> Result<CryptoServiceClient<Channel>, Box<dyn std::error::Error>> {
    let addr = format!("127.0.0.1:{}", port);
    let channel = Channel::from_shared(format!("http://{}", addr))?
        .connect()
        .await?;
    
    Ok(CryptoServiceClient::new(channel))
}

#[tokio::test]
async fn test_echo_service_end_to_end() -> Result<(), Box<dyn std::error::Error>> {
    // Start the test server on port 50055
    start_test_server(50055).await?;
    
    // Connect client
    let mut client = connect_test_echo_client(50055).await?;
    
    // Test basic echo functionality
    let test_payload = "Hello, test!";
    let request = EchoRequest {
        payload: test_payload.to_string(),
        timestamp: current_timestamp_millis(),
    };
    let request_time = request.timestamp;
    
    // Send echo request
    let response = client.echo(request).await?;
    let resp = response.into_inner();
    
    // Validate response
    assert_eq!(resp.payload, test_payload, "Payload should be echoed back");
    assert_eq!(resp.request_timestamp, request_time, "Request timestamp should match");
    assert!(resp.response_timestamp >= request_time, "Response timestamp should be >= request timestamp");
    
    // Test with empty payload
    let empty_request = EchoRequest {
        payload: String::new(),
        timestamp: current_timestamp_millis(),
    };
    
    let empty_response = client.echo(empty_request).await?;
    let empty_resp = empty_response.into_inner();
    assert_eq!(empty_resp.payload, "", "Empty payload should be echoed back");
    
    // Test with large payload
    let large_payload = "x".repeat(1024); // 1KB payload
    let large_request = EchoRequest {
        payload: large_payload.clone(),
        timestamp: current_timestamp_millis(),
    };
    
    let large_response = client.echo(large_request).await?;
    let large_resp = large_response.into_inner();
    assert_eq!(large_resp.payload, large_payload, "Large payload should be echoed back");
    
    println!("✅ All echo service tests passed!");
    Ok(())
}

#[tokio::test]
async fn test_echo_service_performance() -> Result<(), Box<dyn std::error::Error>> {
    // Start the test server on port 50056
    start_test_server(50056).await?;
    
    // Connect client
    let mut client = connect_test_echo_client(50056).await?;
    
    // Performance test with multiple requests
    let num_requests = 10;
    let mut total_latency = 0i64;
    
    for i in 0..num_requests {
        let request = EchoRequest {
            payload: format!("Performance test request {}", i),
            timestamp: current_timestamp_millis(),
        };
        let request_time = request.timestamp;
        
        let response = client.echo(request).await?;
        let resp = response.into_inner();
        
        let latency = resp.response_timestamp - request_time;
        total_latency += latency;
        
        // Ensure reasonable latency (less than 100ms for local testing)
        assert!(latency < 100, "Latency should be less than 100ms, got {}ms", latency);
    }
    
    let avg_latency = total_latency / num_requests;
    println!("📊 Average latency over {} requests: {}ms", num_requests, avg_latency);
    
    // Average latency should be reasonable for local testing
    assert!(avg_latency < 50, "Average latency should be less than 50ms, got {}ms", avg_latency);
    
    println!("✅ Performance test passed!");
    Ok(())
}

#[tokio::test]
async fn test_crypto_service_end_to_end() -> Result<(), Box<dyn std::error::Error>> {
    // Start the test server on port 50057
    start_test_server(50057).await?;
    
    // Connect client
    let mut client = connect_test_crypto_client(50057).await?;
    
    let test_data = b"Hello, crypto test!";
    
    // Test RSA PKCS#1 signing
    let rsa_request = SignRequest {
        data: test_data.to_vec(),
        key_type: KeyType::Rsa as i32,
        algorithm: SigningAlgorithm::RsaPkcs1Sha256 as i32,
        timestamp: current_timestamp_millis(),
    };
    let request_time = rsa_request.timestamp;
    
    let rsa_response = client.sign(rsa_request).await?;
    let rsa_resp = rsa_response.into_inner();
    
    // Validate RSA response
    assert_eq!(rsa_resp.key_type, KeyType::Rsa as i32, "Key type should be RSA");
    assert_eq!(rsa_resp.algorithm, SigningAlgorithm::RsaPkcs1Sha256 as i32, "Algorithm should be RSA PKCS#1");
    assert_eq!(rsa_resp.signature.len(), 256, "RSA signature should be 256 bytes");
    assert_eq!(rsa_resp.request_timestamp, request_time, "Request timestamp should match");
    assert!(rsa_resp.response_timestamp >= request_time, "Response timestamp should be >= request timestamp");
    
    // Test ECC P-256 signing
    let ecc_request = SignRequest {
        data: test_data.to_vec(),
        key_type: KeyType::Ecc as i32,
        algorithm: SigningAlgorithm::EcdsaP256Sha256 as i32,
        timestamp: current_timestamp_millis(),
    };
    
    let ecc_response = client.sign(ecc_request).await?;
    let ecc_resp = ecc_response.into_inner();
    
    // Validate ECC response
    assert_eq!(ecc_resp.key_type, KeyType::Ecc as i32, "Key type should be ECC");
    assert_eq!(ecc_resp.algorithm, SigningAlgorithm::EcdsaP256Sha256 as i32, "Algorithm should be ECDSA P-256");
    assert_eq!(ecc_resp.signature.len(), 64, "ECC P-256 signature should be 64 bytes");
    
    // Test public key retrieval
    let pubkey_request = PublicKeyRequest {
        key_type: KeyType::Rsa as i32,
        timestamp: current_timestamp_millis(),
    };
    
    let pubkey_response = client.get_public_key(pubkey_request).await?;
    let pubkey_resp = pubkey_response.into_inner();
    
    // Validate public key response
    assert_eq!(pubkey_resp.key_type, KeyType::Rsa as i32, "Key type should be RSA");
    assert!(!pubkey_resp.public_key_der.is_empty(), "Public key should not be empty");
    
    println!("✅ All crypto service tests passed!");
    Ok(())
}

#[tokio::test]
async fn test_crypto_service_performance() -> Result<(), Box<dyn std::error::Error>> {
    // Start the test server on port 50058
    start_test_server(50058).await?;
    
    // Connect client
    let mut client = connect_test_crypto_client(50058).await?;
    
    // Performance test with multiple signing requests
    let num_requests = 5;
    let mut total_latency = 0i64;
    let test_data = b"Performance test data";
    
    for i in 0..num_requests {
        let request = SignRequest {
            data: test_data.to_vec(),
            key_type: KeyType::Rsa as i32,
            algorithm: SigningAlgorithm::RsaPkcs1Sha256 as i32,
            timestamp: current_timestamp_millis(),
        };
        let request_time = request.timestamp;
        
        let response = client.sign(request).await?;
        let resp = response.into_inner();
        
        let latency = resp.response_timestamp - request_time;
        total_latency += latency;
        
        // Ensure reasonable latency (less than 100ms for local testing)
        assert!(latency < 100, "Latency should be less than 100ms, got {}ms", latency);
        assert_eq!(resp.signature.len(), 256, "RSA signature should be 256 bytes");
    }
    
    let avg_latency = total_latency / num_requests;
    println!("📊 Average crypto latency over {} requests: {}ms", num_requests, avg_latency);
    
    // Average latency should be reasonable for local testing
    assert!(avg_latency < 50, "Average latency should be less than 50ms, got {}ms", avg_latency);
    
    println!("✅ Crypto performance test passed!");
    Ok(())
}