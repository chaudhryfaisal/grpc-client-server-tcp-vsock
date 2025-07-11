//! gRPC client implementation for the signing service
//!
//! This module provides a high-level client interface for interacting with
//! the gRPC signing service, including connection management, retry logic,
//! and comprehensive error handling.

use crate::config::{ClientConfig, HashAlgorithm};
use crate::error::{Result, ValidationError};
use grpc_shared::proto::signing::{
    signing_service_client::SigningServiceClient,
    SignRequest, SignResponse, GenerateKeyRequest, GenerateKeyResponse,
    ListKeysRequest, ListKeysResponse, DeleteKeyRequest, DeleteKeyResponse,
    HealthCheckRequest, HealthCheckResponse, VerifyRequest, VerifyResponse,
    KeyType as ProtoKeyType, SigningAlgorithm as ProtoSigningAlgorithm,
    HashAlgorithm as ProtoHashAlgorithm,
};
use grpc_shared::{KeyType, SigningAlgorithm, TransportType};
use grpc_shared::error::{Error, NetworkError, TransportError};
use tonic::transport::{Channel, Endpoint};
use tonic::{Request, Status};
use std::time::Duration;

/// High-level gRPC signing client with connection management and retry logic
pub struct GrpcSigningClient {
    config: ClientConfig,
    client: Option<SigningServiceClient<Channel>>,
}

impl GrpcSigningClient {
    /// Create a new gRPC signing client
    pub fn new(config: ClientConfig) -> Self {
        log::info!("Creating gRPC signing client for {}", config.server_address);
        
        Self {
            config,
            client: None,
        }
    }

    /// Connect to the gRPC server
    pub async fn connect(&mut self) -> Result<()> {
        log::info!("Connecting to gRPC server at {} using {:?} transport",
                   self.config.server_address, self.config.transport);

        match self.config.transport {
            TransportType::Tcp => {
                let uri = format!("http://{}", self.config.server_address);
                let endpoint = Endpoint::from_shared(uri)
                    .map_err(|e| Error::Network(NetworkError::ConnectionFailed {
                        message: format!("Failed to connect to {}: {}", self.config.server_address, e),
                    }))?
                    .timeout(self.config.connection_pool.idle_timeout)
                    .connect_timeout(self.config.connection_pool.idle_timeout)
                    .tcp_keepalive(if self.config.connection_pool.max_size > 0 {
                        Some(Duration::from_secs(60))
                    } else {
                        None
                    });

                let channel = endpoint.connect().await
                    .map_err(|e| Error::Network(NetworkError::ConnectionFailed {
                        message: format!("Failed to connect to {}: {}", self.config.server_address, e),
                    }))?;
                
                self.client = Some(SigningServiceClient::new(channel));
            }
            #[cfg(unix)]
            TransportType::Vsock => {
                return Err(Error::Transport(TransportError::Vsock {
                    message: "VSOCK transport not yet implemented".to_string(),
                }));
            }
        }

        log::info!("Successfully connected to gRPC server");
        Ok(())
    }

    /// Disconnect from the gRPC server
    pub async fn disconnect(&mut self) -> Result<()> {
        if self.client.is_some() {
            log::info!("Disconnecting from gRPC server");
            self.client = None;
            log::info!("Successfully disconnected from gRPC server");
        }
        Ok(())
    }

