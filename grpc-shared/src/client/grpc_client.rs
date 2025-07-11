//! gRPC client implementation for the signing service
//!
//! This module implements the gRPC client as specified in PRD Task 16: Basic gRPC Client

use crate::config::ClientConfig;
use crate::error::Result;

/// gRPC signing client implementation
#[derive(Debug)]
pub struct GrpcSigningClient {
    config: ClientConfig,
}

impl GrpcSigningClient {
    /// Create a new gRPC signing client
    pub fn new(config: ClientConfig) -> Self {
        Self { config }
    }

    /// Connect to the server
    pub async fn connect(&self) -> Result<()> {
        // TODO: Implement gRPC client connection
        log::info!("Connecting to gRPC signing server at {}", self.config.server_address);
        Ok(())
    }

    /// Disconnect from the server
    pub async fn disconnect(&self) -> Result<()> {
        // TODO: Implement graceful disconnection
        log::info!("Disconnecting from gRPC signing server");
        Ok(())
    }
}