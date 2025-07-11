/// Key information
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct KeyInfo {
    #[prost(string, tag = "1")]
    pub key_id: ::prost::alloc::string::String,
    #[prost(enumeration = "KeyType", tag = "2")]
    pub key_type: i32,
    /// Unix timestamp
    #[prost(int64, tag = "3")]
    pub created_at: i64,
    #[prost(string, tag = "4")]
    pub description: ::prost::alloc::string::String,
    #[prost(bool, tag = "5")]
    pub is_active: bool,
}
/// The request message containing the data to be signed
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SignRequest {
    #[prost(bytes = "vec", tag = "1")]
    pub data: ::prost::alloc::vec::Vec<u8>,
    #[prost(enumeration = "KeyType", tag = "2")]
    pub key_type: i32,
    #[prost(enumeration = "SigningAlgorithm", tag = "3")]
    pub algorithm: i32,
    #[prost(string, tag = "4")]
    pub key_id: ::prost::alloc::string::String,
}
/// The response message containing the signature
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SignResponse {
    #[prost(bytes = "vec", tag = "1")]
    pub signature: ::prost::alloc::vec::Vec<u8>,
    #[prost(bool, tag = "2")]
    pub success: bool,
    #[prost(string, tag = "3")]
    pub error_message: ::prost::alloc::string::String,
    #[prost(enumeration = "ErrorCode", tag = "4")]
    pub error_code: i32,
    #[prost(uint64, tag = "5")]
    pub processing_time_us: u64,
}
/// Request to generate a new key pair
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GenerateKeyRequest {
    #[prost(string, tag = "1")]
    pub key_id: ::prost::alloc::string::String,
    #[prost(enumeration = "KeyType", tag = "2")]
    pub key_type: i32,
    #[prost(string, tag = "3")]
    pub description: ::prost::alloc::string::String,
}
/// Response for key generation
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GenerateKeyResponse {
    #[prost(bool, tag = "1")]
    pub success: bool,
    #[prost(string, tag = "2")]
    pub error_message: ::prost::alloc::string::String,
    #[prost(enumeration = "ErrorCode", tag = "3")]
    pub error_code: i32,
    #[prost(message, optional, tag = "4")]
    pub key_info: ::core::option::Option<KeyInfo>,
}
/// Request to list available keys
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ListKeysRequest {
    /// Optional filter by key type
    #[prost(enumeration = "KeyType", optional, tag = "1")]
    pub key_type_filter: ::core::option::Option<i32>,
    /// Optional filter by active status
    #[prost(bool, optional, tag = "2")]
    pub active_only: ::core::option::Option<bool>,
}
/// Response containing list of keys
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ListKeysResponse {
    #[prost(message, repeated, tag = "1")]
    pub keys: ::prost::alloc::vec::Vec<KeyInfo>,
    #[prost(bool, tag = "2")]
    pub success: bool,
    #[prost(string, tag = "3")]
    pub error_message: ::prost::alloc::string::String,
    #[prost(enumeration = "ErrorCode", tag = "4")]
    pub error_code: i32,
}
/// Request to delete a key
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DeleteKeyRequest {
    #[prost(string, tag = "1")]
    pub key_id: ::prost::alloc::string::String,
}
/// Response for key deletion
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DeleteKeyResponse {
    #[prost(bool, tag = "1")]
    pub success: bool,
    #[prost(string, tag = "2")]
    pub error_message: ::prost::alloc::string::String,
    #[prost(enumeration = "ErrorCode", tag = "3")]
    pub error_code: i32,
}
/// Health check request
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct HealthCheckRequest {
    /// Optional service name to check
    #[prost(string, tag = "1")]
    pub service: ::prost::alloc::string::String,
}
/// Health check response
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct HealthCheckResponse {
    #[prost(enumeration = "health_check_response::ServingStatus", tag = "1")]
    pub status: i32,
    #[prost(string, tag = "2")]
    pub message: ::prost::alloc::string::String,
}
/// Nested message and enum types in `HealthCheckResponse`.
pub mod health_check_response {
    #[derive(
        Clone,
        Copy,
        Debug,
        PartialEq,
        Eq,
        Hash,
        PartialOrd,
        Ord,
        ::prost::Enumeration
    )]
    #[repr(i32)]
    pub enum ServingStatus {
        Unknown = 0,
        Serving = 1,
        NotServing = 2,
        ServiceUnknown = 3,
    }
    impl ServingStatus {
        /// String value of the enum field names used in the ProtoBuf definition.
        ///
        /// The values are not transformed in any way and thus are considered stable
        /// (if the ProtoBuf definition does not change) and safe for programmatic use.
        pub fn as_str_name(&self) -> &'static str {
            match self {
                ServingStatus::Unknown => "UNKNOWN",
                ServingStatus::Serving => "SERVING",
                ServingStatus::NotServing => "NOT_SERVING",
                ServingStatus::ServiceUnknown => "SERVICE_UNKNOWN",
            }
        }
        /// Creates an enum from field names used in the ProtoBuf definition.
        pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
            match value {
                "UNKNOWN" => Some(Self::Unknown),
                "SERVING" => Some(Self::Serving),
                "NOT_SERVING" => Some(Self::NotServing),
                "SERVICE_UNKNOWN" => Some(Self::ServiceUnknown),
                _ => None,
            }
        }
    }
}
/// Request to verify a signature
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct VerifyRequest {
    #[prost(bytes = "vec", tag = "1")]
    pub data: ::prost::alloc::vec::Vec<u8>,
    #[prost(bytes = "vec", tag = "2")]
    pub signature: ::prost::alloc::vec::Vec<u8>,
    #[prost(string, tag = "3")]
    pub key_id: ::prost::alloc::string::String,
    #[prost(enumeration = "SigningAlgorithm", tag = "4")]
    pub algorithm: i32,
    #[prost(enumeration = "HashAlgorithm", tag = "5")]
    pub hash_algorithm: i32,
}
/// Response for signature verification
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct VerifyResponse {
    #[prost(bool, tag = "1")]
    pub valid: bool,
    #[prost(bool, tag = "2")]
    pub success: bool,
    #[prost(string, tag = "3")]
    pub error_message: ::prost::alloc::string::String,
    #[prost(enumeration = "ErrorCode", tag = "4")]
    pub error_code: i32,
}
/// Key types supported
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum KeyType {
    Unspecified = 0,
    Rsa2048 = 1,
    Rsa3072 = 2,
    Rsa4096 = 3,
    EccP256 = 4,
    EccP384 = 5,
    EccP521 = 6,
}
impl KeyType {
    /// String value of the enum field names used in the ProtoBuf definition.
    ///
    /// The values are not transformed in any way and thus are considered stable
    /// (if the ProtoBuf definition does not change) and safe for programmatic use.
    pub fn as_str_name(&self) -> &'static str {
        match self {
            KeyType::Unspecified => "KEY_TYPE_UNSPECIFIED",
            KeyType::Rsa2048 => "KEY_TYPE_RSA_2048",
            KeyType::Rsa3072 => "KEY_TYPE_RSA_3072",
            KeyType::Rsa4096 => "KEY_TYPE_RSA_4096",
            KeyType::EccP256 => "KEY_TYPE_ECC_P256",
            KeyType::EccP384 => "KEY_TYPE_ECC_P384",
            KeyType::EccP521 => "KEY_TYPE_ECC_P521",
        }
    }
    /// Creates an enum from field names used in the ProtoBuf definition.
    pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
        match value {
            "KEY_TYPE_UNSPECIFIED" => Some(Self::Unspecified),
            "KEY_TYPE_RSA_2048" => Some(Self::Rsa2048),
            "KEY_TYPE_RSA_3072" => Some(Self::Rsa3072),
            "KEY_TYPE_RSA_4096" => Some(Self::Rsa4096),
            "KEY_TYPE_ECC_P256" => Some(Self::EccP256),
            "KEY_TYPE_ECC_P384" => Some(Self::EccP384),
            "KEY_TYPE_ECC_P521" => Some(Self::EccP521),
            _ => None,
        }
    }
}
/// Signing algorithms supported
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum SigningAlgorithm {
    Unspecified = 0,
    RsaPssSha256 = 1,
    RsaPssSha384 = 2,
    RsaPssSha512 = 3,
    RsaPkcs1Sha256 = 4,
    RsaPkcs1Sha384 = 5,
    RsaPkcs1Sha512 = 6,
    EcdsaSha256 = 7,
    EcdsaSha384 = 8,
    EcdsaSha512 = 9,
}
impl SigningAlgorithm {
    /// String value of the enum field names used in the ProtoBuf definition.
    ///
    /// The values are not transformed in any way and thus are considered stable
    /// (if the ProtoBuf definition does not change) and safe for programmatic use.
    pub fn as_str_name(&self) -> &'static str {
        match self {
            SigningAlgorithm::Unspecified => "SIGNING_ALGORITHM_UNSPECIFIED",
            SigningAlgorithm::RsaPssSha256 => "SIGNING_ALGORITHM_RSA_PSS_SHA256",
            SigningAlgorithm::RsaPssSha384 => "SIGNING_ALGORITHM_RSA_PSS_SHA384",
            SigningAlgorithm::RsaPssSha512 => "SIGNING_ALGORITHM_RSA_PSS_SHA512",
            SigningAlgorithm::RsaPkcs1Sha256 => "SIGNING_ALGORITHM_RSA_PKCS1_SHA256",
            SigningAlgorithm::RsaPkcs1Sha384 => "SIGNING_ALGORITHM_RSA_PKCS1_SHA384",
            SigningAlgorithm::RsaPkcs1Sha512 => "SIGNING_ALGORITHM_RSA_PKCS1_SHA512",
            SigningAlgorithm::EcdsaSha256 => "SIGNING_ALGORITHM_ECDSA_SHA256",
            SigningAlgorithm::EcdsaSha384 => "SIGNING_ALGORITHM_ECDSA_SHA384",
            SigningAlgorithm::EcdsaSha512 => "SIGNING_ALGORITHM_ECDSA_SHA512",
        }
    }
    /// Creates an enum from field names used in the ProtoBuf definition.
    pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
        match value {
            "SIGNING_ALGORITHM_UNSPECIFIED" => Some(Self::Unspecified),
            "SIGNING_ALGORITHM_RSA_PSS_SHA256" => Some(Self::RsaPssSha256),
            "SIGNING_ALGORITHM_RSA_PSS_SHA384" => Some(Self::RsaPssSha384),
            "SIGNING_ALGORITHM_RSA_PSS_SHA512" => Some(Self::RsaPssSha512),
            "SIGNING_ALGORITHM_RSA_PKCS1_SHA256" => Some(Self::RsaPkcs1Sha256),
            "SIGNING_ALGORITHM_RSA_PKCS1_SHA384" => Some(Self::RsaPkcs1Sha384),
            "SIGNING_ALGORITHM_RSA_PKCS1_SHA512" => Some(Self::RsaPkcs1Sha512),
            "SIGNING_ALGORITHM_ECDSA_SHA256" => Some(Self::EcdsaSha256),
            "SIGNING_ALGORITHM_ECDSA_SHA384" => Some(Self::EcdsaSha384),
            "SIGNING_ALGORITHM_ECDSA_SHA512" => Some(Self::EcdsaSha512),
            _ => None,
        }
    }
}
/// Hash algorithms supported
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum HashAlgorithm {
    Unspecified = 0,
    Sha256 = 1,
    Sha384 = 2,
    Sha512 = 3,
}
impl HashAlgorithm {
    /// String value of the enum field names used in the ProtoBuf definition.
    ///
    /// The values are not transformed in any way and thus are considered stable
    /// (if the ProtoBuf definition does not change) and safe for programmatic use.
    pub fn as_str_name(&self) -> &'static str {
        match self {
            HashAlgorithm::Unspecified => "HASH_ALGORITHM_UNSPECIFIED",
            HashAlgorithm::Sha256 => "HASH_ALGORITHM_SHA256",
            HashAlgorithm::Sha384 => "HASH_ALGORITHM_SHA384",
            HashAlgorithm::Sha512 => "HASH_ALGORITHM_SHA512",
        }
    }
    /// Creates an enum from field names used in the ProtoBuf definition.
    pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
        match value {
            "HASH_ALGORITHM_UNSPECIFIED" => Some(Self::Unspecified),
            "HASH_ALGORITHM_SHA256" => Some(Self::Sha256),
            "HASH_ALGORITHM_SHA384" => Some(Self::Sha384),
            "HASH_ALGORITHM_SHA512" => Some(Self::Sha512),
            _ => None,
        }
    }
}
/// Error codes for better error handling
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum ErrorCode {
    Unspecified = 0,
    InvalidKeyId = 1,
    InvalidAlgorithm = 2,
    InvalidData = 3,
    KeyGenerationFailed = 4,
    SigningFailed = 5,
    VerificationFailed = 6,
    KeyNotFound = 7,
    KeyAlreadyExists = 8,
    InternalError = 9,
    InvalidSignature = 10,
}
impl ErrorCode {
    /// String value of the enum field names used in the ProtoBuf definition.
    ///
    /// The values are not transformed in any way and thus are considered stable
    /// (if the ProtoBuf definition does not change) and safe for programmatic use.
    pub fn as_str_name(&self) -> &'static str {
        match self {
            ErrorCode::Unspecified => "ERROR_CODE_UNSPECIFIED",
            ErrorCode::InvalidKeyId => "ERROR_CODE_INVALID_KEY_ID",
            ErrorCode::InvalidAlgorithm => "ERROR_CODE_INVALID_ALGORITHM",
            ErrorCode::InvalidData => "ERROR_CODE_INVALID_DATA",
            ErrorCode::KeyGenerationFailed => "ERROR_CODE_KEY_GENERATION_FAILED",
            ErrorCode::SigningFailed => "ERROR_CODE_SIGNING_FAILED",
            ErrorCode::VerificationFailed => "ERROR_CODE_VERIFICATION_FAILED",
            ErrorCode::KeyNotFound => "ERROR_CODE_KEY_NOT_FOUND",
            ErrorCode::KeyAlreadyExists => "ERROR_CODE_KEY_ALREADY_EXISTS",
            ErrorCode::InternalError => "ERROR_CODE_INTERNAL_ERROR",
            ErrorCode::InvalidSignature => "ERROR_CODE_INVALID_SIGNATURE",
        }
    }
    /// Creates an enum from field names used in the ProtoBuf definition.
    pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
        match value {
            "ERROR_CODE_UNSPECIFIED" => Some(Self::Unspecified),
            "ERROR_CODE_INVALID_KEY_ID" => Some(Self::InvalidKeyId),
            "ERROR_CODE_INVALID_ALGORITHM" => Some(Self::InvalidAlgorithm),
            "ERROR_CODE_INVALID_DATA" => Some(Self::InvalidData),
            "ERROR_CODE_KEY_GENERATION_FAILED" => Some(Self::KeyGenerationFailed),
            "ERROR_CODE_SIGNING_FAILED" => Some(Self::SigningFailed),
            "ERROR_CODE_VERIFICATION_FAILED" => Some(Self::VerificationFailed),
            "ERROR_CODE_KEY_NOT_FOUND" => Some(Self::KeyNotFound),
            "ERROR_CODE_KEY_ALREADY_EXISTS" => Some(Self::KeyAlreadyExists),
            "ERROR_CODE_INTERNAL_ERROR" => Some(Self::InternalError),
            "ERROR_CODE_INVALID_SIGNATURE" => Some(Self::InvalidSignature),
            _ => None,
        }
    }
}
/// Generated client implementations.
pub mod signing_service_client {
    #![allow(unused_variables, dead_code, missing_docs, clippy::let_unit_value)]
    use tonic::codegen::*;
    use tonic::codegen::http::Uri;
    /// The signing service definition
    #[derive(Debug, Clone)]
    pub struct SigningServiceClient<T> {
        inner: tonic::client::Grpc<T>,
    }
    impl SigningServiceClient<tonic::transport::Channel> {
        /// Attempt to create a new client by connecting to a given endpoint.
        pub async fn connect<D>(dst: D) -> Result<Self, tonic::transport::Error>
        where
            D: std::convert::TryInto<tonic::transport::Endpoint>,
            D::Error: Into<StdError>,
        {
            let conn = tonic::transport::Endpoint::new(dst)?.connect().await?;
            Ok(Self::new(conn))
        }
    }
    impl<T> SigningServiceClient<T>
    where
        T: tonic::client::GrpcService<tonic::body::BoxBody>,
        T::Error: Into<StdError>,
        T::ResponseBody: Body<Data = Bytes> + Send + 'static,
        <T::ResponseBody as Body>::Error: Into<StdError> + Send,
    {
        pub fn new(inner: T) -> Self {
            let inner = tonic::client::Grpc::new(inner);
            Self { inner }
        }
        pub fn with_origin(inner: T, origin: Uri) -> Self {
            let inner = tonic::client::Grpc::with_origin(inner, origin);
            Self { inner }
        }
        pub fn with_interceptor<F>(
            inner: T,
            interceptor: F,
        ) -> SigningServiceClient<InterceptedService<T, F>>
        where
            F: tonic::service::Interceptor,
            T::ResponseBody: Default,
            T: tonic::codegen::Service<
                http::Request<tonic::body::BoxBody>,
                Response = http::Response<
                    <T as tonic::client::GrpcService<tonic::body::BoxBody>>::ResponseBody,
                >,
            >,
            <T as tonic::codegen::Service<
                http::Request<tonic::body::BoxBody>,
            >>::Error: Into<StdError> + Send + Sync,
        {
            SigningServiceClient::new(InterceptedService::new(inner, interceptor))
        }
        /// Compress requests with the given encoding.
        ///
        /// This requires the server to support it otherwise it might respond with an
        /// error.
        #[must_use]
        pub fn send_compressed(mut self, encoding: CompressionEncoding) -> Self {
            self.inner = self.inner.send_compressed(encoding);
            self
        }
        /// Enable decompressing responses.
        #[must_use]
        pub fn accept_compressed(mut self, encoding: CompressionEncoding) -> Self {
            self.inner = self.inner.accept_compressed(encoding);
            self
        }
        /// Signs data using the specified key and algorithm
        pub async fn sign(
            &mut self,
            request: impl tonic::IntoRequest<super::SignRequest>,
        ) -> Result<tonic::Response<super::SignResponse>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/signing.SigningService/Sign",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        /// Generates a new key pair
        pub async fn generate_key(
            &mut self,
            request: impl tonic::IntoRequest<super::GenerateKeyRequest>,
        ) -> Result<tonic::Response<super::GenerateKeyResponse>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/signing.SigningService/GenerateKey",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        /// Lists available keys
        pub async fn list_keys(
            &mut self,
            request: impl tonic::IntoRequest<super::ListKeysRequest>,
        ) -> Result<tonic::Response<super::ListKeysResponse>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/signing.SigningService/ListKeys",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        /// Deletes a key
        pub async fn delete_key(
            &mut self,
            request: impl tonic::IntoRequest<super::DeleteKeyRequest>,
        ) -> Result<tonic::Response<super::DeleteKeyResponse>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/signing.SigningService/DeleteKey",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        /// Health check
        pub async fn health_check(
            &mut self,
            request: impl tonic::IntoRequest<super::HealthCheckRequest>,
        ) -> Result<tonic::Response<super::HealthCheckResponse>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/signing.SigningService/HealthCheck",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        /// Verifies a signature
        pub async fn verify(
            &mut self,
            request: impl tonic::IntoRequest<super::VerifyRequest>,
        ) -> Result<tonic::Response<super::VerifyResponse>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/signing.SigningService/Verify",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
    }
}
/// Generated server implementations.
pub mod signing_service_server {
    #![allow(unused_variables, dead_code, missing_docs, clippy::let_unit_value)]
    use tonic::codegen::*;
    /// Generated trait containing gRPC methods that should be implemented for use with SigningServiceServer.
    #[async_trait]
    pub trait SigningService: Send + Sync + 'static {
        /// Signs data using the specified key and algorithm
        async fn sign(
            &self,
            request: tonic::Request<super::SignRequest>,
        ) -> Result<tonic::Response<super::SignResponse>, tonic::Status>;
        /// Generates a new key pair
        async fn generate_key(
            &self,
            request: tonic::Request<super::GenerateKeyRequest>,
        ) -> Result<tonic::Response<super::GenerateKeyResponse>, tonic::Status>;
        /// Lists available keys
        async fn list_keys(
            &self,
            request: tonic::Request<super::ListKeysRequest>,
        ) -> Result<tonic::Response<super::ListKeysResponse>, tonic::Status>;
        /// Deletes a key
        async fn delete_key(
            &self,
            request: tonic::Request<super::DeleteKeyRequest>,
        ) -> Result<tonic::Response<super::DeleteKeyResponse>, tonic::Status>;
        /// Health check
        async fn health_check(
            &self,
            request: tonic::Request<super::HealthCheckRequest>,
        ) -> Result<tonic::Response<super::HealthCheckResponse>, tonic::Status>;
        /// Verifies a signature
        async fn verify(
            &self,
            request: tonic::Request<super::VerifyRequest>,
        ) -> Result<tonic::Response<super::VerifyResponse>, tonic::Status>;
    }
    /// The signing service definition
    #[derive(Debug)]
    pub struct SigningServiceServer<T: SigningService> {
        inner: _Inner<T>,
        accept_compression_encodings: EnabledCompressionEncodings,
        send_compression_encodings: EnabledCompressionEncodings,
    }
    struct _Inner<T>(Arc<T>);
    impl<T: SigningService> SigningServiceServer<T> {
        pub fn new(inner: T) -> Self {
            Self::from_arc(Arc::new(inner))
        }
        pub fn from_arc(inner: Arc<T>) -> Self {
            let inner = _Inner(inner);
            Self {
                inner,
                accept_compression_encodings: Default::default(),
                send_compression_encodings: Default::default(),
            }
        }
        pub fn with_interceptor<F>(
            inner: T,
            interceptor: F,
        ) -> InterceptedService<Self, F>
        where
            F: tonic::service::Interceptor,
        {
            InterceptedService::new(Self::new(inner), interceptor)
        }
        /// Enable decompressing requests with the given encoding.
        #[must_use]
        pub fn accept_compressed(mut self, encoding: CompressionEncoding) -> Self {
            self.accept_compression_encodings.enable(encoding);
            self
        }
        /// Compress responses with the given encoding, if the client supports it.
        #[must_use]
        pub fn send_compressed(mut self, encoding: CompressionEncoding) -> Self {
            self.send_compression_encodings.enable(encoding);
            self
        }
    }
    impl<T, B> tonic::codegen::Service<http::Request<B>> for SigningServiceServer<T>
    where
        T: SigningService,
        B: Body + Send + 'static,
        B::Error: Into<StdError> + Send + 'static,
    {
        type Response = http::Response<tonic::body::BoxBody>;
        type Error = std::convert::Infallible;
        type Future = BoxFuture<Self::Response, Self::Error>;
        fn poll_ready(
            &mut self,
            _cx: &mut Context<'_>,
        ) -> Poll<Result<(), Self::Error>> {
            Poll::Ready(Ok(()))
        }
        fn call(&mut self, req: http::Request<B>) -> Self::Future {
            let inner = self.inner.clone();
            match req.uri().path() {
                "/signing.SigningService/Sign" => {
                    #[allow(non_camel_case_types)]
                    struct SignSvc<T: SigningService>(pub Arc<T>);
                    impl<
                        T: SigningService,
                    > tonic::server::UnaryService<super::SignRequest> for SignSvc<T> {
                        type Response = super::SignResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::SignRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).sign(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = SignSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/signing.SigningService/GenerateKey" => {
                    #[allow(non_camel_case_types)]
                    struct GenerateKeySvc<T: SigningService>(pub Arc<T>);
                    impl<
                        T: SigningService,
                    > tonic::server::UnaryService<super::GenerateKeyRequest>
                    for GenerateKeySvc<T> {
                        type Response = super::GenerateKeyResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::GenerateKeyRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move {
                                (*inner).generate_key(request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = GenerateKeySvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/signing.SigningService/ListKeys" => {
                    #[allow(non_camel_case_types)]
                    struct ListKeysSvc<T: SigningService>(pub Arc<T>);
                    impl<
                        T: SigningService,
                    > tonic::server::UnaryService<super::ListKeysRequest>
                    for ListKeysSvc<T> {
                        type Response = super::ListKeysResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::ListKeysRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).list_keys(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = ListKeysSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/signing.SigningService/DeleteKey" => {
                    #[allow(non_camel_case_types)]
                    struct DeleteKeySvc<T: SigningService>(pub Arc<T>);
                    impl<
                        T: SigningService,
                    > tonic::server::UnaryService<super::DeleteKeyRequest>
                    for DeleteKeySvc<T> {
                        type Response = super::DeleteKeyResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::DeleteKeyRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).delete_key(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = DeleteKeySvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/signing.SigningService/HealthCheck" => {
                    #[allow(non_camel_case_types)]
                    struct HealthCheckSvc<T: SigningService>(pub Arc<T>);
                    impl<
                        T: SigningService,
                    > tonic::server::UnaryService<super::HealthCheckRequest>
                    for HealthCheckSvc<T> {
                        type Response = super::HealthCheckResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::HealthCheckRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move {
                                (*inner).health_check(request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = HealthCheckSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/signing.SigningService/Verify" => {
                    #[allow(non_camel_case_types)]
                    struct VerifySvc<T: SigningService>(pub Arc<T>);
                    impl<
                        T: SigningService,
                    > tonic::server::UnaryService<super::VerifyRequest>
                    for VerifySvc<T> {
                        type Response = super::VerifyResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::VerifyRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).verify(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = VerifySvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                _ => {
                    Box::pin(async move {
                        Ok(
                            http::Response::builder()
                                .status(200)
                                .header("grpc-status", "12")
                                .header("content-type", "application/grpc")
                                .body(empty_body())
                                .unwrap(),
                        )
                    })
                }
            }
        }
    }
    impl<T: SigningService> Clone for SigningServiceServer<T> {
        fn clone(&self) -> Self {
            let inner = self.inner.clone();
            Self {
                inner,
                accept_compression_encodings: self.accept_compression_encodings,
                send_compression_encodings: self.send_compression_encodings,
            }
        }
    }
    impl<T: SigningService> Clone for _Inner<T> {
        fn clone(&self) -> Self {
            Self(self.0.clone())
        }
    }
    impl<T: std::fmt::Debug> std::fmt::Debug for _Inner<T> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{:?}", self.0)
        }
    }
    impl<T: SigningService> tonic::server::NamedService for SigningServiceServer<T> {
        const NAME: &'static str = "signing.SigningService";
    }
}
