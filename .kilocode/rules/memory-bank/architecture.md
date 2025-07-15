# System Architecture

## Project Structure

### Single Crate Design
- **Package Name**: `grpc-performance-rs`
- **Organization**: All components in one Rust crate for fast compilation
- **Multiple Binaries**: Separate executables for server, client, and benchmark
- **Shared Code**: Common protocol definitions and utilities in [`src/lib.rs`](src/lib.rs:1)

### Source Code Paths

#### Core Library
- [`src/lib.rs`](src/lib.rs:1) - Shared types, utilities, and proto includes
- [`src/transport.rs`](src/transport.rs:1) - Transport abstraction layer for TCP/VSOCK

#### Binary Executables
- [`src/bin/server.rs`](src/bin/server.rs:1) - gRPC server with dual transport support
- [`src/bin/client.rs`](src/bin/client.rs:1) - gRPC client for testing services
- [`src/bin/benchmark.rs`](src/bin/benchmark.rs:1) - Performance benchmarking tool

#### Protocol Definitions
- [`proto/echo.proto`](proto/echo.proto:1) - Echo service definition
- [`proto/crypto.proto`](proto/crypto.proto:1) - Crypto service definition
- [`build.rs`](build.rs:1) - Proto compilation configuration

#### Testing & Examples
- [`tests/integration_test.rs`](tests/integration_test.rs:1) - End-to-end integration tests
- [`examples/transport_demo.rs`](examples/transport_demo.rs:1) - Transport layer demonstration

## Key Technical Decisions

### Transport Abstraction Layer
**Design Pattern**: Unified interface for TCP and VSOCK transports
- [`TransportConfig`](src/transport.rs:11) enum for configuration
- [`TransportFactory`](src/transport.rs:261) for creating connections/listeners
- [`Connection`](src/transport.rs:103) and [`Listener`](src/transport.rs:144) unified types
- **Benefit**: Identical service functionality across both transports

### Error Handling Strategy
**Pattern**: No panics/unwraps in production code
- [`AppError`](src/lib.rs:27) custom error type with thiserror
- [`AppResult<T>`](src/lib.rs:50) type alias for consistent error handling
- **gRPC Status Codes**: Proper error mapping in service implementations
- **Error Propagation**: All operations return Results, never panic

### Performance Optimizations
**HTTP/2 Configuration**: Optimized for high throughput
- TCP keepalive, nodelay settings
- Adaptive window sizing (1MB initial windows)
- Connection pooling and stream limits
- **Location**: [`server.rs:225-233`](src/bin/server.rs:225)

### Cryptographic Architecture
**Current State**: Placeholder implementation with proper interface
- [`CryptoKeys`](src/lib.rs:61) struct for key management
- Support for RSA (PKCS#1, PSS) and ECC (P-256, P-384) algorithms
- **Future**: Ring crate integration for production cryptography

## Component Relationships

### Service Layer
```
EchoService ──┐
              ├── gRPC Server ── Transport Layer ── TCP/VSOCK
CryptoService ─┘
```

### Transport Layer Architecture
```
TransportConfig ── TransportFactory ──┬── TcpTransport
                                      └── VsockTransport
                                            │
                                            ├── Connection (TCP/VSOCK)
                                            └── Listener (TCP/VSOCK)
```

### Client Architecture
```
Client Binary ──┬── EchoServiceClient ──┐
                └── CryptoServiceClient ─┤── Channel ── Transport Layer
                                         │
Benchmark ──────┬── Metrics Collection ──┘
                └── Rate Limiting
```

## Critical Implementation Paths

### Server Startup Flow
1. **Runtime Configuration**: [`server.rs:200-209`](src/bin/server.rs:200) - Multi-threaded tokio runtime
2. **Service Initialization**: [`server.rs:212-222`](src/bin/server.rs:212) - Echo and Crypto services
3. **Server Configuration**: [`server.rs:225-233`](src/bin/server.rs:225) - HTTP/2 optimizations
4. **Service Registration**: [`server.rs:236-240`](src/bin/server.rs:236) - Add services to server

### Request Processing Flow
1. **Transport Layer**: Unified connection handling
2. **Service Layer**: Echo/Crypto service implementations
3. **Error Handling**: Proper gRPC status code mapping
4. **Logging**: Detailed request/response logging with latency metrics

### Benchmark Architecture
1. **Configuration**: CLI args + environment variables
2. **Metrics Collection**: Atomic counters for thread-safe statistics
3. **Connection Management**: Semaphore-based concurrency control
4. **Rate Limiting**: Configurable requests per second

## Design Patterns in Use

### Factory Pattern
- [`TransportFactory`](src/transport.rs:261) for creating transport instances
- Abstracts TCP vs VSOCK creation logic

### Strategy Pattern
- [`Transport`](src/transport.rs:184) trait for different transport implementations
- Allows seamless switching between TCP and VSOCK

### Builder Pattern
- Tokio runtime configuration
- gRPC server configuration with optimizations

### Error Handling Pattern
- Custom error types with thiserror
- Result-based error propagation
- No panic/unwrap in production code

## Performance Architecture

### Threading Model
- **Server**: Multi-threaded tokio runtime with configurable worker threads
- **Client**: Async/await with connection pooling
- **Benchmark**: Semaphore-controlled concurrency

### Memory Management
- **Zero-copy**: Where possible in transport layer
- **Connection Pooling**: Reuse connections for performance
- **Efficient Serialization**: Protobuf with minimal overhead

### Monitoring Points
- Request/response latency measurement
- Success/failure rate tracking
- Throughput metrics (requests per second)
- Transport-specific performance characteristics