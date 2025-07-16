use std::time::Duration;
use http::Uri;
use log::{debug, info};
use tonic::transport::{Channel, Endpoint};
use hyper_util::rt::tokio::TokioIo;
use crate::{AppError, AppResult};
use crate::transport::{Connection, TransportConfig, TransportError, TransportFactory};

/// Create a custom channel using our transport abstraction
pub async fn create_transport_channel(transport_config: &TransportConfig) -> AppResult<Channel> {
    info!("Creating transport channel for {}", transport_config);

    match transport_config {
        TransportConfig::Tcp(addr) => {
            // For TCP, use tonic's built-in channel creation
            debug!("Creating TCP channel to {}", addr);
            let endpoint = Channel::from_shared(format!("http://{}", addr))
                .map_err(|e| AppError::TransportLayer(TransportError::InvalidAddress(format!("Invalid TCP address: {}", e))))?;

            let channel = endpoint
                .tcp_keepalive(Some(Duration::from_secs(5))) // More aggressive keepalive
                .tcp_nodelay(true)
                .http2_keep_alive_interval(Duration::from_secs(5)) // Faster detection
                .keep_alive_timeout(Duration::from_secs(3)) // Faster timeout
                .keep_alive_while_idle(true) // Keep connections alive
                .initial_stream_window_size(Some(32 * 1024 * 1024)) // 32MB for higher throughput
                .initial_connection_window_size(Some(32 * 1024 * 1024)) // 32MB for higher throughput
                .http2_adaptive_window(true) // Enable adaptive windowing
                .timeout(Duration::from_secs(15)) // Reduced timeout
                .connect_timeout(Duration::from_secs(5)) // Faster connection establishment
                .connect()
                .await
                .map_err(|e| AppError::TransportLayer(TransportError::ConnectionFailed(format!("Failed to connect via TCP: {}", e))))?;

            Ok(channel)
        }
        TransportConfig::Vsock { cid, port } => {
            // For VSOCK, use our transport factory with a custom connector
            debug!("Creating VSOCK channel to CID {} port {}", cid, port);

            let config = transport_config.clone();
            let connector = tower::service_fn(move |_: Uri| {
                let config = config.clone();
                async move {
                    let connection = TransportFactory::connect(&config).await
                        .map_err(|e| std::io::Error::new(std::io::ErrorKind::ConnectionRefused, e.to_string()))?;

                    match connection {
                        Connection::Vsock(stream) => Ok(TokioIo::new(stream)),
                        Connection::Tcp(_) => Err(std::io::Error::new(
                            std::io::ErrorKind::InvalidData,
                            "Expected VSOCK connection but got TCP"
                        )),
                    }
                }
            });

            let endpoint = Endpoint::from_static("http://[::]:50051");
            let channel = endpoint
                .tcp_keepalive(Some(Duration::from_secs(5))) // More aggressive keepalive
                .tcp_nodelay(true)
                .http2_keep_alive_interval(Duration::from_secs(5)) // Faster detection
                .keep_alive_timeout(Duration::from_secs(3)) // Faster timeout
                .keep_alive_while_idle(true) // Keep connections alive
                .initial_stream_window_size(Some(32 * 1024 * 1024)) // 32MB for higher throughput
                .initial_connection_window_size(Some(32 * 1024 * 1024)) // 32MB for higher throughput
                .http2_adaptive_window(true) // Enable adaptive windowing
                .timeout(Duration::from_secs(15)) // Reduced timeout
                .connect_timeout(Duration::from_secs(5)) // Faster connection establishment
                .connect_with_connector(connector)
                .await
                .map_err(|e| AppError::TransportLayer(TransportError::ConnectionFailed(format!("Failed to connect via VSOCK: {}", e))))?;

            Ok(channel)
        }
    }
}
