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
        let signature = match (KeyType::from_i32(req.key_type), SigningAlgorithm::from_i32(req.algorithm)) {
            (Some(KeyType::Rsa), Some(SigningAlgorithm::RsaPkcs1Sha256)) => {
                self.crypto_keys.sign_rsa_pkcs1_sha256(&req.data)
                    .map_err(|e| Status::internal(format!("RSA PKCS#1 signing failed: {}", e)))?
            }
            (Some(KeyType::Rsa), Some(SigningAlgorithm::RsaPssSha256)) => {
                self.crypto_keys.sign_rsa_pss_sha256(&req.data)
                    .map_err(|e| Status::internal(format!("RSA PSS signing failed: {}", e)))?
            }
            (Some(KeyType::Ecc), Some(SigningAlgorithm::EcdsaP256Sha256)) => {
                self.crypto_keys.sign_ecdsa_p256_sha256(&req.data)
                    .map_err(|e| Status::internal(format!("ECDSA P-256 signing failed: {}", e)))?
            }
            (Some(KeyType::Ecc), Some(SigningAlgorithm::EcdsaP384Sha256)) => {
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
        let public_key_der = match KeyType::from_i32(req.key_type) {
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

#[tokio::main]
async fn main() -> AppResult<()> {
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

    info!("Starting gRPC server on {}", addr);

    // Create echo service
    let echo_service = EchoServiceImpl::default();
    
    // Create crypto service
    let crypto_service = CryptoServiceImpl::new()
        .map_err(|e| {
            error!("Failed to initialize crypto service: {}", e);
            std::io::Error::new(std::io::ErrorKind::Other, e)
        })?;
    
    info!("Crypto keys generated successfully");

    // Build and start the server
    let server_result = Server::builder()
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
}