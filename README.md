# High-Performance gRPC Client/Server System

A high-performance, minimal latency gRPC client and server implementation in Rust supporting both TCP and VSOCK transports with cryptographic services.

## Overview

This project provides:
- **Echo Service**: High-performance request/response testing with latency measurement
- **Crypto Service**: RSA and ECC signing operations via gRPC
- **Dual Transport**: Support for both TCP and VSOCK communication
- **Performance Benchmarking**: Comprehensive benchmarking tools with detailed metrics

## Quick Start

### Build the Project

```bash
# Build all binaries in release mode
make build

# Or use cargo directly
cargo build --release --bins
```

### Run Integration Tests

```bash
make test
```

## Running the Binaries

### 1. gRPC Server (`src/bin/server.rs`)

The server provides both Echo and Crypto services with optimized HTTP/2 configuration.

#### Basic Usage

```bash
# Start server with default settings
make server

# Or run directly
./target/release/server

# Stop the server
make stop-server
```

#### Configuration Options

The server is configured via environment variables:

| Environment Variable | Default Value | Description |
|---------------------|---------------|-------------|
| `SERVER_ADDR` | `127.0.0.1:50051` | Server bind address (TCP) or `vsock://cid:port` (VSOCK) |
| `WORKER_THREADS` | CPU count | Number of tokio worker threads |
| `RUST_LOG` | `info` | Log level (`trace`, `debug`, `info`, `warn`, `error`) |

#### Examples

```bash
# TCP server on custom port
SERVER_ADDR=127.0.0.1:8080 ./target/release/server

# VSOCK server (for VM environments)
SERVER_ADDR=vsock://2:50051 ./target/release/server

# Custom worker threads and debug logging
WORKER_THREADS=8 RUST_LOG=debug ./target/release/server

# Minimal logging for production
RUST_LOG=warn ./target/release/server
```

#### Server Features

