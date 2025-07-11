# Rust gRPC Client/Server with TCP/VSOCK Transport

A high-performance gRPC client/server implementation in Rust with cryptographic signing capabilities, supporting both TCP and VSOCK transports.

## Project Structure

This is a Rust workspace containing three crates:

- **`grpc-shared`**: Core library crate with shared functionality
- **`grpc-server`**: Server binary crate
- **`grpc-client`**: Client binary crate

### Architecture Overview

```
rust-grpc-client-server-tcp-vsock/
├── Cargo.toml                    # Workspace configuration
├── grpc-shared/                  # Shared library crate
│   ├── Cargo.toml               # Library dependencies
│   ├── build.rs                 # Protocol buffer compilation
│   ├── proto/
│   │   └── signing.proto        # gRPC service definitions
│   └── src/
│       ├── lib.rs               # Main library entry point
│       ├── config/              # Configuration management
│       ├── crypto/              # Cryptographic operations
│       ├── error/               # Error handling
│       ├── transport/           # TCP/VSOCK transport layers
│       ├── server/              # Server-side components
│       ├── client/              # Client-side components
│       ├── proto/               # Generated protobuf code
│       └── benchmarks/          # Performance benchmarking
├── grpc-server/                 # Server binary
│   ├── Cargo.toml
│   └── src/main.rs
└── grpc-client/                 # Client binary
    ├── Cargo.toml
    └── src/main.rs
```

## Features

### Core Capabilities
- **gRPC Communication**: High-performance RPC using tonic
- **Cryptographic Signing**: RSA and ECC signing operations using the ring crate
- **Dual Transport Support**: TCP and VSOCK transports
- **Async/Await**: Full async support with tokio runtime
- **Configuration Management**: TOML-based configuration with serde
- **Comprehensive Error Handling**: Custom error types with thiserror
- **CLI Interface**: Command-line interfaces for both server and client
- **Benchmarking**: Performance measurement capabilities

### Transport Layers
- **TCP**: Standard network transport for general use
- **VSOCK**: Virtual socket transport for VM-to-host communication (Linux-specific)

### Cryptographic Operations
- **RSA Signing**: RSA-PSS and PKCS#1 v1.5 with SHA-256/384/512
- **ECC Signing**: ECDSA with P-256, P-384, P-521 curves
- **Key Management**: Secure key generation and loading
- **Signature Verification**: Full verification support

## Dependencies

### Core Dependencies
- **tokio**: Async runtime
- **tonic**: gRPC framework
- **prost**: Protocol buffer implementation
- **ring**: Cryptographic operations
- **serde**: Serialization framework
- **thiserror**: Error handling
- **clap**: Command-line argument parsing
- **tracing**: Structured logging

### Optional Features
- **vsock**: VSOCK transport support (Linux only)
- **crypto**: Cryptographic operations
- **benchmarks**: Performance benchmarking

## Building

### Prerequisites
- Rust 1.70+ (2021 edition)
- Protocol Buffers compiler (`protoc`)

### Install protoc
```bash
# Ubuntu/Debian
sudo apt-get install protobuf-compiler

# macOS
brew install protobuf

# Or download from: https://github.com/protocolbuffers/protobuf/releases
```

### Build Commands
```bash
# Check all crates
cargo check --workspace

# Build all crates
cargo build --workspace

# Build with release optimizations
cargo build --workspace --release

# Build with specific features
cargo build --workspace --features "crypto,vsock,benchmarks"
```

## Usage

### Server
```bash
# Run with default configuration
cargo run --bin grpc-server

# Run with custom configuration
cargo run --bin grpc-server -- --config custom-server.toml

# Run with specific bind address and port
cargo run --bin grpc-server -- --bind-address 127.0.0.1 --port 50051

# Show help
cargo run --bin grpc-server -- --help
```

### Client
```bash
# Run with default configuration
cargo run --bin grpc-client

# Run with custom server address
cargo run --bin grpc-client -- --server-address 127.0.0.1:50051

# Run with custom data to sign
cargo run --bin grpc-client -- --data "Custom message to sign"

# Run in benchmark mode
cargo run --bin grpc-client -- --benchmark

# Show help
cargo run --bin grpc-client -- --help
```

## Configuration

### Server Configuration (server-config.toml)
```toml
[server]
bind_address = "127.0.0.1"
port = 50051
transport = "tcp"  # or "vsock"

[crypto]
key_path = "server-key.pem"
algorithm = "rsa_pss_sha256"

[logging]
level = "info"
```

### Client Configuration (client-config.toml)
```toml
[client]
server_address = "127.0.0.1:50051"
transport = "tcp"  # or "vsock"
timeout_ms = 5000

[logging]
level = "info"
```

## Protocol Buffer Schema

The gRPC service is defined in [`grpc-shared/proto/signing.proto`](grpc-shared/proto/signing.proto):

```protobuf
service SigningService {
  rpc Sign(SignRequest) returns (SignResponse);
  rpc Verify(VerifyRequest) returns (VerifyResponse);
  rpc Health(HealthCheckRequest) returns (HealthCheckResponse);
}
```

## Development Status

### ✅ Completed (Phase 1, Task 1: Project Setup)
- [x] Workspace structure with three crates
- [x] Complete dependency configuration
- [x] Protocol buffer definitions and compilation
- [x] Module structure and organization
- [x] Error handling framework
- [x] Configuration management
- [x] Transport layer abstractions
- [x] CLI interfaces for server and client
- [x] Feature flags for optional dependencies
- [x] Comprehensive documentation

### 🚧 Next Steps (Phase 1, Task 2+)
- [ ] Implement actual cryptographic operations
- [ ] Complete transport layer implementations
- [ ] Add gRPC service implementations
- [ ] Implement configuration file loading
- [ ] Add comprehensive testing
- [ ] Performance benchmarking
- [ ] Security hardening

## Testing

```bash
# Run all tests
cargo test --workspace

# Run tests with all features
cargo test --workspace --all-features

# Run specific crate tests
cargo test -p grpc-shared
cargo test -p grpc-server
cargo test -p grpc-client
```

## Performance

The project is designed for high performance with:
- Zero-copy operations where possible
- Async/await for non-blocking I/O
- Efficient serialization with Protocol Buffers
- Optimized cryptographic operations with ring
- Connection pooling and reuse
- Configurable timeouts and limits

## Security Considerations

- Cryptographic operations use the audited `ring` crate
- Secure key generation and storage
- Input validation and sanitization
- Memory-safe Rust implementation
- Configurable security parameters

## Platform Support

- **Linux**: Full support including VSOCK
- **macOS**: TCP transport only
- **Windows**: TCP transport only

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests for new functionality
5. Ensure all tests pass
6. Submit a pull request

## Troubleshooting

### Common Issues

1. **protoc not found**: Install Protocol Buffers compiler
2. **VSOCK not available**: VSOCK is Linux-specific, use TCP on other platforms
3. **Permission denied**: Ensure proper file permissions for key files
4. **Port already in use**: Change the port in configuration or stop conflicting services

### Debug Mode
```bash
# Run with debug logging
RUST_LOG=debug cargo run --bin grpc-server
RUST_LOG=debug cargo run --bin grpc-client