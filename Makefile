# Makefile for the grpc-performance-rs project

.PHONY: all build test server client benchmark clean help setup

# Variables
TARGET_DIR := target/release

# Default target
all: build

# Setup development environment
setup:
	apt-get update && apt-get install -y protobuf-compiler

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
	@echo "Executing performance benchmarks..."
	@$(TARGET_DIR)/benchmark

# Clean build artifacts
clean:
	@echo "Cleaning build artifacts..."
	@cargo clean

# Display help information about available targets
help:
	@echo "Usage: make [target]"
	@echo ""
	@echo "Targets:"
	@echo "  all       Build all binaries (default)"
	@echo "  build     Build all binaries in release mode"
	@echo "  test      Run integration tests"
	@echo "  server    Start the gRPC server"
	@echo "  client    Run the sample client"
	@echo "  benchmark Execute performance benchmarks"
	@echo "  clean     Clean build artifacts"
	@echo "  help      Display this help message"