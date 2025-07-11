//! gRPC server implementation for the signing service
//!
//! This module implements the gRPC server as specified in PRD Task 12: Basic gRPC Server
//! and Task 1: Implement SigningService Trait

use crate::config::{ServerConfig, TransportType, SigningAlgorithm as ConfigSigningAlgorithm};
use crate::crypto::{KeyManager, RingSigner, SigningOperation, Signer};
use crate::error::Result;
use crate::proto::signing::{
    SignRequest, SignResponse, HealthCheckRequest, HealthCheckResponse,
    GenerateKeyRequest, GenerateKeyResponse, ListKeysRequest, ListKeysResponse,
    DeleteKeyRequest, DeleteKeyResponse, VerifyRequest, VerifyResponse,
    KeyType as ProtoKeyType, SigningAlgorithm as ProtoSigningAlgorithm,
    health_check_response::ServingStatus,
    signing_service_server::SigningService,
};
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::{Mutex, oneshot};
use tonic::{Request, Response, Status};
use tonic::transport::Server;

/// gRPC signing server implementation
#[derive(Debug)]
pub struct GrpcSigningServer {
    config: ServerConfig,
    key_manager: Arc<Mutex<KeyManager>>,
    signer: Arc<RingSigner>,
    shutdown_tx: Option<oneshot::Sender<()>>,
}

/// Clone implementation for GrpcSigningServer (excluding shutdown channel)
impl Clone for GrpcSigningServer {
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
            key_manager: self.key_manager.clone(),
            signer: self.signer.clone(),
            shutdown_tx: None, // Don't clone the shutdown channel
        }
    }
}

impl GrpcSigningServer {
    /// Create a new gRPC signing server with initialized crypto components
    pub async fn new(config: ServerConfig) -> Result<Self> {
        log::info!("Initializing gRPC signing server");
        
        // Initialize key manager with configuration
        let mut key_manager = KeyManager::new(
            config.crypto.key_generation.clone(),
            config.crypto.key_loading.clone(),
        );
        
        // Initialize keys (generate or load)
        key_manager.initialize().await?;
        log::info!("Key manager initialized with {} keys", key_manager.list_keys().len());
        
        // Create signer
        let signer = RingSigner::new();
        
        Ok(Self {
            config,
            key_manager: Arc::new(Mutex::new(key_manager)),
            signer: Arc::new(signer),
            shutdown_tx: None,
        })
    }

    /// Start the server with graceful shutdown support
    pub async fn start(&self) -> Result<()> {
        use crate::proto::signing::signing_service_server::SigningServiceServer;
        
        log::info!("Starting gRPC signing server on {}:{} using {:?} transport",
                   self.config.bind_address, self.config.port, self.config.transport);

        let service = SigningServiceServer::new(self.clone());
        
        // Create server builder with performance configuration
        let mut server_builder = Server::builder()
            .max_concurrent_streams(Some(self.config.performance.max_connections))
            .timeout(self.config.performance.request_timeout)
            .tcp_keepalive(if self.config.performance.keep_alive.enabled {
                Some(self.config.performance.keep_alive.interval)
            } else {
                None
            });

        // Configure worker threads if specified
        if let Some(worker_threads) = self.config.performance.worker_threads {
            log::info!("Configuring server with {} worker threads", worker_threads);
        }

        let server = server_builder.add_service(service);

        // Start server based on transport type
        match self.config.transport {
            TransportType::Tcp => {
                let addr = format!("{}:{}", self.config.bind_address, self.config.port)
                    .parse()
                    .map_err(|_e| crate::Error::Network(crate::error::NetworkError::InvalidAddress {
                        address: format!("{}:{}", self.config.bind_address, self.config.port)
                    }))?;

                log::info!("Starting TCP server on {}", addr);
                server
                    .serve(addr)
                    .await
                    .map_err(|e| crate::Error::Transport(crate::error::TransportError::Tcp {
                        message: format!("TCP server error: {}", e)
                    }))?;
            }
            #[cfg(unix)]
            TransportType::Vsock => {
                // VSOCK implementation would go here
                // For now, return an error as VSOCK requires additional dependencies
                return Err(crate::Error::Transport(crate::error::TransportError::Vsock {
                    message: "VSOCK transport not yet implemented".to_string()
                }));
            }
        }

        Ok(())
    }

