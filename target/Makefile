# Makefile for the grpc-performance-rs project

.PHONY: all build test server client benchmark benchmark-light benchmark-medium benchmark-heavy benchmark-tcp benchmark-vsock benchmark-latency benchmark-throughput clean help setup

# Variables
TARGET_DIR := target/release

# Default target
all: build

# Setup development environment
setup:
	apt-get update && apt-get install -y protobuf-compiler net-tools procps lsof

# Build all binaries in release mode
build:
	@echo "Building all binaries in release mode..."
	@cargo build --release --bins

# Run integration tests
test:
	@echo "Running integration tests..."
	@cargo test --release

# Start the gRPC server
# Depends on 'build' to ensure the binary is up-to-date
server: build
	@echo "Starting gRPC server..."
	@$(TARGET_DIR)/server

# Run the sample client
# Depends on 'build' to ensure the binary is up-to-date
client: build
	@echo "Running sample client..."
	@$(TARGET_DIR)/client

# Execute performance benchmarks
# Depends on 'build' to ensure the binary is up-to-date
benchmark: build
	@echo "Executing default performance benchmark..."
	@$(TARGET_DIR)/benchmark --duration 30s --connections 10

# Additional benchmark targets
benchmark-light: build
	@echo "Running light benchmark..."
	@$(TARGET_DIR)/benchmark --connections 10 --duration 5s --service echo

benchmark-medium: build
	@echo "Running medium benchmark..."
	@$(TARGET_DIR)/benchmark --connections 50 --duration 10s --service both

benchmark-heavy: build
	@echo "Running heavy benchmark..."
	@$(TARGET_DIR)/benchmark --connections 100 --duration 15s --service both

benchmark-tcp: build
	@echo "Running TCP transport benchmark..."
	@$(TARGET_DIR)/benchmark --transport tcp --duration 5s --connections 50

benchmark-vsock: build
	@echo "Running VSOCK transport benchmark..."
	@$(TARGET_DIR)/benchmark --transport vsock --server 2:1234 --duration 60s --connections 50

benchmark-latency: build
	@echo "Running latency benchmark..."
	@$(TARGET_DIR)/benchmark --connections 1 --requests 1000 --service echo

benchmark-throughput: build
	@echo "Running throughput benchmark..."
	@$(TARGET_DIR)/benchmark --connections 100 --rate 1000 --duration 60s

# Clean build artifacts
clean:
	@echo "Cleaning build artifacts..."
	@cargo clean

# Display help information about available targets
help:
	@echo "Usage: make [target]"
	@echo ""
	@echo "Build Targets:"
	@echo "  all       Build all binaries (default)"
	@echo "  build     Build all binaries in release mode"
	@echo "  clean     Clean build artifacts"
	@echo ""
	@echo "Runtime Targets:"
	@echo "  server    Start the gRPC server"
	@echo "  client    Run the sample client"
	@echo "  test      Run integration tests"
	@echo ""
	@echo "Benchmark Targets:"
	@echo "  benchmark           Execute default performance benchmark"
	@echo "  benchmark-light     Light load test (10 connections, 30s)"
	@echo "  benchmark-medium    Medium load test (50 connections, 60s)"
	@echo "  benchmark-heavy     Heavy load test (100 connections, 120s)"
	@echo "  benchmark-tcp       TCP transport benchmark"
	@echo "  benchmark-vsock     VSOCK transport benchmark"
	@echo "  benchmark-latency   Latency measurement test"
	@echo "  benchmark-throughput Throughput measurement test"
	@echo ""
	@echo "  help      Display this help message"