- **Echo Service**: Logs and echoes back payloads with timestamp information
- **Crypto Service**: 
  - RSA signing (PKCS#1, PSS with SHA-256)
  - ECC signing (P-256, P-384 with SHA-256/384)
  - Public key retrieval
- **HTTP/2 Optimizations**: TCP keepalive, adaptive windows, connection pooling
- **Error Handling**: No panics, proper gRPC status codes

### 2. gRPC Client (`src/bin/client.rs`)

The client tests both Echo and Crypto services with latency measurement.

#### Basic Usage

```bash
# Run client with default settings
make client

# Or run directly
./target/release/client
```

#### Configuration Options

| Environment Variable | Default Value | Description |
|---------------------|---------------|-------------|
| `SERVER_ADDR` | `127.0.0.1:50051` | Server address to connect to |
| `RUST_LOG` | `info` | Log level |

#### Examples

```bash
# Connect to custom server
SERVER_ADDR=127.0.0.1:8080 ./target/release/client

# Connect via VSOCK
SERVER_ADDR=vsock://2:50051 ./target/release/client

# Debug logging
RUST_LOG=debug ./target/release/client
```

#### Client Operations

The client automatically performs:
1. **Echo Tests**: Sends 4 test messages and measures latency
2. **Crypto Tests**: 
   - RSA PKCS#1 signing test
   - ECC P-256 signing test  
   - RSA public key retrieval test

### 3. Benchmark Tool (`src/bin/benchmark.rs`)

Comprehensive performance testing tool with detailed metrics and configurable load patterns.

#### Basic Usage

```bash
# Default benchmark (30s duration, 10 connections)
make benchmark

# Or run directly
./target/release/benchmark --duration 30s --connections 10
```

#### Command Line Options

| Option | Environment Variable | Default | Description |
|--------|---------------------|---------|-------------|
| `--connections NUM` | `CONNECTIONS`, `CONCURRENT_REQUESTS` | `10` | Number of concurrent connections |
| `--threads NUM` | `THREADS` | `4` | Number of worker threads |
| `--requests NUM` | `REQUESTS`, `TOTAL_REQUESTS` | `1000` | Total number of requests |
| `--rate RPS` | `RATE_LIMIT` | None | Requests per second limit |
| `--transport TYPE` | `TRANSPORT` | `tcp` | Transport type (`tcp` or `vsock`) |
| `--service TYPE` | `SERVICE`, `BENCHMARK_TYPE` | `echo` | Service type (`echo`, `crypto`, or `both`) |
| `--duration DURATION` | `DURATION` | None | Test duration (e.g., `30s`, `2m`, `1h`) |
| `--server ADDR` | `SERVER_ADDR` | `127.0.0.1:50051` | Server address |

#### Benchmark Examples

```bash
# Quick performance test
./target/release/benchmark --connections 50 --duration 10s

# High-load test
./target/release/benchmark --connections 100 --requests 10000 --service both

# Rate-limited test
./target/release/benchmark --rate 1000 --duration 60s

# VSOCK transport test
./target/release/benchmark --transport vsock --server vsock://2:50051 --duration 30s

# Crypto service only
./target/release/benchmark --service crypto --connections 20 --duration 15s

# Both services
./target/release/benchmark --service both --connections 50 --duration 30s
```

#### Pre-configured Benchmark Targets

```bash
# Light load test (10 connections, 5s)
make benchmark-light

# Medium load test (50 connections, 10s)  
make benchmark-medium

# Heavy load test (100 connections, 15s)
make benchmark-heavy

# Transport-specific tests
make benchmark-tcp
make benchmark-vsock

# Performance characteristic tests
make benchmark-latency      # Low concurrency, latency focus
make benchmark-throughput   # High concurrency, throughput focus
```

#### Benchmark Metrics

The benchmark tool provides comprehensive metrics:

- **Request Statistics**: Total, successful, failed requests
- **Success Rate**: Percentage of successful requests
- **Latency Metrics**: Average, minimum, maximum latency in microseconds
- **Throughput**: Requests per second
- **Duration**: Total test execution time

Example output:
```
=== Echo Service Benchmark Results ===
Total requests: 1000
Successful requests: 1000
Failed requests: 0
Success rate: 100.00%
Duration: 10.234s
Requests per second: 97.71
Average latency: 102.45 μs
Min latency: 89 μs
Max latency: 1205 μs
```

## Transport Support

### TCP Transport

Standard network transport for remote connections.

**Address Format**: `host:port`

Examples:
- `127.0.0.1:50051` (default)
- `0.0.0.0:8080`
- `192.168.1.100:50051`

### VSOCK Transport

Optimized for VM-to-host communication with minimal overhead.

**Address Format**: `vsock://cid:port`

Examples:
- `vsock://2:50051` (guest to host)
- `vsock://3:1234` (specific VM)

**Note**: VSOCK requires running in a VM environment with VSOCK support.

## Performance Characteristics

Based on testing, the system achieves:

- **Echo Service**: 2,000+ RPS with 1-3ms average latency
- **Crypto Service**: 1,900+ RPS with 1-2ms average latency  
- **Success Rate**: 100% under normal load conditions
- **Scalability**: Supports 100+ concurrent connections
- **Transport Parity**: Identical performance across TCP and VSOCK

## Environment Variables Reference

### Server Configuration

```bash
# Server binding
export SERVER_ADDR="127.0.0.1:50051"        # TCP
export SERVER_ADDR="vsock://2:50051"         # VSOCK

# Performance tuning
export WORKER_THREADS=8                      # Worker thread count

# Logging
export RUST_LOG="info"                       # Log level
```

### Client Configuration

```bash
# Server connection
export SERVER_ADDR="127.0.0.1:50051"        # Target server

# Logging
export RUST_LOG="debug"                      # Detailed logging
```

### Benchmark Configuration

```bash
# Load configuration
export CONNECTIONS=50                        # Concurrent connections
export REQUESTS=10000                        # Total requests
export RATE_LIMIT=1000                       # Requests per second

# Test configuration  
export TRANSPORT="tcp"                       # tcp or vsock
export SERVICE="both"                        # echo, crypto, or both
export DURATION="60"                         # Duration in seconds

# Server target
export SERVER_ADDR="127.0.0.1:50051"        # Target server
```

## Error Handling

The system implements robust error handling:

- **No Panics**: All operations return proper `Result` types
- **gRPC Status Codes**: Appropriate status codes for different error conditions
- **Graceful Degradation**: Continues operation when possible
- **Detailed Logging**: Comprehensive error information for debugging

## Development

### Project Structure

```
src/
├── lib.rs              # Shared types and utilities
├── transport.rs        # Transport abstraction layer
└── bin/
    ├── server.rs       # gRPC server implementation
    ├── client.rs       # gRPC client for testing
    └── benchmark.rs    # Performance benchmarking tool
```

### Building and Testing

```bash
# Development build
cargo build

# Release build (recommended for performance testing)
cargo build --release

# Run tests
cargo test

# Run with logging
RUST_LOG=debug cargo run --bin server
```

### Adding New Services

1. Define service in `proto/` directory
2. Update `build.rs` for proto compilation
3. Implement service in `src/bin/server.rs`
4. Add client calls in `src/bin/client.rs`
5. Update benchmark tool if needed

## Troubleshooting

### Common Issues

**Server won't start**:
- Check if port is already in use: `lsof -i :50051`
- Verify VSOCK support in VM environments
- Check firewall settings for custom ports

**Client connection fails**:
- Ensure server is running: `make server`
- Verify server address matches client configuration
- Check network connectivity for remote connections

**Poor performance**:
- Use release builds: `cargo build --release`
- Tune `WORKER_THREADS` based on CPU cores
- Adjust benchmark concurrency settings
- Monitor system resources during testing

**VSOCK issues**:
- Ensure running in VM with VSOCK support
- Check VM configuration for VSOCK enablement
- Verify CID (Context ID) values are correct

### Debugging

Enable debug logging for detailed information:

```bash
# Server debugging
RUST_LOG=debug ./target/release/server

# Client debugging  
RUST_LOG=debug ./target/release/client

# Benchmark debugging
RUST_LOG=debug ./target/release/benchmark --connections 1 --requests 10
```

## License

This project is part of a high-performance gRPC research implementation.