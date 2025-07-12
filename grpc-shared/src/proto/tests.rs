//! Tests for protocol buffer definitions

#[cfg(test)]
mod tests {
    use crate::proto::signing::*;

    #[test]
    fn test_key_type_enum() {
        // Test that all key types are properly defined
        assert_eq!(KeyType::Unspecified as i32, 0);
        assert_eq!(KeyType::Rsa2048 as i32, 1);
        assert_eq!(KeyType::Rsa3072 as i32, 2);
        assert_eq!(KeyType::Rsa4096 as i32, 3);
        assert_eq!(KeyType::EccP256 as i32, 4);
        assert_eq!(KeyType::EccP384 as i32, 5);
        assert_eq!(KeyType::EccP521 as i32, 6);
    }

    #[test]
    fn test_signing_algorithm_enum() {
        // Test that all signing algorithms are properly defined
        assert_eq!(SigningAlgorithm::Unspecified as i32, 0);
        assert_eq!(SigningAlgorithm::RsaPssSha256 as i32, 1);
        assert_eq!(SigningAlgorithm::RsaPssSha384 as i32, 2);
        assert_eq!(SigningAlgorithm::RsaPssSha512 as i32, 3);
        assert_eq!(SigningAlgorithm::RsaPkcs1Sha256 as i32, 4);
        assert_eq!(SigningAlgorithm::RsaPkcs1Sha384 as i32, 5);
        assert_eq!(SigningAlgorithm::RsaPkcs1Sha512 as i32, 6);
        assert_eq!(SigningAlgorithm::EcdsaSha256 as i32, 7);
        assert_eq!(SigningAlgorithm::EcdsaSha384 as i32, 8);
        assert_eq!(SigningAlgorithm::EcdsaSha512 as i32, 9);
    }

    #[test]
    fn test_hash_algorithm_enum() {
        // Test that all hash algorithms are properly defined
        assert_eq!(HashAlgorithm::Unspecified as i32, 0);
        assert_eq!(HashAlgorithm::Sha256 as i32, 1);
        assert_eq!(HashAlgorithm::Sha384 as i32, 2);
        assert_eq!(HashAlgorithm::Sha512 as i32, 3);
    }

    #[test]
    fn test_error_code_enum() {
        // Test that all error codes are properly defined
        assert_eq!(ErrorCode::Unspecified as i32, 0);
        assert_eq!(ErrorCode::InvalidKeyId as i32, 1);
        assert_eq!(ErrorCode::InvalidAlgorithm as i32, 2);
        assert_eq!(ErrorCode::InvalidData as i32, 3);
        assert_eq!(ErrorCode::KeyGenerationFailed as i32, 4);
        assert_eq!(ErrorCode::SigningFailed as i32, 5);
        assert_eq!(ErrorCode::VerificationFailed as i32, 6);
        assert_eq!(ErrorCode::KeyNotFound as i32, 7);
        assert_eq!(ErrorCode::KeyAlreadyExists as i32, 8);
        assert_eq!(ErrorCode::InternalError as i32, 9);
        assert_eq!(ErrorCode::InvalidSignature as i32, 10);
    }

    #[test]
    fn test_sign_request_creation() {
        let request = SignRequest {
            data: b"test data".to_vec(),
            key_id: "test-key".to_string(),
            algorithm: SigningAlgorithm::RsaPssSha256 as i32,
            key_type: KeyType::Rsa2048 as i32,
        };

        assert_eq!(request.data, b"test data");
        assert_eq!(request.key_id, "test-key");
        assert_eq!(request.algorithm, SigningAlgorithm::RsaPssSha256 as i32);
        assert_eq!(request.key_type, KeyType::Rsa2048 as i32);
    }

    #[test]
    fn test_key_info_creation() {
        let key_info = KeyInfo {
            key_id: "test-key".to_string(),
            key_type: KeyType::Rsa2048 as i32,
            created_at: 1234567890,
            description: "Test key".to_string(),
            is_active: true,
        };

        assert_eq!(key_info.key_id, "test-key");
        assert_eq!(key_info.key_type, KeyType::Rsa2048 as i32);
        assert_eq!(key_info.created_at, 1234567890);
        assert_eq!(key_info.description, "Test key");
        assert!(key_info.is_active);
    }

    #[test]
    fn test_generate_key_request() {
        let request = GenerateKeyRequest {
            key_id: "new-key".to_string(),
            key_type: KeyType::EccP256 as i32,
            description: "Test ECC key".to_string(),
        };

        assert_eq!(request.key_id, "new-key");
        assert_eq!(request.key_type, KeyType::EccP256 as i32);
        assert_eq!(request.description, "Test ECC key");
    }

    #[test]
    fn test_list_keys_request() {
        let request = ListKeysRequest {
            key_type_filter: Some(KeyType::Rsa2048 as i32),
            active_only: Some(true),
        };

        assert_eq!(request.key_type_filter, Some(KeyType::Rsa2048 as i32));
        assert_eq!(request.active_only, Some(true));
    }

    #[test]
    fn test_verify_request() {
        let request = VerifyRequest {
            data: b"test data".to_vec(),
            signature: b"test signature".to_vec(),
            key_id: "test-key".to_string(),
            algorithm: SigningAlgorithm::EcdsaSha256 as i32,
            hash_algorithm: HashAlgorithm::Sha256 as i32,
        };

        assert_eq!(request.data, b"test data");
        assert_eq!(request.signature, b"test signature");
        assert_eq!(request.key_id, "test-key");
        assert_eq!(request.algorithm, SigningAlgorithm::EcdsaSha256 as i32);
        assert_eq!(request.hash_algorithm, HashAlgorithm::Sha256 as i32);
    }

    #[test]
    fn test_health_check_request() {
        let request = HealthCheckRequest {
            service: "signing".to_string(),
        };

        assert_eq!(request.service, "signing");
    }

    #[test]
    fn test_health_check_response() {
        let response = HealthCheckResponse {
            status: health_check_response::ServingStatus::Serving as i32,
            message: "Service is healthy".to_string(),
        };

        assert_eq!(response.status, health_check_response::ServingStatus::Serving as i32);
        assert_eq!(response.message, "Service is healthy");
    }

    #[test]
    fn test_sign_response_with_error() {
        let response = SignResponse {
            signature: vec![],
            success: false,
            error_message: "Key not found".to_string(),
            error_code: ErrorCode::KeyNotFound as i32,
            processing_time_us: 1500,
        };

        assert!(!response.success);
        assert_eq!(response.error_message, "Key not found");
        assert_eq!(response.error_code, ErrorCode::KeyNotFound as i32);
        assert_eq!(response.processing_time_us, 1500);
    }

    #[test]
    fn test_verify_response() {
        let response = VerifyResponse {
            valid: true,
            success: true,
            error_message: String::new(),
            error_code: ErrorCode::Unspecified as i32,
        };

        assert!(response.valid);
        assert!(response.success);
        assert!(response.error_message.is_empty());
        assert_eq!(response.error_code, ErrorCode::Unspecified as i32);
    }
}