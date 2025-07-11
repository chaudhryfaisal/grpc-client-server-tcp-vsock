/// Request message for signing operations
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SignRequest {
    /// Data to be signed
    #[prost(bytes = "vec", tag = "1")]
    pub data: ::prost::alloc::vec::Vec<u8>,
    /// Key type for signing
    #[prost(enumeration = "KeyType", tag = "2")]
    pub key_type: i32,
    /// Signing algorithm
    #[prost(enumeration = "SigningAlgorithm", tag = "3")]
    pub algorithm: i32,
    /// Optional key identifier
    #[prost(string, tag = "4")]
    pub key_id: ::prost::alloc::string::String,
}
/// Response message for signing operations
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SignResponse {
    /// The signature
    #[prost(bytes = "vec", tag = "1")]
    pub signature: ::prost::alloc::vec::Vec<u8>,
    /// Key type used
    #[prost(enumeration = "KeyType", tag = "2")]
    pub key_type: i32,
    /// Algorithm used
    #[prost(enumeration = "SigningAlgorithm", tag = "3")]
    pub algorithm: i32,
    /// Processing time in microseconds
    #[prost(uint64, tag = "4")]
    pub processing_time_us: u64,
}
/// Health check request
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct HealthCheckRequest {
    #[prost(string, tag = "1")]
    pub service: ::prost::alloc::string::String,
}
/// Health check response
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct HealthCheckResponse {
    #[prost(enumeration = "health_check_response::ServingStatus", tag = "1")]
    pub status: i32,
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
/// Supported key types
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
            KeyType::Rsa2048 => "RSA_2048",
            KeyType::Rsa3072 => "RSA_3072",
            KeyType::Rsa4096 => "RSA_4096",
            KeyType::EccP256 => "ECC_P256",
            KeyType::EccP384 => "ECC_P384",
            KeyType::EccP521 => "ECC_P521",
        }
    }
    /// Creates an enum from field names used in the ProtoBuf definition.
    pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
        match value {
            "KEY_TYPE_UNSPECIFIED" => Some(Self::Unspecified),
            "RSA_2048" => Some(Self::Rsa2048),
            "RSA_3072" => Some(Self::Rsa3072),
            "RSA_4096" => Some(Self::Rsa4096),
            "ECC_P256" => Some(Self::EccP256),
            "ECC_P384" => Some(Self::EccP384),
            "ECC_P521" => Some(Self::EccP521),
            _ => None,
        }
    }
}
/// Supported signing algorithms
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum SigningAlgorithm {
    AlgorithmUnspecified = 0,
    RsaPssSha256 = 1,
    RsaPssSha384 = 2,
    RsaPssSha512 = 3,
    RsaPkcs1v15Sha256 = 4,
    RsaPkcs1v15Sha384 = 5,
    RsaPkcs1v15Sha512 = 6,
    EcdsaP256Sha256 = 7,
    EcdsaP384Sha384 = 8,
    EcdsaP521Sha512 = 9,
}
impl SigningAlgorithm {
    /// String value of the enum field names used in the ProtoBuf definition.
    ///
    /// The values are not transformed in any way and thus are considered stable
    /// (if the ProtoBuf definition does not change) and safe for programmatic use.
    pub fn as_str_name(&self) -> &'static str {
        match self {
            SigningAlgorithm::AlgorithmUnspecified => "ALGORITHM_UNSPECIFIED",
            SigningAlgorithm::RsaPssSha256 => "RSA_PSS_SHA256",
            SigningAlgorithm::RsaPssSha384 => "RSA_PSS_SHA384",
            SigningAlgorithm::RsaPssSha512 => "RSA_PSS_SHA512",
            SigningAlgorithm::RsaPkcs1v15Sha256 => "RSA_PKCS1V15_SHA256",
            SigningAlgorithm::RsaPkcs1v15Sha384 => "RSA_PKCS1V15_SHA384",
            SigningAlgorithm::RsaPkcs1v15Sha512 => "RSA_PKCS1V15_SHA512",
            SigningAlgorithm::EcdsaP256Sha256 => "ECDSA_P256_SHA256",
            SigningAlgorithm::EcdsaP384Sha384 => "ECDSA_P384_SHA384",
            SigningAlgorithm::EcdsaP521Sha512 => "ECDSA_P521_SHA512",
        }
    }
    /// Creates an enum from field names used in the ProtoBuf definition.
    pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
        match value {
            "ALGORITHM_UNSPECIFIED" => Some(Self::AlgorithmUnspecified),
            "RSA_PSS_SHA256" => Some(Self::RsaPssSha256),
            "RSA_PSS_SHA384" => Some(Self::RsaPssSha384),
            "RSA_PSS_SHA512" => Some(Self::RsaPssSha512),
            "RSA_PKCS1V15_SHA256" => Some(Self::RsaPkcs1v15Sha256),
            "RSA_PKCS1V15_SHA384" => Some(Self::RsaPkcs1v15Sha384),
            "RSA_PKCS1V15_SHA512" => Some(Self::RsaPkcs1v15Sha512),
            "ECDSA_P256_SHA256" => Some(Self::EcdsaP256Sha256),
            "ECDSA_P384_SHA384" => Some(Self::EcdsaP384Sha384),
            "ECDSA_P521_SHA512" => Some(Self::EcdsaP521Sha512),
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
        /// Performs cryptographic signing operations
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
        /// Health check endpoint
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
    }
}
/// Generated server implementations.
pub mod signing_service_server {
    #![allow(unused_variables, dead_code, missing_docs, clippy::let_unit_value)]
    use tonic::codegen::*;
    /// Generated trait containing gRPC methods that should be implemented for use with SigningServiceServer.
    #[async_trait]
    pub trait SigningService: Send + Sync + 'static {
        /// Performs cryptographic signing operations
        async fn sign(
            &self,
            request: tonic::Request<super::SignRequest>,
        ) -> Result<tonic::Response<super::SignResponse>, tonic::Status>;
        /// Health check endpoint
        async fn health_check(
            &self,
            request: tonic::Request<super::HealthCheckRequest>,
        ) -> Result<tonic::Response<super::HealthCheckResponse>, tonic::Status>;
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
