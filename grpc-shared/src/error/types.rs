//! Error types for the gRPC client/server system
//!
//! Defines all error categories as specified in PRD section 7.1:
//! - Network Errors: Connection failures, timeouts, transport errors
//! - Cryptographic Errors: Key failures, signing errors, validation errors
//! - Configuration Errors: Invalid settings, missing files, permission errors
//! - Resource Errors: Memory exhaustion, file descriptor limits

use thiserror::Error;

/// Main error type for the gRPC system
#[derive(Error, Debug)]
pub enum Error {
    /// Network-related errors
    #[error("Network error: {0}")]
    Network(#[from] NetworkError),

    /// Cryptographic operation errors
    #[error("Cryptographic error: {0}")]
    Crypto(#[from] CryptoError),

    /// Configuration-related errors
    #[error("Configuration error: {0}")]
    Config(#[from] ConfigError),

    /// Resource-related errors
    #[error("Resource error: {0}")]
    Resource(#[from] ResourceError),

    /// Transport-specific errors
    #[error("Transport error: {0}")]
    Transport(#[from] TransportError),

    /// gRPC-specific errors
    #[error("gRPC error: {0}")]
    Grpc(#[from] tonic::Status),

    /// Generic I/O errors
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}

/// Network-related errors
#[derive(Error, Debug)]
pub enum NetworkError {
    /// Connection failed to establish
    #[error("Connection failed: {message}")]
    ConnectionFailed { message: String },

    /// Connection timeout
    #[error("Connection timeout after {timeout_ms}ms")]
    ConnectionTimeout { timeout_ms: u64 },

    /// Connection was lost
    #[error("Connection lost: {reason}")]
    ConnectionLost { reason: String },

    /// DNS resolution failed
    #[error("DNS resolution failed for {hostname}: {error}")]
    DnsResolution { hostname: String, error: String },

    /// Invalid address format
    #[error("Invalid address: {address}")]
    InvalidAddress { address: String },
}

/// Cryptographic operation errors
#[derive(Error, Debug)]
pub enum CryptoError {
    /// Key generation failed
    #[error("Key generation failed: {reason}")]
    KeyGeneration { reason: String },

    /// Key loading failed
    #[error("Key loading failed from {path}: {reason}")]
    KeyLoading { path: String, reason: String },

    /// Invalid key format
    #[error("Invalid key format: {reason}")]
    InvalidKeyFormat { reason: String },

    /// Signing operation failed
    #[error("Signing failed with {algorithm}: {reason}")]
    SigningFailed { algorithm: String, reason: String },

    /// Signature verification failed
    #[error("Signature verification failed: {reason}")]
    VerificationFailed { reason: String },

    /// Unsupported algorithm
    #[error("Unsupported algorithm: {algorithm}")]
    UnsupportedAlgorithm { algorithm: String },

    /// Ring cryptography library error
    #[error("Ring error")]
    Ring,
}

/// Configuration-related errors
#[derive(Error, Debug)]
pub enum ConfigError {
    /// Configuration file not found
    #[error("Configuration file not found: {path}")]
    FileNotFound { path: String },

    /// Invalid configuration format
    #[error("Invalid configuration format in {path}: {reason}")]
    InvalidFormat { path: String, reason: String },

    /// Missing required configuration field
    #[error("Missing required configuration field: {field}")]
    MissingField { field: String },

    /// Invalid configuration value
    #[error("Invalid configuration value for {field}: {value}")]
    InvalidValue { field: String, value: String },

    /// Permission denied accessing configuration
    #[error("Permission denied accessing {path}")]
    PermissionDenied { path: String },

    /// Configuration parsing error
    #[error("Configuration parsing error: {0}")]
    Parsing(#[from] config::ConfigError),
}

/// Resource-related errors
#[derive(Error, Debug)]
pub enum ResourceError {
    /// Memory allocation failed
    #[error("Memory allocation failed: {size} bytes")]
    MemoryAllocation { size: usize },

    /// File descriptor limit reached
    #[error("File descriptor limit reached: {limit}")]
    FileDescriptorLimit { limit: u32 },

    /// Thread pool exhausted
    #[error("Thread pool exhausted: {active_threads}/{max_threads}")]
    ThreadPoolExhausted { active_threads: u32, max_threads: u32 },

    /// Connection pool exhausted
    #[error("Connection pool exhausted: {active_connections}/{max_connections}")]
    ConnectionPoolExhausted { active_connections: u32, max_connections: u32 },

    /// Disk space insufficient
    #[error("Insufficient disk space: {available} bytes available, {required} bytes required")]
    InsufficientDiskSpace { available: u64, required: u64 },
}

/// Transport-specific errors
#[derive(Error, Debug)]
pub enum TransportError {
    /// TCP transport error
    #[error("TCP transport error: {message}")]
    Tcp { message: String },

    /// VSOCK transport error
    #[error("VSOCK transport error: {message}")]
    Vsock { message: String },

    /// TLS/MTLS error
    #[error("TLS error: {message}")]
    Tls { message: String },

    /// Unsupported transport type
    #[error("Unsupported transport type: {transport_type}")]
    UnsupportedType { transport_type: String },

    /// Transport configuration error
    #[error("Transport configuration error: {message}")]
    Configuration { message: String },
}

/// Result type alias for the gRPC system
pub type Result<T> = std::result::Result<T, Error>;

// Implement conversion from common error types
impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Self {
        Error::Config(ConfigError::InvalidFormat {
            path: "JSON".to_string(),
            reason: err.to_string(),
        })
    }
}

impl From<tokio::time::error::Elapsed> for Error {
    fn from(_: tokio::time::error::Elapsed) -> Self {
        Error::Network(NetworkError::ConnectionTimeout { timeout_ms: 0 })
    }
}