    /// Ensure client is connected, with retry logic
    async fn ensure_connected(&mut self) -> Result<&mut SigningServiceClient<Channel>> {
        if self.client.is_none() {
            self.connect().await?;
        }

        // Test connection with retry logic
        if self.config.retry.enabled {
            let mut attempts = 0;
            while attempts < self.config.retry.max_attempts {
                if let Some(ref mut client) = self.client {
                    // Try a simple health check to verify connection
                    let health_req = Request::new(HealthCheckRequest {
                        service: "signing".to_string(),
                    });
                    
                    match client.health_check(health_req).await {
                        Ok(_) => break,
                        Err(e) => {
                            attempts += 1;
                            if attempts >= self.config.retry.max_attempts {
                                return Err(Error::Network(NetworkError::ConnectionFailed {
                                    message: format!("Failed to connect to {} after {} attempts: {}",
                                                    self.config.server_address, attempts, e),
                                }));
                            }
                            log::warn!("Connection attempt {} failed: {}. Retrying in {:?}...", 
                                      attempts, e, self.config.retry.initial_delay);
                            
                            tokio::time::sleep(self.config.retry.initial_delay).await;
                            
                            // Reconnect
                            self.client = None;
                            self.connect().await?;
                        }
                    }
                }
            }
        }

        self.client.as_mut().ok_or_else(||
            Error::Network(NetworkError::ConnectionLost {
                reason: format!("Connection to {} was lost", self.config.server_address),
            })
        )
    }

    /// Sign data using the specified key and algorithm
    pub async fn sign(
        &mut self,
        data: &[u8],
        key_id: &str,
        key_type: KeyType,
        algorithm: SigningAlgorithm,
    ) -> Result<SignResponse> {
        // Input validation
        if data.is_empty() {
            return Err(Error::Validation(ValidationError::InvalidInput {
                field: "data".to_string(),
                message: "Data cannot be empty".to_string(),
            }));
        }

        if key_id.is_empty() {
            return Err(Error::Validation(ValidationError::InvalidInput {
                field: "key_id".to_string(),
                message: "Key ID cannot be empty".to_string(),
            }));
        }

        // Convert types and extract config values before borrowing
        let proto_key_type = Self::convert_key_type_static(key_type);
        let proto_algorithm = Self::convert_signing_algorithm_static(algorithm);
        let timeout = self.config.connection_pool.idle_timeout;
        let timeout_ms = timeout.as_millis() as u64;
        let server_address = self.config.server_address.clone();
        
        let client = self.ensure_connected().await?;
        
        let request = Request::new(SignRequest {
            data: data.to_vec(),
            key_type: proto_key_type as i32,
            algorithm: proto_algorithm as i32,
            key_id: key_id.to_string(),
        });

        let response = tokio::time::timeout(
            timeout,
            client.sign(request)
        )
        .await
        .map_err(|_| Error::Network(grpc_shared::error::NetworkError::ConnectionTimeout {
            timeout_ms
        }))?
        .map_err(|e| Self::convert_grpc_error_static(e, &server_address, timeout_ms))?;

        Ok(response.into_inner())
    }

    /// Generate a new key pair
    pub async fn generate_key(
        &mut self,
        key_id: &str,
        key_type: KeyType,
        description: &str,
    ) -> Result<GenerateKeyResponse> {
        // Convert types and extract config values before borrowing
        let proto_key_type = Self::convert_key_type_static(key_type);
        let timeout = self.config.connection_pool.idle_timeout;
        let timeout_ms = timeout.as_millis() as u64;
        let server_address = self.config.server_address.clone();
        
        let client = self.ensure_connected().await?;
        
        let request = Request::new(GenerateKeyRequest {
            key_id: key_id.to_string(),
            key_type: proto_key_type as i32,
            description: description.to_string(),
        });

        let response = tokio::time::timeout(
            timeout,
            client.generate_key(request)
        )
        .await
        .map_err(|_| Error::Network(grpc_shared::error::NetworkError::ConnectionTimeout {
            timeout_ms
        }))?
        .map_err(|e| Self::convert_grpc_error_static(e, &server_address, timeout_ms))?;

        Ok(response.into_inner())
    }