    /// Start the server with graceful shutdown channel
    pub async fn start_with_shutdown(&mut self, shutdown_rx: oneshot::Receiver<()>) -> Result<()> {
        use crate::proto::signing::signing_service_server::SigningServiceServer;
        
        log::info!("Starting gRPC signing server on {}:{} using {:?} transport",
                   self.config.bind_address, self.config.port, self.config.transport);

        let service = SigningServiceServer::new(self.clone());
        
        // Create server builder with performance configuration
        let mut server_builder = Server::builder()
            .max_concurrent_streams(Some(self.config.performance.max_connections))
            .timeout(self.config.performance.request_timeout)
            .tcp_keepalive(if self.config.performance.keep_alive.enabled {
                Some(self.config.performance.keep_alive.interval)
            } else {
                None
            });

        let server = server_builder.add_service(service);

        // Start server based on transport type with graceful shutdown
        match self.config.transport {
            TransportType::Tcp => {
                let addr = format!("{}:{}", self.config.bind_address, self.config.port)
                    .parse()
                    .map_err(|_e| crate::Error::Network(crate::error::NetworkError::InvalidAddress {
                        address: format!("{}:{}", self.config.bind_address, self.config.port)
                    }))?;

                log::info!("Starting TCP server on {} with graceful shutdown", addr);
                server
                    .serve_with_shutdown(addr, async {
                        shutdown_rx.await.ok();
                        log::info!("Graceful shutdown signal received");
                    })
                    .await
                    .map_err(|e| crate::Error::Transport(crate::error::TransportError::Tcp {
                        message: format!("TCP server error: {}", e)
                    }))?;
            }
            #[cfg(unix)]
            TransportType::Vsock => {
                return Err(crate::Error::Transport(crate::error::TransportError::Vsock {
                    message: "VSOCK transport not yet implemented".to_string()
                }));
            }
        }

        log::info!("Server shutdown complete");
        Ok(())
    }

    /// Stop the server gracefully
    pub async fn stop(&mut self) -> Result<()> {
        log::info!("Initiating graceful shutdown of gRPC signing server");
        
        if let Some(shutdown_tx) = self.shutdown_tx.take() {
            if let Err(_) = shutdown_tx.send(()) {
                log::warn!("Failed to send shutdown signal - receiver may have been dropped");
            }
        } else {
            log::warn!("No shutdown channel available - server may not be running");
        }
        
        Ok(())
    }

    /// Create shutdown channel pair
    pub fn create_shutdown_channel() -> (oneshot::Sender<()>, oneshot::Receiver<()>) {
        oneshot::channel()
    }

    /// Validate signing request
    fn validate_sign_request(&self, request: &SignRequest) -> std::result::Result<(), Status> {
        // Check if data is not empty
        if request.data.is_empty() {
            return Err(Status::invalid_argument("Data cannot be empty"));
        }

        // Check data size limits (max 1MB for safety)
        if request.data.len() > 1024 * 1024 {
            return Err(Status::invalid_argument("Data size exceeds maximum limit of 1MB"));
        }

        // Validate key_id is not empty
        if request.key_id.is_empty() {
            return Err(Status::invalid_argument("Key ID must be specified"));
        }

        Ok(())
    }

