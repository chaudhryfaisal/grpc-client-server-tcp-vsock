//! Configuration management for the gRPC client/server system
//!
//! This module provides configuration structures and loading functionality
//! as specified in PRD sections 10.1 and 10.2.

pub mod settings;

pub use settings::{
    ClientConfig, ClientCryptoConfig, ConnectionPoolConfig, CryptoConfig,
    KeyGenerationConfig, KeyLoadingConfig, KeyType, LoggingConfig,
    PerformanceConfig, RetryConfig, ServerConfig, SigningAlgorithm,
    TlsConfig, TransportType,
};