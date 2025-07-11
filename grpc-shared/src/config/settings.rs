//! Configuration structures for the gRPC client/server system
//!
//! Implements the configuration schema from PRD sections 10.1 and 10.2

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::Duration;

/// Transport type enumeration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum TransportType {
    /// TCP transport
    Tcp,
    /// VSOCK transport (Unix only)
    #[cfg(unix)]
    Vsock,
}

/// Server configuration as specified in PRD section 10.1
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    /// Address to bind the server to
    pub bind_address: String,
    /// Port to bind the server to
    pub port: u16,
    /// Transport type to use
    pub transport: TransportType,
    /// Optional TLS configuration
    pub tls: Option<TlsConfig>,
    /// Cryptographic configuration
    pub crypto: CryptoConfig,
    /// Logging configuration
    pub logging: LoggingConfig,
    /// Performance configuration
    pub performance: PerformanceConfig,
}

/// Client configuration as specified in PRD section 10.2
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientConfig {
    /// Server address to connect to
    pub server_address: String,
    /// Transport type to use
    pub transport: TransportType,
    /// Optional TLS configuration
    pub tls: Option<TlsConfig>,
    /// Client-specific cryptographic configuration
    pub crypto: ClientCryptoConfig,
    /// Connection pool configuration
    pub connection_pool: ConnectionPoolConfig,
    /// Retry configuration
    pub retry: RetryConfig,
}

/// TLS/MTLS configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TlsConfig {
    /// Enable TLS
    pub enabled: bool,
    /// Path to certificate file
    pub cert_path: Option<PathBuf>,
    /// Path to private key file
    pub key_path: Option<PathBuf>,
    /// Path to CA certificate file for client verification
    pub ca_cert_path: Option<PathBuf>,
    /// Require client certificates (MTLS)
    pub require_client_cert: bool,
    /// Allowed cipher suites
    pub cipher_suites: Vec<String>,
    /// Minimum TLS version
    pub min_tls_version: String,
}

/// Cryptographic configuration for server
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CryptoConfig {
    /// Default key type to use
    pub default_key_type: KeyType,
    /// Key generation settings
    pub key_generation: KeyGenerationConfig,
    /// Key loading settings
    pub key_loading: KeyLoadingConfig,
    /// Supported algorithms
    pub supported_algorithms: Vec<SigningAlgorithm>,
}

/// Client-specific cryptographic configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientCryptoConfig {
    /// Preferred key type
    pub preferred_key_type: KeyType,
    /// Preferred signing algorithm
    pub preferred_algorithm: SigningAlgorithm,
    /// Algorithm preferences in order
    pub algorithm_preferences: Vec<SigningAlgorithm>,
}

/// Key generation configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyGenerationConfig {
    /// Generate keys at startup
    pub generate_at_startup: bool,
    /// Key types to generate
    pub key_types: Vec<KeyType>,
    /// Key storage directory
    pub storage_dir: Option<PathBuf>,
}

/// Key loading configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyLoadingConfig {
    /// Directory containing key files
    pub key_dir: Option<PathBuf>,
    /// Specific key file paths
    pub key_files: Vec<KeyFileConfig>,
}

/// Individual key file configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyFileConfig {
    /// Key identifier
    pub key_id: String,
    /// Key type
    pub key_type: KeyType,
    /// Path to private key file
    pub private_key_path: PathBuf,
    /// Path to public key file (optional)
    pub public_key_path: Option<PathBuf>,
}

/// Key type enumeration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum KeyType {
    /// RSA 2048-bit key
    Rsa2048,
    /// RSA 3072-bit key
    Rsa3072,
    /// RSA 4096-bit key
    Rsa4096,
    /// ECC P-256 key
    EccP256,
    /// ECC P-384 key
    EccP384,
    /// ECC P-521 key
    EccP521,
}

/// Signing algorithm enumeration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum SigningAlgorithm {
    /// RSA PSS with SHA-256
    RsaPssSha256,
    /// RSA PSS with SHA-384
    RsaPssSha384,
    /// RSA PSS with SHA-512
    RsaPssSha512,
    /// RSA PKCS#1 v1.5 with SHA-256
    RsaPkcs1v15Sha256,
    /// RSA PKCS#1 v1.5 with SHA-384
    RsaPkcs1v15Sha384,
    /// RSA PKCS#1 v1.5 with SHA-512
    RsaPkcs1v15Sha512,
    /// ECDSA P-256 with SHA-256
    EcdsaP256Sha256,
    /// ECDSA P-384 with SHA-384
    EcdsaP384Sha384,
    /// ECDSA P-521 with SHA-512
    EcdsaP521Sha512,
}