    /// Convert proto signing algorithm to config signing algorithm
    fn proto_to_config_algorithm(proto_algorithm: ProtoSigningAlgorithm) -> std::result::Result<ConfigSigningAlgorithm, Status> {
        match proto_algorithm {
            ProtoSigningAlgorithm::RsaPssSha256 => Ok(ConfigSigningAlgorithm::RsaPssSha256),
            ProtoSigningAlgorithm::RsaPssSha384 => Ok(ConfigSigningAlgorithm::RsaPssSha384),
            ProtoSigningAlgorithm::RsaPssSha512 => Ok(ConfigSigningAlgorithm::RsaPssSha512),
            ProtoSigningAlgorithm::RsaPkcs1Sha256 => Ok(ConfigSigningAlgorithm::RsaPkcs1v15Sha256),
            ProtoSigningAlgorithm::RsaPkcs1Sha384 => Ok(ConfigSigningAlgorithm::RsaPkcs1v15Sha384),
            ProtoSigningAlgorithm::RsaPkcs1Sha512 => Ok(ConfigSigningAlgorithm::RsaPkcs1v15Sha512),
            ProtoSigningAlgorithm::EcdsaSha256 => Ok(ConfigSigningAlgorithm::EcdsaP256Sha256),
            ProtoSigningAlgorithm::EcdsaSha384 => Ok(ConfigSigningAlgorithm::EcdsaP384Sha384),
            ProtoSigningAlgorithm::EcdsaSha512 => Ok(ConfigSigningAlgorithm::EcdsaP521Sha512),
            _ => Err(Status::invalid_argument("Unsupported algorithm")),
        }
    }
}

#[tonic::async_trait]
impl SigningService for GrpcSigningServer {
    /// Performs cryptographic signing operations
    async fn sign(
        &self,
        request: Request<SignRequest>,
    ) -> std::result::Result<Response<SignResponse>, Status> {
        let start_time = Instant::now();
        let request_inner = request.into_inner();
        
        // Generate correlation ID for logging
        let correlation_id = uuid::Uuid::new_v4().to_string();
        
        log::info!(
            "Received signing request [{}]: algorithm={:?}, data_len={}, key_id='{}'",
            correlation_id,
            request_inner.algorithm,
            request_inner.data.len(),
            request_inner.key_id
        );

        // Validate request
        if let Err(status) = self.validate_sign_request(&request_inner) {
            let response = SignResponse {
                signature: vec![],
                success: false,
                error_message: status.message().to_string(),
                error_code: 3, // INVALID_DATA
                processing_time_us: start_time.elapsed().as_micros() as u64,
            };
            return Ok(Response::new(response));
        }

        // Convert proto algorithm to config algorithm
        let algorithm = match Self::proto_to_config_algorithm(
            ProtoSigningAlgorithm::from_i32(request_inner.algorithm).unwrap_or(ProtoSigningAlgorithm::RsaPssSha256)
        ) {
            Ok(alg) => alg,
            Err(_) => {
                let response = SignResponse {
                    signature: vec![],
                    success: false,
                    error_message: "Invalid signing algorithm".to_string(),
                    error_code: 2, // INVALID_ALGORITHM
                    processing_time_us: start_time.elapsed().as_micros() as u64,
                };
                return Ok(Response::new(response));
            }
        };

        // Get key from key manager
        let key_manager = self.key_manager.lock().await;
        let key_pair = match key_manager.get_key(&request_inner.key_id) {
            Some(key) => key.clone(),
            None => {
                let response = SignResponse {
                    signature: vec![],
                    success: false,
                    error_message: format!("Key with ID '{}' not found", request_inner.key_id),
                    error_code: 7, // KEY_NOT_FOUND
                    processing_time_us: start_time.elapsed().as_micros() as u64,
                };
                return Ok(Response::new(response));
            }
        };

        // Create signing operation
        let signing_operation = SigningOperation::new(
            request_inner.data.clone(),
            algorithm.clone(),
            key_pair,
        );

        // Release the lock before signing
        drop(key_manager);

        // Perform signing operation
        let signing_result = match self.signer.sign(signing_operation).await {
            Ok(result) => result,
            Err(e) => {
                log::error!("Signing operation failed [{}]: {}", correlation_id, e);
                let response = SignResponse {
                    signature: vec![],
                    success: false,
                    error_message: format!("Signing failed: {}", e),
                    error_code: 5, // SIGNING_FAILED
                    processing_time_us: start_time.elapsed().as_micros() as u64,
                };
                return Ok(Response::new(response));
            }
        };

        let total_processing_time = start_time.elapsed().as_micros() as u64;

        log::info!(
            "Signing operation completed [{}]: algorithm={:?}, signature_len={}, processing_time={}μs",
            correlation_id,
            signing_result.algorithm,
            signing_result.signature.len(),
            total_processing_time
        );

        // Create successful response
        let response = SignResponse {
            signature: signing_result.signature,
            success: true,
            error_message: String::new(),
            error_code: 0, // UNSPECIFIED (success)
            processing_time_us: total_processing_time,
        };

        Ok(Response::new(response))
    }

