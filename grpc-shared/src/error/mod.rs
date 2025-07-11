//! Error handling for the gRPC client/server system
//!
//! This module provides comprehensive error types and conversion traits
//! for all operations in the system, following the PRD requirements
//! for never panicking and proper error propagation.

pub mod types;

pub use types::{Error, Result};

// Re-export common error types for convenience
pub use types::{
    ConfigError, CryptoError, NetworkError, ResourceError, TransportError, ValidationError,
};