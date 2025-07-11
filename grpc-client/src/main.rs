//! gRPC Client Binary
//!
//! High-performance gRPC client with cryptographic operations
//! Supports both TCP and VSOCK transports

use clap::Parser;
use grpc_shared::config::ClientConfig;
use std::path::PathBuf;
use std::fs;

mod client;
mod config;
mod error;

use client::GrpcSigningClient;

/// Command line arguments for the gRPC client
#[derive(Parser, Debug)]
#[command(name = "grpc-client")]
#[command(about = "High-performance gRPC client with cryptographic operations")]
struct Args {
    /// Configuration file path
    #[arg(short, long, default_value = "client-config.toml")]
    config: PathBuf,

    /// Server address
    #[arg(short, long)]
    server_address: Option<String>,

    /// Data to sign (for testing)
    #[arg(short, long, default_value = "Hello, World!")]
    data: String,

    /// Log level
    #[arg(short, long, default_value = "info")]
    log_level: String,

    /// Run benchmark mode
    #[arg(long)]
    benchmark: bool,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    // Initialize logging
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or(&args.log_level))
        .init();

    log::info!("Starting gRPC signing client");

    // Load configuration
    let mut config = load_config(&args.config).unwrap_or_else(|_| {
        log::warn!("Failed to load config file, using defaults");
        ClientConfig::default()
    });

    // Override config with command line arguments
    if let Some(server_address) = args.server_address {
        config.server_address = server_address;
    }

    log::info!("Client configuration: {:?}", config);

    // Create the client
    let mut client = GrpcSigningClient::new(config);

    if args.benchmark {
        log::info!("Running in benchmark mode");
        run_benchmark(&mut client).await?;
    } else {
        log::info!("Running single signing request");
        run_single_request(&mut client, &args.data).await?;
    }

    log::info!("Client operation complete");
    Ok(())
}

/// Load client configuration from file
fn load_config(config_path: &PathBuf) -> anyhow::Result<ClientConfig> {
    log::info!("Loading configuration from: {:?}", config_path);
    
    // Start with default configuration
    let mut config = ClientConfig::default();
    
    // Load from TOML file if it exists
    if config_path.exists() {
        log::info!("Found configuration file: {:?}", config_path);
        let toml_content = fs::read_to_string(config_path)?;
        config = toml::from_str(&toml_content)?;
        log::info!("Configuration loaded from file successfully");
    } else {
        log::warn!("Configuration file not found: {:?}, using defaults", config_path);
    }
    
    // Apply environment variable overrides
    if let Ok(server_address) = std::env::var("GRPC_CLIENT_SERVER_ADDRESS") {
        log::info!("Overriding server_address from environment: {}", server_address);
        config.server_address = server_address;
    }
    
    if let Ok(transport) = std::env::var("GRPC_CLIENT_TRANSPORT") {
        log::info!("Overriding transport from environment: {}", transport);
        match transport.to_lowercase().as_str() {
            "tcp" => config.transport = grpc_shared::TransportType::Tcp,
            #[cfg(unix)]
            "vsock" => config.transport = grpc_shared::TransportType::Vsock,
            _ => log::warn!("Unknown transport type in environment: {}", transport),
        }
    }
    
    log::info!("Configuration loaded successfully");
    log::debug!("Final configuration: {:?}", config);
    
    Ok(config)
}

/// Run a single signing request
async fn run_single_request(client: &mut GrpcSigningClient, data: &str) -> anyhow::Result<()> {
    log::info!("Connecting to server");
    client.connect().await?;

    // First, check server health
    log::info!("Checking server health");
    let health_response = client.health_check(Some("signing")).await?;
    log::info!("Server health status: {:?}", health_response.status);

    // Generate a test key if needed
    let key_id = "test-key-001";
    log::info!("Generating test key: {}", key_id);
    let key_response = client.generate_key(
        key_id,
        grpc_shared::KeyType::EccP256,
        "Test key for signing demonstration"
    ).await?;
    log::info!("Key generation response: success={}, key_id={}",
               key_response.success,
               key_response.key_info.as_ref().map(|k| k.key_id.as_str()).unwrap_or("unknown"));

    // Perform the signing request
    log::info!("Sending signing request for data: {}", data);
    let sign_response = client.sign(
        data.as_bytes(),
        key_id,
        grpc_shared::KeyType::EccP256,
        grpc_shared::SigningAlgorithm::EcdsaP256Sha256,
    ).await?;

    log::info!("Signing completed successfully!");
    log::info!("Signature length: {} bytes", sign_response.signature.len());
    log::info!("Processing time: {} μs", sign_response.processing_time_us);

    // Verify the signature
    log::info!("Verifying signature");
    let verify_response = client.verify(
        data.as_bytes(),
        &sign_response.signature,
        key_id,
        grpc_shared::SigningAlgorithm::EcdsaP256Sha256,
        crate::config::HashAlgorithm::Sha256,
    ).await?;

    log::info!("Signature verification: valid={}", verify_response.valid);
    if verify_response.valid {
        log::info!("✅ End-to-end signing and verification successful!");
    } else {
        log::error!("❌ Signature verification failed!");
    }

    // List available keys
    log::info!("Listing available keys");
    let list_response = client.list_keys(
        Some(grpc_shared::KeyType::EccP256),
        Some(true)
    ).await?;
    log::info!("Found {} keys", list_response.keys.len());
    for key in &list_response.keys {
        log::info!("  Key: {} (type: {:?}, created: {}, active: {})",
                   key.key_id, key.key_type, key.created_at, key.is_active);
    }

    log::info!("Disconnecting from server");
    client.disconnect().await?;

    Ok(())
}

