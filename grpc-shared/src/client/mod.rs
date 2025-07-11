//! Client implementation for the gRPC signing service
//!
//! This module provides client-side functionality as specified in PRD Phase 5: Client Implementation

pub mod connection;
pub mod grpc_client;

pub use grpc_client::GrpcSigningClient;
pub use connection::ClientConnection;