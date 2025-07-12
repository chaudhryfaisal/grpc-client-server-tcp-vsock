// Crypto module with proper test integration
pub mod keys;
pub mod signing;

#[cfg(test)]
mod tests;

// Re-export commonly used types
pub use keys::{KeyPair, KeyManager};
pub use signing::{RingSigner, SigningOperation, Signer};