    /// Generates a new key pair
    async fn generate_key(
        &self,
        request: Request<GenerateKeyRequest>,
    ) -> std::result::Result<Response<GenerateKeyResponse>, Status> {
        let request_inner = request.into_inner();
        
        log::info!("Generate key request: key_id='{}', key_type={:?}", 
                   request_inner.key_id, request_inner.key_type);

        // TODO: Implement key generation
        let response = GenerateKeyResponse {
            success: false,
            error_message: "Key generation not yet implemented".to_string(),
            error_code: 9, // INTERNAL_ERROR
            key_info: None,
        };

        Ok(Response::new(response))
    }

    /// Lists available keys
    async fn list_keys(
        &self,
        request: Request<ListKeysRequest>,
    ) -> std::result::Result<Response<ListKeysResponse>, Status> {
        let _request_inner = request.into_inner();
        
        log::debug!("List keys request");

        let key_manager = self.key_manager.lock().await;
        let key_ids = key_manager.list_keys();
        
        // TODO: Convert keys to KeyInfo format
        let response = ListKeysResponse {
            keys: vec![], // Placeholder - need to implement KeyInfo conversion
            success: true,
            error_message: format!("Found {} keys", key_ids.len()),
            error_code: 0, // UNSPECIFIED (success)
        };

        Ok(Response::new(response))
    }

    /// Deletes a key
    async fn delete_key(
        &self,
        request: Request<DeleteKeyRequest>,
    ) -> std::result::Result<Response<DeleteKeyResponse>, Status> {
        let request_inner = request.into_inner();
        
        log::info!("Delete key request: key_id='{}'", request_inner.key_id);

        // TODO: Implement key deletion
        let response = DeleteKeyResponse {
            success: false,
            error_message: "Key deletion not yet implemented".to_string(),
            error_code: 9, // INTERNAL_ERROR
        };

        Ok(Response::new(response))
    }

    /// Health check endpoint
    async fn health_check(
        &self,
        request: Request<HealthCheckRequest>,
    ) -> std::result::Result<Response<HealthCheckResponse>, Status> {
        let request_inner = request.into_inner();
        
        log::debug!("Health check request for service: {}", request_inner.service);

        // Check key manager status
        let key_manager = self.key_manager.lock().await;
        let key_count = key_manager.list_keys().len();
        drop(key_manager);

        // Determine service status
        let (status, message) = if key_count > 0 {
            log::debug!("Health check: Service healthy with {} keys available", key_count);
            (ServingStatus::Serving, format!("Service healthy with {} keys available", key_count))
        } else {
            log::warn!("Health check: Service unhealthy - no keys available");
            (ServingStatus::NotServing, "Service unhealthy - no keys available".to_string())
        };

        let response = HealthCheckResponse {
            status: status as i32,
            message,
        };

        Ok(Response::new(response))
    }

    /// Verifies a signature
    async fn verify(
        &self,
        request: Request<VerifyRequest>,
    ) -> std::result::Result<Response<VerifyResponse>, Status> {
        let request_inner = request.into_inner();
        
        log::info!("Verify request: key_id='{}', algorithm={:?}, data_len={}, signature_len={}", 
                   request_inner.key_id, request_inner.algorithm, 
                   request_inner.data.len(), request_inner.signature.len());

        // TODO: Implement signature verification
        let response = VerifyResponse {
            valid: false,
            success: false,
            error_message: "Signature verification not yet implemented".to_string(),
            error_code: 9, // INTERNAL_ERROR
        };

        Ok(Response::new(response))
    }
}