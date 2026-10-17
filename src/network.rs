//! Explicit blocking TCP connections for the Networking foundation.
//!
//! This module exposes byte-stream operations only. It does not frame or
//! serialize messages, authenticate peers, reconnect automatically, or create
//! background tasks.

use core::fmt;
use std::io::{self, Read, Write};
use std::net::{Shutdown, SocketAddr, TcpListener, TcpStream};
use std::time::Duration;

/// Errors returned by [`Connection`] operations.
#[derive(Debug)]
pub enum ConnectionError {
    /// The operating system failed to establish, use, configure, or close the
    /// underlying TCP stream.
    Io(io::Error),
}

impl fmt::Display for ConnectionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(error) => write!(f, "network connection I/O failed: {error}"),
        }
    }
}

impl std::error::Error for ConnectionError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(error) => Some(error),
        }
    }
}

impl From<io::Error> for ConnectionError {
    fn from(error: io::Error) -> Self {
        Self::Io(error)
    }
}

/// A blocking TCP listening endpoint that accepts incoming connections.
///
/// Binding an endpoint does not create background tasks or manage accepted
/// connection state. `accept` blocks until a peer connects or the operating
/// system returns an error. The endpoint does not implement message framing,
/// authentication, encryption, or connection-state tracking.
#[derive(Debug)]
pub struct Endpoint {
    listener: TcpListener,
}

impl Endpoint {
    /// Binds a TCP listening endpoint to the requested local socket address.
    ///
    /// Use port `0` to ask the operating system to choose an available port,
    /// then inspect it with [`Self::local_addr`].
    pub fn bind(address: SocketAddr) -> Result<Self, ConnectionError> {
        Ok(Self {
            listener: TcpListener::bind(address)?,
        })
    }

    /// Returns the local address assigned to this listening endpoint.
    pub fn local_addr(&self) -> Result<SocketAddr, ConnectionError> {
        Ok(self.listener.local_addr()?)
    }

    /// Configures whether `accept` is non-blocking.
    ///
    /// The default is blocking. In non-blocking mode, `accept` may return an
    /// I/O error with kind `WouldBlock` when no connection is waiting.
    pub fn set_nonblocking(&self, nonblocking: bool) -> Result<(), ConnectionError> {
        self.listener.set_nonblocking(nonblocking)?;
        Ok(())
    }

    /// Accepts one incoming connection and returns its peer address.
    ///
    /// In the default blocking mode this waits until a peer connects or an
    /// operating-system error occurs. The returned stream is wrapped in the
    /// same byte-stream [`Connection`] API used by outbound connections.
    pub fn accept(&self) -> Result<(Connection, SocketAddr), ConnectionError> {
        let (stream, peer_address) = self.listener.accept()?;
        Ok((Connection::from_tcp_stream(stream), peer_address))
    }
}

/// An owned, blocking TCP byte stream.
///
/// Reads may return fewer bytes than requested; a read returning `Ok(0)` means
/// the peer has closed its sending side (EOF), or the supplied buffer is empty.
/// Use [`Self::write_all`] when the whole byte slice must be written. This type
/// does not add message boundaries: callers must apply a framing protocol above
/// the stream. Access requires `&mut self`, so concurrent use needs external
/// synchronization. No background tasks or automatic reconnects are created.
#[derive(Debug)]
pub struct Connection {
    stream: TcpStream,
}

impl Connection {
    /// Establishes a TCP connection using the operating system's default
    /// connection timeout behaviour.
    pub fn connect(address: SocketAddr) -> Result<Self, ConnectionError> {
        Ok(Self {
            stream: TcpStream::connect(address)?,
        })
    }

    /// Establishes a TCP connection with an explicit maximum connection time.
    /// The timeout must be non-zero; platform granularity may apply.
    pub fn connect_timeout(
        address: SocketAddr,
        timeout: Duration,
    ) -> Result<Self, ConnectionError> {
        if timeout.is_zero() {
            return Err(ConnectionError::Io(io::Error::new(
                io::ErrorKind::InvalidInput,
                "connection timeout must be non-zero",
            )));
        }
        Ok(Self {
            stream: TcpStream::connect_timeout(&address, timeout)?,
        })
    }

    /// Wraps an already established TCP stream, such as one accepted by a listener.
    pub fn from_tcp_stream(stream: TcpStream) -> Self {
        Self { stream }
    }

    /// Returns the local socket address reported by the operating system.
    pub fn local_addr(&self) -> Result<SocketAddr, ConnectionError> {
        Ok(self.stream.local_addr()?)
    }

    /// Returns the peer socket address reported by the operating system.
    pub fn peer_addr(&self) -> Result<SocketAddr, ConnectionError> {
        Ok(self.stream.peer_addr()?)
    }

    /// Sets the read timeout. `None` disables the timeout.
    pub fn set_read_timeout(&self, timeout: Option<Duration>) -> Result<(), ConnectionError> {
        self.stream.set_read_timeout(timeout)?;
        Ok(())
    }

    /// Sets the write timeout. `None` disables the timeout.
    pub fn set_write_timeout(&self, timeout: Option<Duration>) -> Result<(), ConnectionError> {
        self.stream.set_write_timeout(timeout)?;
        Ok(())
    }

    /// Reads some bytes. Reads may be partial; `Ok(0)` on a non-empty buffer is EOF.
    pub fn read(&mut self, buffer: &mut [u8]) -> Result<usize, ConnectionError> {
        Ok(self.stream.read(buffer)?)
    }

    /// Writes all bytes or returns the underlying I/O error.
    ///
    /// If an error occurs after a prefix was written, stream state may be
    /// uncertain. This method does not restart the whole slice automatically.
    pub fn write_all(&mut self, bytes: &[u8]) -> Result<(), ConnectionError> {
        self.stream.write_all(bytes)?;
        Ok(())
    }

    /// Explicitly shuts down the selected stream directions.
    pub fn shutdown(&self, how: Shutdown) -> Result<(), ConnectionError> {
        self.stream.shutdown(how)?;
        Ok(())
    }
}
