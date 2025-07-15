//! Transport abstraction layer for gRPC client/server supporting TCP and VSOCK.

use std::fmt;
use std::net::SocketAddr;
use std::str::FromStr;
use std::pin::Pin;
use std::task::{Context, Poll};
use tokio::io::{AsyncRead, AsyncWrite, ReadBuf};
use tokio::net::{TcpListener, TcpStream};
use tokio_vsock::{VsockListener, VsockStream};
use vsock::VMADDR_CID_ANY;

/// Configuration for different transport types.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TransportConfig {
    /// TCP transport with socket address
    Tcp(SocketAddr),
    /// VSOCK transport with context ID and port
    Vsock { cid: u32, port: u32 },
}

impl TransportConfig {
    /// Get the port number for this transport configuration
    pub fn port(&self) -> u32 {
        match self {
            TransportConfig::Tcp(addr) => addr.port() as u32,
            TransportConfig::Vsock { port, .. } => *port,
        }
    }

    /// Check if this is a TCP transport
    pub fn is_tcp(&self) -> bool {
        matches!(self, TransportConfig::Tcp(_))
    }

    /// Check if this is a VSOCK transport
    pub fn is_vsock(&self) -> bool {
        matches!(self, TransportConfig::Vsock { .. })
    }
}

impl fmt::Display for TransportConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TransportConfig::Tcp(addr) => write!(f, "{}", addr),
            TransportConfig::Vsock { cid, port } => write!(f, "vsock://{}:{}", cid, port),
        }
    }
}

impl FromStr for TransportConfig {
    type Err = TransportError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Some(vsock_addr) = s.strip_prefix("vsock://") {
            // Parse VSOCK address: vsock://cid:port
            let parts: Vec<&str> = vsock_addr.split(':').collect();
            if parts.len() != 2 {
                return Err(TransportError::InvalidAddress(format!(
                    "VSOCK address must be in format 'vsock://cid:port', got: {}",
                    s
                )));
            }

            let cid = parts[0].parse::<u32>().map_err(|_| {
                TransportError::InvalidAddress(format!("Invalid CID in VSOCK address: {}", parts[0]))
            })?;

            let port = parts[1].parse::<u32>().map_err(|_| {
                TransportError::InvalidAddress(format!("Invalid port in VSOCK address: {}", parts[1]))
            })?;

            Ok(TransportConfig::Vsock { cid, port })
        } else {
            // Parse TCP address: host:port
            let addr = s.parse::<SocketAddr>().map_err(|e| {
                TransportError::InvalidAddress(format!("Invalid TCP address '{}': {}", s, e))
            })?;
            Ok(TransportConfig::Tcp(addr))
        }
    }
}

/// Errors that can occur during transport operations.
#[derive(Debug, thiserror::Error)]
pub enum TransportError {
    #[error("Invalid address format: {0}")]
    InvalidAddress(String),

    #[error("TCP transport error: {0}")]
    Tcp(#[from] std::io::Error),

    #[error("VSOCK transport error: {0}")]
    Vsock(String),

    #[error("Transport not supported: {0}")]
    NotSupported(String),

    #[error("Connection failed: {0}")]
    ConnectionFailed(String),

    #[error("Bind failed: {0}")]
    BindFailed(String),
}

/// Unified connection type that can represent either TCP or VSOCK connections.
#[derive(Debug)]
pub enum Connection {
    Tcp(TcpStream),
    Vsock(VsockStream),
}

impl Connection {
    /// Get the remote address of this connection as a string
    pub fn remote_addr(&self) -> Result<String, TransportError> {
        match self {
            Connection::Tcp(stream) => {
                stream.peer_addr()
                    .map(|addr| addr.to_string())
                    .map_err(TransportError::Tcp)
            }
            Connection::Vsock(stream) => {
                stream.peer_addr()
                    .map(|addr| format!("vsock://{}:{}", addr.cid(), addr.port()))
                    .map_err(|e| TransportError::Vsock(e.to_string()))
            }
        }
    }

