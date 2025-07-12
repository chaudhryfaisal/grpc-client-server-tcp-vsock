//! Protocol Buffer definitions and generated code for the signing service.
//! 
//! This module contains all the generated gRPC service definitions, message types,
//! and enums for the signing service protocol.

// Include the generated protobuf code
pub mod signing {
    tonic::include_proto!("signing");
}

// Re-export commonly used types for convenience
pub use signing::{
    // Service client and server
    signing_service_client::SigningServiceClient,
    signing_service_server::{SigningService, SigningServiceServer},
    
    // Request/Response types
    SignRequest, SignResponse,
    GenerateKeyRequest, GenerateKeyResponse,
    ListKeysRequest, ListKeysResponse,
    DeleteKeyRequest, DeleteKeyResponse,
    HealthCheckRequest, HealthCheckResponse,
    VerifyRequest, VerifyResponse,
    
    // Data types
    KeyInfo,
    
    // Enums
    KeyType, SigningAlgorithm, HashAlgorithm, ErrorCode,
};

// Additional re-export for easier access
pub use signing::health_check_response::ServingStatus;

// Type aliases for convenience
pub type SigningClient = SigningServiceClient<tonic::transport::Channel>;

#[cfg(feature = "server")]
pub type SigningServer = SigningServiceServer<crate::server::GrpcSigningServer>;

// Include tests
#[cfg(test)]
mod tests;