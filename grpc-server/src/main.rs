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

    // Create the server
    let mut server = GrpcSigningServer::new(config).await?;
    
    // Create shutdown channel
    let (shutdown_tx, shutdown_rx) = GrpcSigningServer::create_shutdown_channel();
    
    // Handle shutdown gracefully
    tokio::select! {
        result = server.start_with_shutdown(shutdown_rx) => {
            if let Err(e) = result {
                log::error!("Server error: {}", e);
                return Err(e.into());
            }
        }
        _ = tokio::signal::ctrl_c() => {
            log::info!("Received Ctrl+C, initiating graceful shutdown");
            if let Err(_) = shutdown_tx.send(()) {
                log::error!("Failed to send shutdown signal");
            }
            // Give the server a moment to shut down gracefully
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        }
    }

    log::info!("Server shutdown complete");
    Ok(())
}

/// Load server configuration from file
fn load_config(config_path: &PathBuf) -> anyhow::Result<ServerConfig> {
    log::info!("Loading configuration from: {:?}", config_path);
    
    if !config_path.exists() {
        log::warn!("Configuration file does not exist: {:?}", config_path);
        return Ok(ServerConfig::default());
    }

    let config_content = std::fs::read_to_string(config_path)
        .map_err(|e| anyhow::anyhow!("Failed to read config file: {}", e))?;

    let config: ServerConfig = toml::from_str(&config_content)
        .map_err(|e| anyhow::anyhow!("Failed to parse config file: {}", e))?;

    log::info!("Configuration loaded successfully");
    Ok(config)
}