//! Shared types and utilities for the gRPC performance testing system

use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use http;
use ring::signature;
use ring::signature::{RsaKeyPair, EcdsaKeyPair, KeyPair};
use rsa::{RsaPrivateKey, pkcs8::EncodePrivateKey};

// Include the generated proto code
pub mod echo {
    tonic::include_proto!("echo");
}

pub mod crypto {
    tonic::include_proto!("crypto");
}

// Transport abstraction layer
pub mod transport;

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
    CryptoError(String),
    #[error("Key generation error: {0}")]
    KeyGeneration(String),
    #[error("Signing error: {0}")]
    Signing(String),
    #[error("Unsupported algorithm: {0}")]
    UnsupportedAlgorithm(String),
    #[error("Ring error: {0}")]
    Ring(String),
    #[error("Ring key rejected: {0}")]
    KeyRejected(String),
}

/// Result type alias for the application
pub type AppResult<T> = Result<T, AppError>;

/// Default server address for TCP connections
pub const DEFAULT_SERVER_ADDR: &str = "127.0.0.1:50051";

/// Default log level for the application
pub const DEFAULT_LOG_LEVEL: &str = "info";

/// Cryptographic key manager for RSA and ECC keys using ring crate
#[derive(Debug)]
pub struct CryptoKeys {
    rsa_key_pair: Option<Arc<RsaKeyPair>>,
    ecc_p256_key_pair: Arc<EcdsaKeyPair>,
    ecc_p384_key_pair: Arc<EcdsaKeyPair>,
    rng: ring::rand::SystemRandom,
}

impl CryptoKeys {
    /// Generate new RSA and ECC key pairs using ring crate
    pub fn generate() -> AppResult<Self> {
        let rng = ring::rand::SystemRandom::new();
        
        // Generate RSA key pair - ring doesn't provide RSA key generation
        // We'll use a minimal test key for demonstration
        let rsa_key_pair = Self::create_test_rsa_key().ok();
        if rsa_key_pair.is_none() {
            eprintln!("Warning: Failed to create RSA key pair - RSA operations will be unavailable");
        }
        
        // Generate ECC P-256 key pair
        let ecc_p256_key_pair = {
            let ecc_p256_pkcs8 = EcdsaKeyPair::generate_pkcs8(&signature::ECDSA_P256_SHA256_FIXED_SIGNING, &rng)
                .map_err(|e| AppError::CryptoError(format!("Failed to generate P-256 key: {:?}", e)))?;
            EcdsaKeyPair::from_pkcs8(&signature::ECDSA_P256_SHA256_FIXED_SIGNING, ecc_p256_pkcs8.as_ref(), &rng)
                .map_err(|e| AppError::CryptoError(format!("Failed to create P-256 key pair: {:?}", e)))?
        };
        
        // Generate ECC P-384 key pair
        let ecc_p384_key_pair = {
            let ecc_p384_pkcs8 = EcdsaKeyPair::generate_pkcs8(&signature::ECDSA_P384_SHA384_FIXED_SIGNING, &rng)
                .map_err(|e| AppError::CryptoError(format!("Failed to generate P-384 key: {:?}", e)))?;
            EcdsaKeyPair::from_pkcs8(&signature::ECDSA_P384_SHA384_FIXED_SIGNING, ecc_p384_pkcs8.as_ref(), &rng)
                .map_err(|e| AppError::CryptoError(format!("Failed to create P-384 key pair: {:?}", e)))?
        };
        
        Ok(CryptoKeys {
            rsa_key_pair: rsa_key_pair.map(Arc::new),
            ecc_p256_key_pair: Arc::new(ecc_p256_key_pair),
            ecc_p384_key_pair: Arc::new(ecc_p384_key_pair),
            rng,
        })
    }
    