    /// List available keys
    pub async fn list_keys(
        &mut self,
        key_type_filter: Option<KeyType>,
        active_only: Option<bool>,
    ) -> Result<ListKeysResponse> {
        // Convert types and extract config values before borrowing
        let proto_key_type_filter = key_type_filter.map(|kt| Self::convert_key_type_static(kt) as i32);
        let timeout = self.config.connection_pool.idle_timeout;
        let timeout_ms = timeout.as_millis() as u64;
        let server_address = self.config.server_address.clone();
        
        let client = self.ensure_connected().await?;
        
        let request = Request::new(ListKeysRequest {
            key_type_filter: proto_key_type_filter,
            active_only,
        });

        let response = tokio::time::timeout(
            timeout,
            client.list_keys(request)
        )
        .await
        .map_err(|_| Error::Network(grpc_shared::error::NetworkError::ConnectionTimeout {
            timeout_ms
        }))?
        .map_err(|e| Self::convert_grpc_error_static(e, &server_address, timeout_ms))?;

        Ok(response.into_inner())
    }

    /// Delete a key
    pub async fn delete_key(&mut self, key_id: &str) -> Result<DeleteKeyResponse> {
        if key_id.is_empty() {
            return Err(Error::Validation(ValidationError::InvalidInput {
                field: "key_id".to_string(),
                message: "Key ID cannot be empty".to_string(),
            }));
        }

        // Extract config values before borrowing
        let timeout = self.config.connection_pool.idle_timeout;
        let timeout_ms = timeout.as_millis() as u64;
        let server_address = self.config.server_address.clone();
        
        let client = self.ensure_connected().await?;
        
        let request = Request::new(DeleteKeyRequest {
            key_id: key_id.to_string(),
        });

        let response = tokio::time::timeout(
            timeout,
            client.delete_key(request)
        )
        .await
        .map_err(|_| Error::Network(grpc_shared::error::NetworkError::ConnectionTimeout {
            timeout_ms
        }))?
        .map_err(|e| Self::convert_grpc_error_static(e, &server_address, timeout_ms))?;

        Ok(response.into_inner())
    }

    /// Health check
    pub async fn health_check(&mut self, service: Option<&str>) -> Result<HealthCheckResponse> {
        // Extract config values before borrowing
        let timeout = self.config.connection_pool.idle_timeout;
        let timeout_ms = timeout.as_millis() as u64;
        let server_address = self.config.server_address.clone();
        
        let client = self.ensure_connected().await?;
        
        let request = Request::new(HealthCheckRequest {
            service: service.unwrap_or("signing").to_string(),
        });

        let response = tokio::time::timeout(
            timeout,
            client.health_check(request)
        )
        .await
        .map_err(|_| Error::Network(grpc_shared::error::NetworkError::ConnectionTimeout {
            timeout_ms
        }))?
        .map_err(|e| Self::convert_grpc_error_static(e, &server_address, timeout_ms))?;

        Ok(response.into_inner())
    }

    /// Verify a signature
    pub async fn verify(
        &mut self,
        data: &[u8],
        signature: &[u8],
        key_id: &str,
        algorithm: SigningAlgorithm,
        hash_algorithm: HashAlgorithm,
    ) -> Result<VerifyResponse> {
        // Input validation
        if data.is_empty() {
            return Err(Error::Validation(ValidationError::InvalidInput {
                field: "data".to_string(),
                message: "Data cannot be empty".to_string(),
            }));
        }

        if signature.is_empty() {
            return Err(Error::Validation(ValidationError::InvalidInput {
                field: "signature".to_string(),
                message: "Signature cannot be empty".to_string(),
            }));
        }

        if key_id.is_empty() {
            return Err(Error::Validation(ValidationError::InvalidInput {
                field: "key_id".to_string(),
                message: "Key ID cannot be empty".to_string(),
            }));
        }

        // Convert types and extract config values before borrowing
        let proto_algorithm = Self::convert_signing_algorithm_static(algorithm);
        let proto_hash_algorithm = Self::convert_hash_algorithm_static(hash_algorithm);
        let timeout = self.config.connection_pool.idle_timeout;
        let timeout_ms = timeout.as_millis() as u64;
        let server_address = self.config.server_address.clone();
        
        let client = self.ensure_connected().await?;
        
        let request = Request::new(VerifyRequest {
            data: data.to_vec(),
            signature: signature.to_vec(),
            key_id: key_id.to_string(),
            algorithm: proto_algorithm as i32,
            hash_algorithm: proto_hash_algorithm as i32,
        });

        let response = tokio::time::timeout(
            timeout,
            client.verify(request)
        )
        .await
        .map_err(|_| Error::Network(grpc_shared::error::NetworkError::ConnectionTimeout {
            timeout_ms
        }))?
        .map_err(|e| Self::convert_grpc_error_static(e, &server_address, timeout_ms))?;

        Ok(response.into_inner())
    }