    /// Get the local address of this connection as a string
    pub fn local_addr(&self) -> Result<String, TransportError> {
        match self {
            Connection::Tcp(stream) => {
                stream.local_addr()
                    .map(|addr| addr.to_string())
                    .map_err(TransportError::Tcp)
            }
            Connection::Vsock(stream) => {
                stream.local_addr()
                    .map(|addr| format!("vsock://{}:{}", addr.cid(), addr.port()))
                    .map_err(|e| TransportError::Vsock(e.to_string()))
            }
        }
    }
}

impl AsyncRead for Connection {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<std::io::Result<()>> {
        match &mut *self {
            Connection::Tcp(stream) => Pin::new(stream).poll_read(cx, buf),
            Connection::Vsock(stream) => Pin::new(stream).poll_read(cx, buf),
        }
    }
}

impl AsyncWrite for Connection {
    fn poll_write(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<Result<usize, std::io::Error>> {
        match &mut *self {
            Connection::Tcp(stream) => Pin::new(stream).poll_write(cx, buf),
            Connection::Vsock(stream) => Pin::new(stream).poll_write(cx, buf),
        }
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), std::io::Error>> {
        match &mut *self {
            Connection::Tcp(stream) => Pin::new(stream).poll_flush(cx),
            Connection::Vsock(stream) => Pin::new(stream).poll_flush(cx),
        }
    }

    fn poll_shutdown(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), std::io::Error>> {
        match &mut *self {
            Connection::Tcp(stream) => Pin::new(stream).poll_shutdown(cx),
            Connection::Vsock(stream) => Pin::new(stream).poll_shutdown(cx),
        }
    }
}

/// Unified listener type that can represent either TCP or VSOCK listeners.
#[derive(Debug)]
pub enum Listener {
    Tcp(TcpListener),
    Vsock(VsockListener),
}

impl Listener {
    /// Accept a new connection from this listener
    pub async fn accept(&mut self) -> Result<Connection, TransportError> {
        match self {
            Listener::Tcp(listener) => {
                let (stream, _) = listener.accept().await.map_err(TransportError::Tcp)?;
                Ok(Connection::Tcp(stream))
            }
            Listener::Vsock(listener) => {
                let (stream, _) = listener.accept().await
                    .map_err(|e| TransportError::Vsock(e.to_string()))?;
                Ok(Connection::Vsock(stream))
            }
        }
    }

    /// Get the local address this listener is bound to
    pub fn local_addr(&self) -> Result<String, TransportError> {
        match self {
            Listener::Tcp(listener) => {
                listener.local_addr()
                    .map(|addr| addr.to_string())
                    .map_err(TransportError::Tcp)
            }
            Listener::Vsock(listener) => {
                listener.local_addr()
                    .map(|addr| format!("vsock://{}:{}", addr.cid(), addr.port()))
                    .map_err(|e| TransportError::Vsock(e.to_string()))
            }
        }
    }
}

/// Transport trait providing unified interface for different transport types.
#[async_trait::async_trait]
pub trait Transport {
    /// Bind to the specified address and return a listener
    async fn bind(config: &TransportConfig) -> Result<Listener, TransportError>;

    /// Connect to the specified address and return a connection
    async fn connect(config: &TransportConfig) -> Result<Connection, TransportError>;

    /// Get a human-readable name for this transport type
    fn name() -> &'static str;
}

/// TCP transport implementation
pub struct TcpTransport;

#[async_trait::async_trait]
impl Transport for TcpTransport {
    async fn bind(config: &TransportConfig) -> Result<Listener, TransportError> {
        match config {
            TransportConfig::Tcp(addr) => {
                let listener = TcpListener::bind(addr).await
                    .map_err(|e| TransportError::BindFailed(format!("TCP bind to {} failed: {}", addr, e)))?;
                Ok(Listener::Tcp(listener))
            }
            _ => Err(TransportError::NotSupported("TCP transport does not support VSOCK addresses".to_string())),
        }
    }

