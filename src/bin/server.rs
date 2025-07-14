//! gRPC server implementation with echo service

use grpc_performance_rs::{
    echo::{echo_service_server::{EchoService, EchoServiceServer}, EchoRequest, EchoResponse},
    crypto::{
        crypto_service_server::{CryptoService, CryptoServiceServer},
        SignRequest, SignResponse, PublicKeyRequest, PublicKeyResponse,
        KeyType, SigningAlgorithm
    },
    current_timestamp_millis, AppResult, DEFAULT_SERVER_ADDR, DEFAULT_LOG_LEVEL, CryptoKeys,
};
use log::{info, error, debug};
use std::env;
use tonic::{transport::Server, Request, Response, Status};

/// Echo service implementation
#[derive(Debug, Default)]
pub struct EchoServiceImpl;

/// Crypto service implementation
#[derive(Debug)]
pub struct CryptoServiceImpl {
    crypto_keys: CryptoKeys,
}

impl CryptoServiceImpl {
    pub fn new() -> AppResult<Self> {
        let crypto_keys = CryptoKeys::generate()?;
        Ok(CryptoServiceImpl { crypto_keys })
    }
}

#[tonic::async_trait]
impl EchoService for EchoServiceImpl {
    async fn echo(
        &self,
        request: Request<EchoRequest>,
    ) -> Result<Response<EchoResponse>, Status> {
        let req = request.into_inner();
        let response_timestamp = current_timestamp_millis();
        
        debug!(
            "Received echo request: payload_len={}, request_timestamp={}",
            req.payload.len(),
            req.timestamp
        );

        // Log the request details
        info!(
            "Echo request processed: payload='{}', latency={}ms",
            req.payload,
            response_timestamp - req.timestamp
        );

        let response = EchoResponse {
            payload: req.payload,
            request_timestamp: req.timestamp,
            response_timestamp,
        };

        Ok(Response::new(response))
    }
}

#[tonic::async_trait]
impl CryptoService for CryptoServiceImpl {
    async fn sign(
        &self,
        request: Request<SignRequest>,
    ) -> Result<Response<SignResponse>, Status> {
        let req = request.into_inner();
        let response_timestamp = current_timestamp_millis();
        
        debug!(
            "Received sign request: data_len={}, key_type={:?}, algorithm={:?}",
            req.data.len(),
            req.key_type,
            req.algorithm
        );

        // Perform signing based on key type and algorithm
        let key_type = KeyType::try_from(req.key_type).map_err(|_| Status::invalid_argument("Invalid key type"))?;
        let algorithm = SigningAlgorithm::try_from(req.algorithm).map_err(|_| Status::invalid_argument("Invalid algorithm"))?;
        
        let signature = match (key_type, algorithm) {
            (KeyType::Rsa, SigningAlgorithm::RsaPkcs1Sha256) => {
                self.crypto_keys.sign_rsa_pkcs1_sha256(&req.data)
                    .map_err(|e| Status::internal(format!("RSA PKCS#1 signing failed: {}", e)))?
            }
            (KeyType::Rsa, SigningAlgorithm::RsaPssSha256) => {
                self.crypto_keys.sign_rsa_pss_sha256(&req.data)
                    .map_err(|e| Status::internal(format!("RSA PSS signing failed: {}", e)))?
            }
            (KeyType::Ecc, SigningAlgorithm::EcdsaP256Sha256) => {
                self.crypto_keys.sign_ecdsa_p256_sha256(&req.data)
                    .map_err(|e| Status::internal(format!("ECDSA P-256 signing failed: {}", e)))?
            }
            (KeyType::Ecc, SigningAlgorithm::EcdsaP384Sha256) => {
                self.crypto_keys.sign_ecdsa_p384_sha384(&req.data)
                    .map_err(|e| Status::internal(format!("ECDSA P-384 signing failed: {}", e)))?
            }
            _ => {
                return Err(Status::invalid_argument(
                    format!("Unsupported key type {:?} or algorithm {:?}", req.key_type, req.algorithm)
                ));
            }
        };

        info!(
            "Sign request processed: key_type={:?}, algorithm={:?}, signature_len={}, latency={}ms",
            req.key_type,
            req.algorithm,
            signature.len(),
            response_timestamp - req.timestamp
        );

        let response = SignResponse {
            signature,
            key_type: req.key_type,
            algorithm: req.algorithm,
            request_timestamp: req.timestamp,
            response_timestamp,
        };

        Ok(Response::new(response))
    }