/// Run benchmark tests
async fn run_benchmark(client: &mut GrpcSigningClient) -> anyhow::Result<()> {
    log::info!("Starting benchmark tests");
    client.connect().await?;

    // Check server health first
    log::info!("Checking server health before benchmark");
    let health_response = client.health_check(Some("signing")).await?;
    log::info!("Server health status: {:?}", health_response.status);

    // Generate a benchmark key
    let key_id = "benchmark-key-001";
    log::info!("Generating benchmark key: {}", key_id);
    let key_response = client.generate_key(
        key_id,
        grpc_shared::KeyType::EccP256,
        "Benchmark key for performance testing"
    ).await?;
    log::info!("Key generation response: success={}", key_response.success);

    // Benchmark parameters
    let test_data = b"Benchmark test data for performance measurement";
    let num_requests = 100;
    let warmup_requests = 10;

    log::info!("Running warmup phase ({} requests)", warmup_requests);
    for i in 0..warmup_requests {
        let _ = client.sign(
            test_data,
            key_id,
            grpc_shared::KeyType::EccP256,
            grpc_shared::SigningAlgorithm::EcdsaP256Sha256,
        ).await?;
        
        if (i + 1) % 5 == 0 {
            log::info!("Warmup progress: {}/{}", i + 1, warmup_requests);
        }
    }

    log::info!("Starting benchmark phase ({} requests)", num_requests);
    let start_time = std::time::Instant::now();
    let mut total_processing_time = 0u64;
    let mut min_time = u64::MAX;
    let mut max_time = 0u64;
    let mut processing_times = Vec::with_capacity(num_requests);

    for i in 0..num_requests {
        let sign_response = client.sign(
            test_data,
            key_id,
            grpc_shared::KeyType::EccP256,
            grpc_shared::SigningAlgorithm::EcdsaP256Sha256,
        ).await?;

        let processing_time = sign_response.processing_time_us;
        
        processing_times.push(processing_time);
        total_processing_time += processing_time;
        min_time = min_time.min(processing_time);
        max_time = max_time.max(processing_time);

        if (i + 1) % 25 == 0 {
            log::info!("Benchmark progress: {}/{} (avg: {:.2}μs)",
                       i + 1, num_requests, total_processing_time as f64 / (i + 1) as f64);
        }
    }

    let total_duration = start_time.elapsed();
    
    // Calculate statistics
    processing_times.sort_unstable();
    let avg_processing_time = total_processing_time as f64 / num_requests as f64;
    let p50 = processing_times[num_requests / 2];
    let p95 = processing_times[(num_requests as f64 * 0.95) as usize];
    let p99 = processing_times[(num_requests as f64 * 0.99) as usize];
    let throughput = num_requests as f64 / total_duration.as_secs_f64();

    // Convert microseconds to milliseconds for display
    let avg_processing_time_ms = avg_processing_time / 1000.0;
    let min_time_ms = min_time as f64 / 1000.0;
    let max_time_ms = max_time as f64 / 1000.0;
    let p50_ms = p50 as f64 / 1000.0;
    let p95_ms = p95 as f64 / 1000.0;
    let p99_ms = p99 as f64 / 1000.0;

    // Print benchmark results
    log::info!("🚀 Benchmark Results:");
    log::info!("  Total requests: {}", num_requests);
    log::info!("  Total duration: {:.2}s", total_duration.as_secs_f64());
    log::info!("  Throughput: {:.2} RPS", throughput);
    log::info!("  Processing time statistics:");
    log::info!("    Average: {:.2}ms", avg_processing_time_ms);
    log::info!("    Minimum: {:.2}ms", min_time_ms);
    log::info!("    Maximum: {:.2}ms", max_time_ms);
    log::info!("    P50: {:.2}ms", p50_ms);
    log::info!("    P95: {:.2}ms", p95_ms);
    log::info!("    P99: {:.2}ms", p99_ms);

    // Performance targets validation
    let target_throughput = 1000.0; // 1K RPS target for benchmark
    let target_p99_latency = 10.0; // 10ms P99 target
    
    log::info!("🎯 Performance Target Validation:");
    if throughput >= target_throughput {
        log::info!("  ✅ Throughput: {:.2} RPS >= {:.2} RPS (PASS)", throughput, target_throughput);
    } else {
        log::warn!("  ❌ Throughput: {:.2} RPS < {:.2} RPS (FAIL)", throughput, target_throughput);
    }
    
    if p99_ms <= target_p99_latency {
        log::info!("  ✅ P99 Latency: {:.2}ms <= {:.1}ms (PASS)", p99_ms, target_p99_latency);
    } else {
        log::warn!("  ❌ P99 Latency: {:.2}ms > {:.1}ms (FAIL)", p99_ms, target_p99_latency);
    }

    log::info!("Benchmark tests completed");
    client.disconnect().await?;
    Ok(())
}