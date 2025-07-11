//! VSOCK transport implementation using tokio-vsock
//!
//! This module implements VSOCK transport as specified in PRD Task 6: VSOCK Transport Implementation
//! Only available on Unix platforms with the vsock feature enabled

#[cfg(feature = "vsock")]
mod vsock_impl {
    use crate::config::TransportType;
    use crate::error::{NetworkError, Result, TransportError};
    use crate::transport::{Connection, Listener, Transport};
    use async_trait::async_trait;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio_vsock::{VsockListener, VsockStream};

    /// VSOCK transport implementation
    #[derive(Debug)]
    pub struct VsockTransport;

    impl VsockTransport {
        /// Create a new VSOCK transport
        pub fn new() -> Self {
            Self
        }
    }

    #[async_trait]
    impl Transport for VsockTransport {
        async fn connect(&self, address: &str) -> Result<Box<dyn Connection>> {
            // Parse VSOCK address format: "cid:port"
            let (cid, port) = parse_vsock_address(address)?;
            
            let stream = VsockStream::connect(cid, port).await.map_err(|e| {
                NetworkError::ConnectionFailed {
                    message: format!("Failed to connect to VSOCK {}:{}: {}", cid, port, e),
                }
            })?;

            Ok(Box::new(VsockConnection::new(stream)))
        }

        async fn bind(&self, address: &str) -> Result<Box<dyn Listener>> {
            // Parse VSOCK address format: "cid:port"
            let (cid, port) = parse_vsock_address(address)?;
            
            let listener = VsockListener::bind(cid, port).await.map_err(|e| {
                TransportError::Configuration {
                    message: format!("Failed to bind VSOCK to {}:{}: {}", cid, port, e),
                }
            })?;

            Ok(Box::new(VsockListenerWrapper::new(listener)))
        }

        fn transport_type(&self) -> TransportType {
            TransportType::Vsock
        }
    }

    /// VSOCK connection wrapper
    #[derive(Debug)]
    pub struct VsockConnection {
        stream: VsockStream,
    }

    impl VsockConnection {
        /// Create a new VSOCK connection
        pub fn new(stream: VsockStream) -> Self {
            Self { stream }
        }
    }

    #[async_trait]
    impl Connection for VsockConnection {
        async fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
            self.stream.read(buf).await.map_err(|e| {
                NetworkError::ConnectionLost {
                    reason: format!("VSOCK read error: {}", e),
                }
                .into()
            })
        }

        async fn write(&mut self, buf: &[u8]) -> Result<usize> {
            self.stream.write(buf).await.map_err(|e| {
                NetworkError::ConnectionLost {
                    reason: format!("VSOCK write error: {}", e),
                }
                .into()
            })
        }

        async fn close(&mut self) -> Result<()> {
            self.stream.shutdown().await.map_err(|e| {
                TransportError::Vsock {
                    message: format!("Failed to close VSOCK connection: {}", e),
                }
                .into()
            })
        }
    }

    /// VSOCK listener wrapper
    #[derive(Debug)]
    pub struct VsockListenerWrapper {
        listener: VsockListener,
    }

    impl VsockListenerWrapper {
        /// Create a new VSOCK listener wrapper
        pub fn new(listener: VsockListener) -> Self {
            Self { listener }
        }
    }

    #[async_trait]
    impl Listener for VsockListenerWrapper {
        async fn accept(&mut self) -> Result<Box<dyn Connection>> {
            let (stream, _addr) = self.listener.accept().await.map_err(|e| {
                TransportError::Vsock {
                    message: format!("Failed to accept VSOCK connection: {}", e),
                }
            })?;

            Ok(Box::new(VsockConnection::new(stream)))
        }

        async fn close(&mut self) -> Result<()> {
            // VSOCK listener doesn't have an explicit close method
            // The listener will be closed when dropped
            Ok(())
        }
    }

    /// Parse VSOCK address in format "cid:port"
    fn parse_vsock_address(address: &str) -> Result<(u32, u32)> {
        let parts: Vec<&str> = address.split(':').collect();
        if parts.len() != 2 {
            return Err(NetworkError::InvalidAddress {
                address: address.to_string(),
            }
            .into());
        }

        let cid = parts[0].parse::<u32>().map_err(|_| {
            NetworkError::InvalidAddress {
                address: address.to_string(),
            }
        })?;

        let port = parts[1].parse::<u32>().map_err(|_| {
            NetworkError::InvalidAddress {
                address: address.to_string(),
            }
        })?;

        Ok((cid, port))
    }

    impl Default for VsockTransport {
        fn default() -> Self {
            Self::new()
        }
    }
}

#[cfg(feature = "vsock")]
pub use vsock_impl::VsockTransport;

#[cfg(not(feature = "vsock"))]
/// Placeholder for when VSOCK feature is not enabled
pub struct VsockTransport;

#[cfg(not(feature = "vsock"))]
impl VsockTransport {
    /// Create a new VSOCK transport (placeholder)
    pub fn new() -> Self {
        Self
    }
}