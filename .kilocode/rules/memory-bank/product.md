# High-Performance gRPC Client/Server System

## Purpose

This project implements a high-performance, minimal latency gRPC client and server system in Rust that supports both TCP and VSOCK transports with cryptographic services. The system is designed to achieve maximum performance with sub-millisecond latencies and support for 100K+ requests per second.

## Problems It Solves

1. **High-Performance gRPC Communication**: Provides optimized gRPC services with minimal latency overhead
2. **Dual Transport Support**: Enables both standard TCP networking and VM-to-host VSOCK communication
3. **Cryptographic Services**: Offers RSA and ECC signing capabilities via gRPC
4. **Performance Benchmarking**: Includes comprehensive benchmarking tools for performance validation
5. **Production-Ready Error Handling**: Implements robust error handling without panics or unwraps

## How It Works

### Core Services

**Echo Service**: 
- Receives payload and echoes it back with timestamps
- Logs detailed request/response information
- Optimized for minimal processing overhead
- Measures server-side and total latency

**Crypto Service**:
- Supports RSA and ECC key generation at server startup
- Provides signing operations (RSA PKCS#1, RSA PSS, ECDSA P-256/P-384)
- Offers public key retrieval functionality
- Client specifies key type and algorithm for each request

### Transport Layer

**TCP Transport**: Standard network transport for remote connections with connection pooling optimization

**VSOCK Transport**: Optimized for VM-to-host communication with minimal overhead

### Performance Features

- Configurable worker threads and connection pooling
- HTTP/2 optimizations (keepalive, adaptive windows, stream limits)
- Comprehensive benchmarking with latency/throughput measurements
- Rate limiting and duration-based testing support

## User Experience Goals

1. **Developer Experience**: Simple command-line tools for server, client, and benchmarking
2. **Performance Transparency**: Detailed logging and metrics for performance analysis
3. **Flexibility**: Support for both TCP and VSOCK transports seamlessly
4. **Reliability**: No panics or unwraps in production code
5. **Ease of Use**: Minimal configuration with sensible defaults
6. **Comprehensive Testing**: End-to-end integration tests for all functionality

## Success Metrics

- **Latency**: Sub-millisecond response times for echo operations (currently achieving 1-3ms)
- **Throughput**: Target 100K+ requests per second based on hardware capabilities
- **Reliability**: 100% success rate with proper error handling
- **Compatibility**: Works across both TCP and VSOCK transports identically
- **Performance**: Optimized for both single and multi-threaded scenarios