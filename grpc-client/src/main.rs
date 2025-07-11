//! gRPC Client Binary
//!
//! High-performance gRPC client with cryptographic operations
//! Supports both TCP and VSOCK transports

use clap::Parser;
use grpc_shared::config::ClientConfig;
use std::path::PathBuf;

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
    // TODO: Implement configuration loading from file
    // For now, return default configuration
    log::info!("Loading configuration from: {:?}", config_path);
    Ok(ClientConfig::default())
}

/// Run a single signing request
async fn run_single_request(client: &mut GrpcSigningClient, data: &str) -> anyhow::Result<()> {
    log::info!("Connecting to server");
    client.connect().await?;

    log::info!("Sending signing request for data: {}", data);
    // TODO: Implement actual signing request
    
    log::info!("Disconnecting from server");
    client.disconnect().await?;

    Ok(())
}

/// Run benchmark tests
async fn run_benchmark(client: &mut GrpcSigningClient) -> anyhow::Result<()> {
    log::info!("Starting benchmark tests");
    client.connect().await?;

    // TODO: Implement benchmark logic
    log::info!("Benchmark tests completed");

    client.disconnect().await?;
    Ok(())
}