    async fn connect(config: &TransportConfig) -> Result<Connection, TransportError> {
        match config {
            TransportConfig::Tcp(addr) => {
                let stream = TcpStream::connect(addr).await
                    .map_err(|e| TransportError::ConnectionFailed(format!("TCP connect to {} failed: {}", addr, e)))?;
                Ok(Connection::Tcp(stream))
            }
            _ => Err(TransportError::NotSupported("TCP transport does not support VSOCK addresses".to_string())),
        }
    }

    fn name() -> &'static str {
        "TCP"
    }
}

/// VSOCK transport implementation
pub struct VsockTransport;

#[async_trait::async_trait]
impl Transport for VsockTransport {
    async fn bind(config: &TransportConfig) -> Result<Listener, TransportError> {
        match config {
            TransportConfig::Vsock { cid, port } => {
                let cid = match *cid {
                    0 => VMADDR_CID_ANY,
                    _ => *cid
                };
                let listener = VsockListener::bind(cid, *port)
                    .map_err(|e| TransportError::BindFailed(format!("VSOCK bind to {}:{} failed: {}", cid, port, e)))?;
                Ok(Listener::Vsock(listener))
            }
            _ => Err(TransportError::NotSupported("VSOCK transport does not support TCP addresses".to_string())),
        }
    }

    async fn connect(config: &TransportConfig) -> Result<Connection, TransportError> {
        match config {
            TransportConfig::Vsock { cid, port } => {
                let stream = VsockStream::connect(*cid, *port).await
                    .map_err(|e| TransportError::ConnectionFailed(format!("VSOCK connect to {}:{} failed: {}", cid, port, e)))?;
                Ok(Connection::Vsock(stream))
            }
            _ => Err(TransportError::NotSupported("VSOCK transport does not support TCP addresses".to_string())),
        }
    }

    fn name() -> &'static str {
        "VSOCK"
    }
}

/// Factory for creating transport instances based on configuration
pub struct TransportFactory;

impl TransportFactory {
    /// Create a listener for the given transport configuration
    pub async fn bind(config: &TransportConfig) -> Result<Listener, TransportError> {
        match config {
            TransportConfig::Tcp(_) => TcpTransport::bind(config).await,
            TransportConfig::Vsock { .. } => VsockTransport::bind(config).await,
        }
    }

    /// Create a connection for the given transport configuration
    pub async fn connect(config: &TransportConfig) -> Result<Connection, TransportError> {
        match config {
            TransportConfig::Tcp(_) => TcpTransport::connect(config).await,
            TransportConfig::Vsock { .. } => VsockTransport::connect(config).await,
        }
    }

    /// Get the transport name for the given configuration
    pub fn transport_name(config: &TransportConfig) -> &'static str {
        match config {
            TransportConfig::Tcp(_) => TcpTransport::name(),
            TransportConfig::Vsock { .. } => VsockTransport::name(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transport_config_from_str() {
        // Test TCP address parsing
        let tcp_config: TransportConfig = "127.0.0.1:50051".parse().unwrap();
        assert!(tcp_config.is_tcp());
        assert_eq!(tcp_config.port(), 50051);

        // Test VSOCK address parsing
        let vsock_config: TransportConfig = "vsock://2:50051".parse().unwrap();
        assert!(vsock_config.is_vsock());
        assert_eq!(vsock_config.port(), 50051);
        if let TransportConfig::Vsock { cid, port } = vsock_config {
            assert_eq!(cid, 2);
            assert_eq!(port, 50051);
        }

        // Test invalid addresses
        assert!("invalid".parse::<TransportConfig>().is_err());
        assert!("vsock://invalid:port".parse::<TransportConfig>().is_err());
        assert!("vsock://2".parse::<TransportConfig>().is_err());
    }

    #[test]
    fn test_transport_config_display() {
        let tcp_config = TransportConfig::Tcp("127.0.0.1:50051".parse().unwrap());
        assert_eq!(tcp_config.to_string(), "127.0.0.1:50051");

        let vsock_config = TransportConfig::Vsock { cid: 2, port: 50051 };
        assert_eq!(vsock_config.to_string(), "vsock://2:50051");
    }
}