    async fn get_public_key(
        &self,
        request: Request<PublicKeyRequest>,
    ) -> Result<Response<PublicKeyResponse>, Status> {
        let req = request.into_inner();
        let response_timestamp = current_timestamp_millis();
        
        debug!(
            "Received public key request: key_type={:?}",
            req.key_type
        );

        // Get public key based on key type
        let public_key_der = match KeyType::try_from(req.key_type) {
            Some(KeyType::Rsa) => {
                self.crypto_keys.get_rsa_public_key_der()
                    .map_err(|e| Status::internal(format!("RSA public key retrieval failed: {}", e)))?
            }
            Some(KeyType::Ecc) => {
                // Default to P-256 for ECC requests
                self.crypto_keys.get_ecc_p256_public_key_der()
                    .map_err(|e| Status::internal(format!("ECC public key retrieval failed: {}", e)))?
            }
            _ => {
                return Err(Status::invalid_argument(
                    format!("Unsupported key type: {:?}", req.key_type)
                ));
            }
        };

        info!(
            "Public key request processed: key_type={:?}, key_len={}, latency={}ms",
            req.key_type,
            public_key_der.len(),
            response_timestamp - req.timestamp
        );

        let response = PublicKeyResponse {
            public_key_der,
            key_type: req.key_type,
            request_timestamp: req.timestamp,
            response_timestamp,
        };

        Ok(Response::new(response))
    }
}

fn main() -> AppResult<()> {
    // Initialize logging
    let log_level = env::var("RUST_LOG").unwrap_or_else(|_| DEFAULT_LOG_LEVEL.to_string());
    env::set_var("RUST_LOG", log_level);
    env_logger::init();

    // Parse server address from environment or use default
    let addr = env::var("SERVER_ADDR")
        .unwrap_or_else(|_| DEFAULT_SERVER_ADDR.to_string())
        .parse()
        .map_err(|e| {
            error!("Invalid server address: {}", e);
            std::io::Error::new(std::io::ErrorKind::InvalidInput, e)
        })?;

    // Get number of worker threads from environment or use all available CPU cores
    let worker_threads = env::var("WORKER_THREADS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or_else(|| num_cpus::get());

    info!("Starting gRPC server on {} with {} worker threads", addr, worker_threads);

    // Optimize tokio runtime for better multi-threading performance
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(worker_threads)
        .thread_name("grpc-server-worker")
        .thread_stack_size(3 * 1024 * 1024) // 3MB stack size
        .enable_all()
        .build()
        .map_err(|e| {
            error!("Failed to build tokio runtime: {}", e);
            std::io::Error::new(std::io::ErrorKind::Other, e)
        })?;

    runtime.block_on(async {
        // Create echo service
        let echo_service = EchoServiceImpl::default();
        
        // Create crypto service
        let crypto_service = CryptoServiceImpl::new()
            .map_err(|e| {
                error!("Failed to initialize crypto service: {}", e);
                std::io::Error::new(std::io::ErrorKind::Other, e)
            })?;
        
        info!("Crypto keys generated successfully");

        // Configure server with optimizations
        let mut server_builder = Server::builder()
            .tcp_keepalive(Some(std::time::Duration::from_secs(30)))
            .tcp_nodelay(true)
            .http2_keepalive_interval(Some(std::time::Duration::from_secs(30)))
            .http2_adaptive_window(Some(true))
            .max_concurrent_streams(Some(1000))
            .initial_stream_window_size(Some(1024 * 1024)) // 1MB
            .initial_connection_window_size(Some(1024 * 1024)) // 1MB
            .max_frame_size(Some(16384)); // 16KB

        // Build and start the server
        let server_result = server_builder
            .add_service(EchoServiceServer::new(echo_service))
            .add_service(CryptoServiceServer::new(crypto_service))
            .serve(addr)
            .await;

        match server_result {
            Ok(_) => {
                info!("Server shutdown gracefully");
                Ok(())
            }
            Err(e) => {
                error!("Server error: {}", e);
                Err(e.into())
            }
        }
    })
}