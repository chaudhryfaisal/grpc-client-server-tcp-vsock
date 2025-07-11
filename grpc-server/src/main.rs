//! gRPC Server Binary
//!
//! High-performance gRPC server with cryptographic operations
//! Supports both TCP and VSOCK transports

use clap::Parser;
use grpc_shared::{config::ServerConfig, server::GrpcSigningServer};
use std::path::PathBuf;

/// Command line arguments for the gRPC server
#[derive(Parser, Debug)]
#[command(name = "grpc-server")]
#[command(about = "High-performance gRPC server with cryptographic operations")]
struct Args {
    /// Configuration file path
    #[arg(short, long, default_value = "server-config.toml")]
    config: PathBuf,

    /// Bind address
    #[arg(short, long)]
    bind_address: Option<String>,

    /// Port number
    #[arg(short, long)]
    port: Option<u16>,

    /// Log level
    #[arg(short, long, default_value = "info")]
    log_level: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    // Initialize logging
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or(&args.log_level))
        .init();

    log::info!("Starting gRPC signing server");

    // Load configuration
    let mut config = load_config(&args.config).unwrap_or_else(|_| {
        log::warn!("Failed to load config file, using defaults");
        ServerConfig::default()
    });

    // Override config with command line arguments
    if let Some(bind_address) = args.bind_address {
        config.bind_address = bind_address;
    }
    if let Some(port) = args.port {
        config.port = port;
    }

    log::info!("Server configuration: {:?}", config);

    // Create and start the server
    let server = GrpcSigningServer::new(config);
    
    // Handle shutdown gracefully
    tokio::select! {
        result = server.start() => {
            if let Err(e) = result {
                log::error!("Server error: {}", e);
                return Err(e.into());
            }
        }
        _ = tokio::signal::ctrl_c() => {
            log::info!("Received shutdown signal");
            server.stop().await?;
        }
    }

    log::info!("Server shutdown complete");
    Ok(())
}

/// Load server configuration from file
fn load_config(config_path: &PathBuf) -> anyhow::Result<ServerConfig> {
    // TODO: Implement configuration loading from file
    // For now, return default configuration
    log::info!("Loading configuration from: {:?}", config_path);
    Ok(ServerConfig::default())
}