//! Transport layer implementations for TCP and VSOCK
//!
//! This module provides transport abstractions as specified in PRD Phase 2: Core Transport Layer

pub mod tcp;
#[cfg(unix)]
pub mod vsock;

use crate::error::Result;
use async_trait::async_trait;

// Re-export TransportType for convenience
pub use crate::config::TransportType;

/// Transport trait for different transport implementations
#[async_trait]
pub trait Transport: Send + Sync {
    /// Connect to the specified address
    async fn connect(&self, address: &str) -> Result<Box<dyn Connection>>;

    /// Bind to the specified address for server
    async fn bind(&self, address: &str) -> Result<Box<dyn Listener>>;

    /// Get the transport type
    fn transport_type(&self) -> TransportType;
}

/// Connection trait for active connections
#[async_trait]
pub trait Connection: Send + Sync {
    /// Read data from the connection
    async fn read(&mut self, buf: &mut [u8]) -> Result<usize>;

    /// Write data to the connection
    async fn write(&mut self, buf: &[u8]) -> Result<usize>;

    /// Close the connection
    async fn close(&mut self) -> Result<()>;
}

/// Listener trait for accepting connections
#[async_trait]
pub trait Listener: Send + Sync {
    /// Accept a new connection
    async fn accept(&mut self) -> Result<Box<dyn Connection>>;

    /// Close the listener
    async fn close(&mut self) -> Result<()>;
}

/// Create transport based on type
pub fn create_transport(transport_type: TransportType) -> Result<Box<dyn Transport>> {
    match transport_type {
        TransportType::Tcp => Ok(Box::new(tcp::TcpTransport::new())),
        #[cfg(all(unix, feature = "vsock"))]
        TransportType::Vsock => Ok(Box::new(vsock::VsockTransport::new())),
        #[cfg(not(all(unix, feature = "vsock")))]
        TransportType::Vsock => {
            Err(crate::error::TransportError::UnsupportedType {
                transport_type: "VSOCK (not available - enable 'vsock' feature and compile on Unix)".to_string(),
            }.into())
        }
    }
}