/// Logging configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    /// Log level
    pub level: String,
    /// Enable structured logging
    pub structured: bool,
    /// Log file path (optional)
    pub file_path: Option<PathBuf>,
    /// Enable request/response logging
    pub log_requests: bool,
    /// Enable performance metrics logging
    pub log_performance: bool,
}

/// Performance configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceConfig {
    /// Maximum number of concurrent connections
    pub max_connections: u32,
    /// Worker thread count
    pub worker_threads: Option<u32>,
    /// Connection timeout
    pub connection_timeout: Duration,
    /// Request timeout
    pub request_timeout: Duration,
    /// Keep-alive settings
    pub keep_alive: KeepAliveConfig,
}

/// Keep-alive configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeepAliveConfig {
    /// Enable keep-alive
    pub enabled: bool,
    /// Keep-alive interval
    pub interval: Duration,
    /// Keep-alive timeout
    pub timeout: Duration,
}

/// Connection pool configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionPoolConfig {
    /// Maximum pool size
    pub max_size: u32,
    /// Minimum pool size
    pub min_size: u32,
    /// Connection idle timeout
    pub idle_timeout: Duration,
    /// Maximum connection lifetime
    pub max_lifetime: Duration,
}

/// Retry configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryConfig {
    /// Enable retries
    pub enabled: bool,
    /// Maximum retry attempts
    pub max_attempts: u32,
    /// Initial retry delay
    pub initial_delay: Duration,
    /// Maximum retry delay
    pub max_delay: Duration,
    /// Backoff multiplier
    pub backoff_multiplier: f64,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            bind_address: "127.0.0.1".to_string(),
            port: 50051,
            transport: TransportType::Tcp,
            tls: None,
            crypto: CryptoConfig::default(),
            logging: LoggingConfig::default(),
            performance: PerformanceConfig::default(),
        }
    }
}

impl Default for ClientConfig {
    fn default() -> Self {
        Self {
            server_address: "127.0.0.1:50051".to_string(),
            transport: TransportType::Tcp,
            tls: None,
            crypto: ClientCryptoConfig::default(),
            connection_pool: ConnectionPoolConfig::default(),
            retry: RetryConfig::default(),
        }
    }
}

impl Default for CryptoConfig {
    fn default() -> Self {
        Self {
            default_key_type: KeyType::EccP256,
            key_generation: KeyGenerationConfig::default(),
            key_loading: KeyLoadingConfig::default(),
            supported_algorithms: vec![
                SigningAlgorithm::EcdsaP256Sha256,
                SigningAlgorithm::RsaPssSha256,
            ],
        }
    }
}

impl Default for ClientCryptoConfig {
    fn default() -> Self {
        Self {
            preferred_key_type: KeyType::EccP256,
            preferred_algorithm: SigningAlgorithm::EcdsaP256Sha256,
            algorithm_preferences: vec![
                SigningAlgorithm::EcdsaP256Sha256,
                SigningAlgorithm::RsaPssSha256,
            ],
        }
    }
}

impl Default for KeyGenerationConfig {
    fn default() -> Self {
        Self {
            generate_at_startup: true,
            key_types: vec![KeyType::EccP256],
            storage_dir: None,
        }
    }
}

impl Default for KeyLoadingConfig {
    fn default() -> Self {
        Self {
            key_dir: None,
            key_files: Vec::new(),
        }
    }
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: "info".to_string(),
            structured: true,
            file_path: None,
            log_requests: false,
            log_performance: true,
        }
    }
}

impl Default for PerformanceConfig {
    fn default() -> Self {
        Self {
            max_connections: 1000,
            worker_threads: None,
            connection_timeout: Duration::from_secs(10),
            request_timeout: Duration::from_secs(30),
            keep_alive: KeepAliveConfig::default(),
        }
    }
}

impl Default for KeepAliveConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            interval: Duration::from_secs(60),
            timeout: Duration::from_secs(5),
        }
    }
}

impl Default for ConnectionPoolConfig {
    fn default() -> Self {
        Self {
            max_size: 10,
            min_size: 1,
            idle_timeout: Duration::from_secs(300),
            max_lifetime: Duration::from_secs(3600),
        }
    }
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            max_attempts: 3,
            initial_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(5),
            backoff_multiplier: 2.0,
        }
    }
}