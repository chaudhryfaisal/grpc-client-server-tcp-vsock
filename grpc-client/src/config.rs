//! Client configuration module
//!
//! This module defines configuration structures for the gRPC client

pub use grpc_shared::config::ClientConfig;

// Re-export HashAlgorithm from proto module since it's not in config
pub use grpc_shared::proto::signing::HashAlgorithm;