    /// Create a test RSA key for demonstration
    /// Uses rsa crate for key generation and converts to ring format
    fn create_test_rsa_key() -> Result<RsaKeyPair, String> {
        use rsa::rand_core::OsRng;
        
        // Generate RSA private key using rsa crate
        let mut rng = OsRng;
        let private_key = RsaPrivateKey::new(&mut rng, 2048)
            .map_err(|e| format!("Failed to generate RSA key: {}", e))?;
        
        // Convert to PKCS#8 DER format for ring
        let private_key_der = private_key.to_pkcs8_der()
            .map_err(|e| format!("Failed to encode RSA key: {}", e))?;
        
        // Create ring RsaKeyPair from DER bytes
        RsaKeyPair::from_pkcs8(private_key_der.as_bytes())
            .map_err(|e| format!("Failed to create ring RSA key pair: {}", e))
    }
    
    /// Get RSA public key in DER format
    pub fn get_rsa_public_key_der(&self) -> AppResult<Vec<u8>> {
        match &self.rsa_key_pair {
            Some(key_pair) => {
                let public_key = key_pair.public();
                Ok(public_key.as_ref().to_vec())
            },
            None => Err(AppError::CryptoError("RSA key pair not available".to_string())),
        }
    }
    
    /// Get ECC P-256 public key in DER format
    pub fn get_ecc_p256_public_key_der(&self) -> AppResult<Vec<u8>> {
        let public_key = self.ecc_p256_key_pair.public_key();
        Ok(public_key.as_ref().to_vec())
    }
    
    /// Get ECC P-384 public key in DER format
    pub fn get_ecc_p384_public_key_der(&self) -> AppResult<Vec<u8>> {
        let public_key = self.ecc_p384_key_pair.public_key();
        Ok(public_key.as_ref().to_vec())
    }
    
    /// Sign data using RSA PKCS#1 v1.5 with SHA-256
    pub fn sign_rsa_pkcs1_sha256(&self, data: &[u8]) -> AppResult<Vec<u8>> {
        match &self.rsa_key_pair {
            Some(key_pair) => {
                let mut signature = vec![0u8; key_pair.public().modulus_len()];
                key_pair
                    .sign(&signature::RSA_PKCS1_SHA256, &self.rng, data, &mut signature)
                    .map_err(|e| AppError::CryptoError(format!("RSA PKCS#1 signing failed: {:?}", e)))?;
                Ok(signature)
            },
            None => Err(AppError::CryptoError("RSA key pair not available".to_string())),
        }
    }
    
    /// Sign data using RSA PSS with SHA-256
    pub fn sign_rsa_pss_sha256(&self, data: &[u8]) -> AppResult<Vec<u8>> {
        match &self.rsa_key_pair {
            Some(key_pair) => {
                let mut signature = vec![0u8; key_pair.public().modulus_len()];
                key_pair
                    .sign(&signature::RSA_PSS_SHA256, &self.rng, data, &mut signature)
                    .map_err(|e| AppError::CryptoError(format!("RSA PSS signing failed: {:?}", e)))?;
                Ok(signature)
            },
            None => Err(AppError::CryptoError("RSA key pair not available".to_string())),
        }
    }
    
    /// Sign data using ECDSA P-256 with SHA-256
    pub fn sign_ecdsa_p256_sha256(&self, data: &[u8]) -> AppResult<Vec<u8>> {
        let signature = self.ecc_p256_key_pair
            .sign(&self.rng, data)
            .map_err(|e| AppError::CryptoError(format!("ECDSA P-256 signing failed: {:?}", e)))?;
        Ok(signature.as_ref().to_vec())
    }
    
    /// Sign data using ECDSA P-384 with SHA-384
    pub fn sign_ecdsa_p384_sha384(&self, data: &[u8]) -> AppResult<Vec<u8>> {
        let signature = self.ecc_p384_key_pair
            .sign(&self.rng, data)
            .map_err(|e| AppError::CryptoError(format!("ECDSA P-384 signing failed: {:?}", e)))?;
        Ok(signature.as_ref().to_vec())
    }
}

// Implement Clone for CryptoKeys by cloning the Arc references
impl Clone for CryptoKeys {
    fn clone(&self) -> Self {
        CryptoKeys {
            rsa_key_pair: self.rsa_key_pair.clone(),
            ecc_p256_key_pair: Arc::clone(&self.ecc_p256_key_pair),
            ecc_p384_key_pair: Arc::clone(&self.ecc_p384_key_pair),
            rng: ring::rand::SystemRandom::new(),
        }
    }
}