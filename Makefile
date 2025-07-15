NAME=$(basename $(notdir ${CURDIR}))
# Makefile for the grpc-performance-rs project

.PHONY: all build test server stop-server client benchmark benchmark-light benchmark-medium benchmark-heavy benchmark-tcp benchmark-vsock benchmark-latency benchmark-throughput clean help setup

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

# Start the gRPC server in background
# Depends on 'build' to ensure the binary is up-to-date
server: build
	@echo "Stopping any existing server processes..."
	@pkill -f "$(TARGET_DIR)/server" 2>/dev/null || true
	@echo "Starting gRPC server in background..."
	@$(TARGET_DIR)/server &
	@sleep 2
	@echo "Server is running in background (use 'make stop-server' to stop)"

# Stop the gRPC server
stop-server:
	@echo "Stopping gRPC server..."
	@pkill -f "$(TARGET_DIR)/server" 2>/dev/null || echo "No server process found"

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
	@$(TARGET_DIR)/benchmark --transport vsock --server ${vsock_addr} --duration 10s --connections 1

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

# VSOCK
e_path=target/${NAME}.eif
e_cid=15
e_cpu=2
e_mem=512
vsock_addr=vsock://${e_cid}:5001
enclave-image-build:
	docker build -t ${NAME} -f Dockerfile .
enclave-image-run: enclave-image-build
	docker run --rm -it --cap-add=NET_ADMIN ${NAME}
enclave-build: enclave-image-build
	nitro-cli build-enclave --docker-uri ${NAME} --output-file ${e_path}
	ls -lah ${e_path}
enclave-run: enclave-terminate
	sudo nitro-cli run-enclave --eif-path ${e_path} --cpu-count ${e_cpu} --memory ${e_mem} \
			--enclave-cid ${e_cid} --enclave-name ${NAME} --debug-mode --attach-console
enclave-console:
	sudo nitro-cli console --enclave-name ${NAME}
enclave-terminate:
	sudo nitro-cli terminate-enclave --all || true
client-enclave:
	SERVER_ADDR=${vsock_addr} make client