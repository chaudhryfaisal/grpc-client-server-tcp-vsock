//! gRPC server implementation with echo service

use grpc_performance_rs::{
    echo::{echo_service_server::{EchoService, EchoServiceServer}, EchoRequest, EchoResponse},
    crypto::{
        crypto_service_server::{CryptoService, CryptoServiceServer},
        SignRequest, SignResponse, PublicKeyRequest, PublicKeyResponse,
        KeyType, SigningAlgorithm
    },
    current_timestamp_millis, AppResult, DEFAULT_SERVER_ADDR, DEFAULT_LOG_LEVEL, CryptoKeys,
    transport::{TransportConfig, TransportFactory},
};
use log::{info, error, debug};
use std::env;
use std::str::FromStr;
use tonic::{transport::Server, Request, Response, Status};
use tower::Service;
use hyper::service::service_fn;

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

        debug!(
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
        let key_type = KeyType::try_from(req.key_type).map_err(|_| Status::invalid_argument("Invalid key type"))?;
        let public_key_der = match key_type {
            KeyType::Rsa => {
                self.crypto_keys.get_rsa_public_key_der()
                    .map_err(|e| Status::internal(format!("RSA public key retrieval failed: {}", e)))?
            }
            KeyType::Ecc => {
                // Default to P-256 for ECC requests
                self.crypto_keys.get_ecc_p256_public_key_der()
                    .map_err(|e| Status::internal(format!("ECC public key retrieval failed: {}", e)))?
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
    let addr_str = env::var("SERVER_ADDR")
        .unwrap_or_else(|_| DEFAULT_SERVER_ADDR.to_string());
    
    let transport_config = TransportConfig::from_str(&addr_str)
        .map_err(|e| {
            error!("Invalid server address '{}': {}", addr_str, e);
            std::io::Error::new(std::io::ErrorKind::InvalidInput, e)
        })?;

    // Get number of worker threads from environment or use all available CPU cores
    let worker_threads = env::var("WORKER_THREADS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or_else(|| num_cpus::get());

    info!("Starting gRPC server on {} ({}) with {} worker threads",
          transport_config,
          if transport_config.is_tcp() { "TCP" } else { "VSOCK" },
          worker_threads);

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

        // Create the gRPC router with services
        let router = Server::builder()
            .tcp_keepalive(Some(std::time::Duration::from_secs(30)))
            .tcp_nodelay(true)
            .http2_keepalive_interval(Some(std::time::Duration::from_secs(30)))
            .http2_adaptive_window(Some(true))
            .max_concurrent_streams(Some(1000))
            .initial_stream_window_size(Some(1024 * 1024)) // 1MB
            .initial_connection_window_size(Some(1024 * 1024)) // 1MB
            .max_frame_size(Some(16384)) // 16KB
            .add_service(EchoServiceServer::new(echo_service))
            .add_service(CryptoServiceServer::new(crypto_service))
            .into_router();

        // Bind to the transport
        let mut listener = TransportFactory::bind(&transport_config).await
            .map_err(|e| {
                error!("Failed to bind to {}: {}", transport_config, e);
                std::io::Error::new(std::io::ErrorKind::AddrInUse, e)
            })?;

        let local_addr = listener.local_addr()
            .map_err(|e| {
                error!("Failed to get local address: {}", e);
                std::io::Error::new(std::io::ErrorKind::Other, e)
            })?;

        info!("gRPC server listening on {}", local_addr);

        // Custom server loop to accept connections
        loop {
            match listener.accept().await {
                Ok(connection) => {
                    let remote_addr = connection.remote_addr()
                        .unwrap_or_else(|_| "unknown".to_string());
                    
                    debug!("Accepted connection from {}", remote_addr);
                    
                    // Clone the router for this connection
                    let router_clone = router.clone();
                    
                    // Spawn a task to handle this connection
                    tokio::spawn(async move {
                        // Create a hyper service from the connection
                        let service = service_fn(move |req| {
                            router_clone.clone().call(req)
                        });
                        
                        // Serve HTTP/2 over this connection using hyper 0.14 API
                        if let Err(e) = hyper::server::conn::Http::new()
                            .http2_only(true)
                            .serve_connection(connection, service)
                            .await
                        {
                            debug!("Connection from {} closed with error: {}", remote_addr, e);
                        } else {
                            debug!("Connection from {} closed gracefully", remote_addr);
                        }
                    });
                }
                Err(e) => {
                    error!("Failed to accept connection: {}", e);
                    // Continue accepting other connections
                }
            }
        }
    })
}