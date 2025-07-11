//! High-Performance gRPC Client/Server with Cryptographic Operations
//!
//! This library provides a production-grade gRPC client and server implementation
//! in Rust that supports both TCP and VSOCK transports, provides cryptographic
//! signing capabilities, and achieves maximum performance with minimal latency.

#![deny(unsafe_code)]
#![warn(clippy::all)]
#![warn(missing_docs)]

// Core modules
pub mod config;
pub mod crypto;
pub mod error;
pub mod proto;
pub mod transport;

// Implementation modules
pub mod client;
pub mod server;

// Optional benchmarking module
#[cfg(feature = "benchmarks")]
pub mod benchmarks;

// Re-export commonly used types for convenience
pub use config::{ClientConfig, ServerConfig};
pub use error::{Error, Result};
pub use transport::{Transport, TransportType};

// Re-export crypto types
pub use crypto::{KeyType, SigningAlgorithm};

// Re-export generated protobuf types
pub use proto::signing::{
    signing_service_client::SigningServiceClient,
    signing_service_server::{SigningService, SigningServiceServer},
    SignRequest, SignResponse,
};