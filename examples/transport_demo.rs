//! Demonstration of the transport abstraction layer

use grpc_performance_rs::transport::{TransportConfig, TransportFactory};
use std::str::FromStr;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Parse different transport configurations
    let tcp_config = TransportConfig::from_str("127.0.0.1:50051")?;
    let vsock_config = TransportConfig::from_str("vsock://2:50051")?;

    println!("TCP Config: {} (port: {})", tcp_config, tcp_config.port());
    println!("VSOCK Config: {} (port: {})", vsock_config, vsock_config.port());

    println!("TCP transport name: {}", TransportFactory::transport_name(&tcp_config));
    println!("VSOCK transport name: {}", TransportFactory::transport_name(&vsock_config));

    // Note: Actual binding/connecting would require appropriate permissions and setup
    println!("Transport abstraction layer ready for use!");

    Ok(())
}