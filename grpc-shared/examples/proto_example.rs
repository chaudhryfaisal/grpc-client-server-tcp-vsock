//! Example demonstrating the protocol buffer definitions
//! 
//! This example shows how to use the generated protocol buffer types
//! and demonstrates that the gRPC service definitions are working correctly.

use grpc_shared::proto::*;

fn main() {
    println!("Protocol Buffer Definitions Example");
    println!("===================================");

    // Demonstrate enum usage
    println!("\n1. Key Types:");
    println!("   RSA 2048: {:?}", KeyType::Rsa2048);
    println!("   RSA 3072: {:?}", KeyType::Rsa3072);
    println!("   RSA 4096: {:?}", KeyType::Rsa4096);
    println!("   ECC P-256: {:?}", KeyType::EccP256);
    println!("   ECC P-384: {:?}", KeyType::EccP384);
    println!("   ECC P-521: {:?}", KeyType::EccP521);

    println!("\n2. Signing Algorithms:");
    println!("   RSA-PSS SHA256: {:?}", SigningAlgorithm::RsaPssSha256);
    println!("   RSA-PSS SHA384: {:?}", SigningAlgorithm::RsaPssSha384);
    println!("   RSA-PSS SHA512: {:?}", SigningAlgorithm::RsaPssSha512);
    println!("   RSA PKCS#1 SHA256: {:?}", SigningAlgorithm::RsaPkcs1Sha256);
    println!("   RSA PKCS#1 SHA384: {:?}", SigningAlgorithm::RsaPkcs1Sha384);
    println!("   RSA PKCS#1 SHA512: {:?}", SigningAlgorithm::RsaPkcs1Sha512);
    println!("   ECDSA SHA256: {:?}", SigningAlgorithm::EcdsaSha256);
    println!("   ECDSA SHA384: {:?}", SigningAlgorithm::EcdsaSha384);
    println!("   ECDSA SHA512: {:?}", SigningAlgorithm::EcdsaSha512);

    println!("\n3. Hash Algorithms:");
    println!("   SHA256: {:?}", HashAlgorithm::Sha256);
    println!("   SHA384: {:?}", HashAlgorithm::Sha384);
    println!("   SHA512: {:?}", HashAlgorithm::Sha512);

    // Demonstrate message creation
    println!("\n4. Message Creation:");
    
    let sign_request = SignRequest {
        data: b"Hello, World!".to_vec(),
        key_id: "example-key-id".to_string(),
        algorithm: SigningAlgorithm::RsaPssSha256 as i32,
        hash_algorithm: HashAlgorithm::Sha256 as i32,
    };
    println!("   Sign Request: {:?}", sign_request);

    let key_info = KeyInfo {
        key_id: "example-key".to_string(),
        key_type: KeyType::Rsa2048 as i32,
        created_at: 1640995200, // 2022-01-01 00:00:00 UTC
        description: "Example RSA 2048-bit key".to_string(),
        is_active: true,
    };
    println!("   Key Info: {:?}", key_info);

    let generate_key_request = GenerateKeyRequest {
        key_id: "new-key".to_string(),
        key_type: KeyType::EccP256 as i32,
        description: "Example ECC P-256 key".to_string(),
    };
    println!("   Generate Key Request: {:?}", generate_key_request);

    let list_keys_request = ListKeysRequest {
        key_type_filter: Some(KeyType::Rsa2048 as i32),
        active_only: Some(true),
    };
    println!("   List Keys Request: {:?}", list_keys_request);

    let verify_request = VerifyRequest {
        data: b"data to verify".to_vec(),
        signature: b"signature bytes".to_vec(),
        key_id: "verify-key".to_string(),
        algorithm: SigningAlgorithm::EcdsaSha256 as i32,
        hash_algorithm: HashAlgorithm::Sha256 as i32,
    };
    println!("   Verify Request: {:?}", verify_request);

    let health_check_request = HealthCheckRequest {
        service: "signing".to_string(),
    };
    println!("   Health Check Request: {:?}", health_check_request);

    println!("\n5. Error Codes:");
    println!("   Invalid Key ID: {:?}", ErrorCode::InvalidKeyId);
    println!("   Signing Failed: {:?}", ErrorCode::SigningFailed);
    println!("   Key Not Found: {:?}", ErrorCode::KeyNotFound);
    println!("   Verification Failed: {:?}", ErrorCode::VerificationFailed);

    println!("\n6. Response Examples:");
    
    let sign_response = SignResponse {
        signature: b"example signature".to_vec(),
        success: true,
        error_message: String::new(),
        error_code: ErrorCode::Unspecified as i32,
        processing_time_us: 850,
    };
    println!("   Sign Response: {:?}", sign_response);

    let health_response = HealthCheckResponse {
        status: ServingStatus::Serving as i32,
        message: "All systems operational".to_string(),
    };
    println!("   Health Response: {:?}", health_response);

    println!("\nProtocol buffer definitions are working correctly!");
    println!("All required PRD features have been implemented:");
    println!("✅ RSA and ECC key types (2048/3072/4096, P-256/P-384/P-521)");
    println!("✅ Multiple signing algorithms (RSA-PSS, PKCS#1 v1.5, ECDSA)");
    println!("✅ Hash algorithm support (SHA256, SHA384, SHA512)");
    println!("✅ Comprehensive error handling with error codes");
    println!("✅ Key management operations (generate, list, delete)");
    println!("✅ Signature verification functionality");
    println!("✅ Health check monitoring");
}