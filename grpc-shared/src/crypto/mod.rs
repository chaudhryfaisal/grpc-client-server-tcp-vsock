//! Cryptographic utilities for the gRPC client/server system
//!
//! This module provides cryptographic operations as specified in PRD sections 2.2 and 8-11:
//! - RSA and ECC key generation and loading
//! - Cryptographic signing operations
//! - Algorithm selection and key management

pub mod keys;
pub mod signing;

pub use keys::{KeyManager, KeyPair};
pub use signing::{RingSigner, Signer, SigningOperation};

// Re-export configuration types
pub use crate::config::{KeyType, SigningAlgorithm};