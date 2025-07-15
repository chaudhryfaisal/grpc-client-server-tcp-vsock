# Technology Stack and Implementation Details

## Core Technologies

### Rust Ecosystem
- **Language**: Rust 2021 Edition
- **Package Manager**: Cargo with workspace-based single crate design
- **Async Runtime**: Tokio with multi-threaded runtime configuration

### gRPC Framework
- **Core**: Tonic 0.11 - High-performance gRPC implementation
- **Protocol Buffers**: Prost 0.12 for serialization
- **Build Integration**: tonic-build 0.11 for proto compilation

### Transport Layer
- **TCP**: Standard tokio::net for TCP networking
- **VSOCK**: tokio-vsock 0.4 for VM-to-host communication
- **HTTP/2**: Hyper 0.14 with Tower 0.4 for service composition

### Cryptography (Placeholder)
- **Target**: Ring 0.17 for production cryptography
- **Current**: Placeholder implementations for RSA/ECC operations
- **Algorithms**: RSA (PKCS#1, PSS), ECDSA (P-256, P-384)

### Development Tools
- **Logging**: log 0.4 + env_logger 0.10 for structured logging
- **Error Handling**: thiserror 1.0 for custom error types
- **CLI**: clap 4.0 for command-line argument parsing
- **Testing**: tokio-test 0.4 for async testing

## Dependencies Configuration

### Production Dependencies
```toml
tonic = "0.11"                    # gRPC framework
prost = "0.12"                    # Protocol buffers
tokio = { version = "1.0", features = ["macros", "rt-multi-thread", "time"] }
log = "0.4"                       # Logging facade
env_logger = "0.10"               # Environment-based logger
thiserror = "1.0"                 # Error handling
http = "0.2"                      # HTTP types
ring = "0.17"                     # Cryptography (placeholder)
rand = "0.8"                      # Random number generation
vsock = "0.4"                     # VSOCK support
tokio-vsock = "0.4"               # Async VSOCK
hyper = "0.14"                    # HTTP implementation
tower = "0.4"                     # Service composition
tower-service = "0.3"             # Service trait
async-trait = "0.1"               # Async traits
clap = { version = "4.0", features = ["derive"] }  # CLI parsing
serde = { version = "1.0", features = ["derive"] } # Serialization
serde_json = "1.0"                # JSON support
num_cpus = "1.16"                 # CPU detection
```

### Build Dependencies
```toml
tonic-build = "0.11"              # Proto compilation
```

### Development Dependencies
```toml
tokio-test = "0.4"                # Async testing utilities
```

## Development Setup

### Build System
- **Primary**: Cargo for Rust compilation
- **Secondary**: Makefile for common operations
- **Proto Compilation**: Automated via build.rs
- **Release Optimization**: All binaries built in release mode

### Environment Variables
- **RUST_LOG**: Configurable logging levels (trace, debug, info, warn, error)
- **SERVER_ADDR**: Server binding address (default: 127.0.0.1:50051)
- **WORKER_THREADS**: Number of tokio worker threads (default: CPU count)

### Makefile Targets
- `build`: Release mode compilation of all binaries
- `test`: Integration test execution
- `server`: Start gRPC server
- `client`: Run sample client
- `benchmark`: Performance testing with various configurations

## Technical Constraints

### Performance Requirements
- **Latency**: Sub-millisecond response times (currently 1-3ms)
- **Throughput**: Target 100K+ requests per second
- **Memory**: Minimal allocation during request processing
- **CPU**: Optimized for both single and multi-core systems

### Error Handling Constraints
- **No Panics**: Zero panic!/unwrap() calls in production code
- **Result Types**: All operations return Result<T, E>
- **gRPC Status**: Proper status code mapping for all errors
- **Graceful Degradation**: Fallback behavior for transport failures

### Compilation Constraints
- **Fast Builds**: Minimal dependencies for quick compilation
- **Feature Flags**: Only required dependency features enabled
- **Single Crate**: All code in one crate to avoid workspace overhead

## Tool Usage Patterns

### Logging Strategy
- **Structured Logging**: Consistent log format across all components
- **Performance Metrics**: Request/response latency tracking
- **Debug Information**: Detailed payload and timing information
- **Configurable Levels**: Runtime log level adjustment

### Testing Approach
- **Integration Tests**: End-to-end testing of all services
- **Transport Testing**: Both TCP and VSOCK transport validation
- **Error Scenarios**: Comprehensive error condition testing
- **Performance Validation**: Latency and throughput verification

### Benchmarking Framework
- **Atomic Metrics**: Thread-safe performance counters
- **Configurable Load**: Variable connection and request counts
- **Rate Limiting**: Controlled request rate testing
- **Duration-based**: Time-bounded test execution
- **Multiple Services**: Echo and crypto service benchmarking

## Protocol Definitions

### Echo Service
```protobuf
service EchoService {
    rpc Echo(EchoRequest) returns (EchoResponse);
}
```
- **Purpose**: Minimal latency request/response testing
- **Payload**: String data with timestamps
- **Metrics**: Server-side and total latency measurement

### Crypto Service
```protobuf
service CryptoService {
    rpc Sign(SignRequest) returns (SignResponse);
    rpc GetPublicKey(PublicKeyRequest) returns (PublicKeyResponse);
}
```
- **Purpose**: Cryptographic operations via gRPC
- **Key Types**: RSA and ECC support
- **Algorithms**: PKCS#1, PSS, ECDSA variants

## Transport Configuration

### TCP Transport
- **Address Format**: `host:port` (e.g., "127.0.0.1:50051")
- **Optimizations**: TCP keepalive, nodelay, connection pooling
- **Use Case**: Standard network communication

### VSOCK Transport
- **Address Format**: `vsock://cid:port` (e.g., "vsock://2:50051")
- **Optimizations**: Minimal overhead for VM communication
- **Use Case**: VM-to-host high-performance communication

## Future Technology Integration

### Production Cryptography
- **Ring Integration**: Replace placeholder crypto with ring crate
- **Key Management**: Secure key generation and storage
- **Algorithm Support**: Full RSA and ECC implementation

### Advanced Monitoring
- **Metrics Collection**: Prometheus-compatible metrics
- **Distributed Tracing**: OpenTelemetry integration
- **Performance Profiling**: CPU and memory profiling support