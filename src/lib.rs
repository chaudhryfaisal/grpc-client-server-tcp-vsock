//! Shared types and utilities for the gRPC performance testing system

use std::time::{SystemTime, UNIX_EPOCH};
use http;

// Include the generated proto code
pub mod echo {
    tonic::include_proto!("echo");
}

pub mod crypto {
    tonic::include_proto!("crypto");
}

/// Get current timestamp in milliseconds since Unix epoch
pub fn current_timestamp_millis() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}

/// Custom error type for the application
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("gRPC transport error: {0}")]
    Transport(#[from] tonic::transport::Error),
    #[error("gRPC status error: {0}")]
    Status(#[from] tonic::Status),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Invalid URI: {0}")]
    InvalidUri(#[from] http::uri::InvalidUri),
    #[error("Invalid configuration: {0}")]
    Config(String),
    #[error("Cryptographic error: {0}")]
    Crypto(String),
    #[error("Key generation error: {0}")]
    KeyGeneration(String),
    #[error("Signing error: {0}")]
    Signing(String),
    #[error("Unsupported algorithm: {0}")]
    UnsupportedAlgorithm(String),
}

/// Result type alias for the application
pub type AppResult<T> = Result<T, AppError>;

/// Default server address for TCP connections
pub const DEFAULT_SERVER_ADDR: &str = "127.0.0.1:50051";

/// Default log level for the application
pub const DEFAULT_LOG_LEVEL: &str = "info";

/// Cryptographic key manager for RSA and ECC keys
/// For now, this is a placeholder implementation that will be completed later
#[derive(Debug)]
pub struct CryptoKeys {
    // TODO: Add actual key storage when ring API is properly configured
}

impl CryptoKeys {
    /// Generate new RSA and ECC key pairs
    /// TODO: Implement actual key generation with ring crate
    pub fn generate() -> AppResult<Self> {
        // Placeholder implementation
        Ok(CryptoKeys {})
    }
    
    /// Get RSA public key in DER format
    /// TODO: Implement actual RSA public key retrieval
    pub fn get_rsa_public_key_der(&self) -> AppResult<Vec<u8>> {
        // Return a placeholder public key for now
        Ok(vec![0x30, 0x82, 0x01, 0x22]) // Placeholder DER header
    }
    
    /// Get ECC P-256 public key in DER format
    /// TODO: Implement actual ECC P-256 public key retrieval
    pub fn get_ecc_p256_public_key_der(&self) -> AppResult<Vec<u8>> {
        // Return a placeholder public key for now
        Ok(vec![0x30, 0x59, 0x30, 0x13]) // Placeholder DER header
    }
    
    /// Get ECC P-384 public key in DER format
    /// TODO: Implement actual ECC P-384 public key retrieval
    pub fn get_ecc_p384_public_key_der(&self) -> AppResult<Vec<u8>> {
        // Return a placeholder public key for now
        Ok(vec![0x30, 0x76, 0x30, 0x10]) // Placeholder DER header
    }
    
    /// Sign data using RSA PKCS#1 v1.5 with SHA-256
    /// TODO: Implement actual RSA PKCS#1 signing
    pub fn sign_rsa_pkcs1_sha256(&self, _data: &[u8]) -> AppResult<Vec<u8>> {
        // Return a placeholder signature for now
        Ok(vec![0u8; 256]) // 2048-bit RSA signature size
    }
    
    /// Sign data using RSA PSS with SHA-256
    /// TODO: Implement actual RSA PSS signing
    pub fn sign_rsa_pss_sha256(&self, _data: &[u8]) -> AppResult<Vec<u8>> {
        // Return a placeholder signature for now
        Ok(vec![0u8; 256]) // 2048-bit RSA signature size
    }
    
    /// Sign data using ECDSA P-256 with SHA-256
    /// TODO: Implement actual ECDSA P-256 signing
    pub fn sign_ecdsa_p256_sha256(&self, _data: &[u8]) -> AppResult<Vec<u8>> {
        // Return a placeholder signature for now
        Ok(vec![0u8; 64]) // P-256 signature size
    }
    
    /// Sign data using ECDSA P-384 with SHA-384
    /// TODO: Implement actual ECDSA P-384 signing
    pub fn sign_ecdsa_p384_sha384(&self, _data: &[u8]) -> AppResult<Vec<u8>> {
        // Return a placeholder signature for now
        Ok(vec![0u8; 96]) // P-384 signature size
    }
}