    // Helper methods for type conversion (static to avoid borrowing issues)
    fn convert_key_type_static(key_type: KeyType) -> ProtoKeyType {
        match key_type {
            KeyType::Rsa2048 => ProtoKeyType::Rsa2048,
            KeyType::Rsa3072 => ProtoKeyType::Rsa3072,
            KeyType::Rsa4096 => ProtoKeyType::Rsa4096,
            KeyType::EccP256 => ProtoKeyType::EccP256,
            KeyType::EccP384 => ProtoKeyType::EccP384,
            KeyType::EccP521 => ProtoKeyType::EccP521,
        }
    }

    fn convert_signing_algorithm_static(algorithm: SigningAlgorithm) -> ProtoSigningAlgorithm {
        match algorithm {
            SigningAlgorithm::RsaPssSha256 => ProtoSigningAlgorithm::RsaPssSha256,
            SigningAlgorithm::RsaPssSha384 => ProtoSigningAlgorithm::RsaPssSha384,
            SigningAlgorithm::RsaPssSha512 => ProtoSigningAlgorithm::RsaPssSha512,
            SigningAlgorithm::RsaPkcs1v15Sha256 => ProtoSigningAlgorithm::RsaPkcs1Sha256,
            SigningAlgorithm::RsaPkcs1v15Sha384 => ProtoSigningAlgorithm::RsaPkcs1Sha384,
            SigningAlgorithm::RsaPkcs1v15Sha512 => ProtoSigningAlgorithm::RsaPkcs1Sha512,
            SigningAlgorithm::EcdsaP256Sha256 => ProtoSigningAlgorithm::EcdsaSha256,
            SigningAlgorithm::EcdsaP384Sha384 => ProtoSigningAlgorithm::EcdsaSha384,
            SigningAlgorithm::EcdsaP521Sha512 => ProtoSigningAlgorithm::EcdsaSha512,
        }
    }

    fn convert_hash_algorithm_static(hash_algorithm: HashAlgorithm) -> ProtoHashAlgorithm {
        match hash_algorithm {
            HashAlgorithm::Sha256 => ProtoHashAlgorithm::Sha256,
            HashAlgorithm::Sha384 => ProtoHashAlgorithm::Sha384,
            HashAlgorithm::Sha512 => ProtoHashAlgorithm::Sha512,
            HashAlgorithm::Unspecified => ProtoHashAlgorithm::Unspecified,
        }
    }

    fn convert_grpc_error_static(status: Status, server_address: &str, timeout_ms: u64) -> Error {
        match status.code() {
            tonic::Code::Unavailable => {
                Error::Network(NetworkError::ConnectionLost {
                    reason: format!("Connection to {} unavailable: {}", server_address, status.message()),
                })
            }
            tonic::Code::DeadlineExceeded => {
                Error::Network(NetworkError::ConnectionTimeout {
                    timeout_ms
                })
            }
            tonic::Code::InvalidArgument => {
                Error::Validation(ValidationError::InvalidInput {
                    field: "request".to_string(),
                    message: status.message().to_string(),
                })
            }
            _ => {
                Error::Network(NetworkError::ConnectionFailed {
                    message: format!("gRPC error for {}: {} - {}",
                                   server_address, status.code(), status.message()),
                })
            }
        }
    }
}