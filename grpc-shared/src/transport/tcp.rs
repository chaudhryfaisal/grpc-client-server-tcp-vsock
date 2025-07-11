//! TCP transport implementation using tokio
//!
//! This module implements TCP transport as specified in PRD Task 5: TCP Transport Implementation

use crate::config::TransportType;
use crate::error::{NetworkError, Result, TransportError};
use crate::transport::{Connection, Listener, Transport};
use async_trait::async_trait;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

/// TCP transport implementation
#[derive(Debug)]
pub struct TcpTransport;

impl TcpTransport {
    /// Create a new TCP transport
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl Transport for TcpTransport {
    async fn connect(&self, address: &str) -> Result<Box<dyn Connection>> {
        let stream = TcpStream::connect(address).await.map_err(|e| {
            NetworkError::ConnectionFailed {
                message: format!("Failed to connect to {}: {}", address, e),
            }
        })?;

        Ok(Box::new(TcpConnection::new(stream)))
    }

    async fn bind(&self, address: &str) -> Result<Box<dyn Listener>> {
        let listener = TcpListener::bind(address).await.map_err(|e| {
            TransportError::Configuration {
                message: format!("Failed to bind to {}: {}", address, e),
            }
        })?;

        Ok(Box::new(TcpListenerWrapper::new(listener)))
    }

    fn transport_type(&self) -> TransportType {
        TransportType::Tcp
    }
}

/// TCP connection wrapper
#[derive(Debug)]
pub struct TcpConnection {
    stream: TcpStream,
}

impl TcpConnection {
    /// Create a new TCP connection
    pub fn new(stream: TcpStream) -> Self {
        Self { stream }
    }
}

#[async_trait]
impl Connection for TcpConnection {
    async fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        self.stream.read(buf).await.map_err(|e| {
            NetworkError::ConnectionLost {
                reason: format!("TCP read error: {}", e),
            }
            .into()
        })
    }

    async fn write(&mut self, buf: &[u8]) -> Result<usize> {
        self.stream.write(buf).await.map_err(|e| {
            NetworkError::ConnectionLost {
                reason: format!("TCP write error: {}", e),
            }
            .into()
        })
    }

    async fn close(&mut self) -> Result<()> {
        self.stream.shutdown().await.map_err(|e| {
            TransportError::Tcp {
                message: format!("Failed to close TCP connection: {}", e),
            }
            .into()
        })
    }
}

/// TCP listener wrapper
#[derive(Debug)]
pub struct TcpListenerWrapper {
    listener: TcpListener,
}

impl TcpListenerWrapper {
    /// Create a new TCP listener wrapper
    pub fn new(listener: TcpListener) -> Self {
        Self { listener }
    }
}

#[async_trait]
impl Listener for TcpListenerWrapper {
    async fn accept(&mut self) -> Result<Box<dyn Connection>> {
        let (stream, _addr) = self.listener.accept().await.map_err(|e| {
            TransportError::Tcp {
                message: format!("Failed to accept TCP connection: {}", e),
            }
        })?;

        Ok(Box::new(TcpConnection::new(stream)))
    }

    async fn close(&mut self) -> Result<()> {
        // TCP listener doesn't have an explicit close method in tokio
        // The listener will be closed when dropped
        Ok(())
    }
}

impl Default for TcpTransport {
    fn default() -> Self {
        Self::new()
    }
}