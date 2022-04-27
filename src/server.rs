use anyhow::Result;
use easy_tokio_rustls::{TlsListener, TlsServer, TlsStream};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream, UnixListener, UnixStream},
};

use crate::DEFAULT_BUFFER_SIZE;

/// Represents a server! Get ready for all those new connections
pub struct Server {
    /// The address being listened to
    pub address: String,
    /// Provides access to the underlying stream, so you can work beyond the
    /// simple abstraction
    pub listener: StreamListener,
}

impl Server {
    /// Create a new server, will be listening and ready to go!
    /// If a `cert_and_key` is provided, TLS will be used
    /// If address begins with unix://, Unix socket will be used
    /// Otherwise, TCP socket type will be assumed
    pub async fn listen<T>(address: T, cert_and_key: Option<CertAndKeyFilePaths>) -> Result<Server>
    where
        T: ToString,
    {
        use StreamListener::*;
        let address = address.to_string();
        let listener = match &address {
            path if path.starts_with("unix://") => Unix(UnixListener::bind(&path[7..])?),
            // A nice default
            id_like_to_think_tls if cert_and_key.is_some() => {
                match id_like_to_think_tls.contains("://") {
                    true => {
                        let addr = id_like_to_think_tls.split_once("://").unwrap().1;
                        let cert_file = &cert_and_key.as_ref().unwrap().cert;
                        let key_file = &cert_and_key.as_ref().unwrap().key;
                        let server = TlsServer::new(addr, cert_file, key_file).await?;
                        Tls(server.listen().await?)
                    }
                    false => {
                        let cert_file = &cert_and_key.as_ref().unwrap().cert;
                        let key_file = &cert_and_key.as_ref().unwrap().key;
                        let server =
                            TlsServer::new(id_like_to_think_tls, cert_file, key_file).await?;
                        Tls(server.listen().await?)
                    }
                }
            }
            fine_assumed_tcp => match fine_assumed_tcp.contains("://") {
                true => {
                    let addr = fine_assumed_tcp.split_once("://").unwrap().1;
                    Tcp(TcpListener::bind(addr).await?)
                }
                _ => Tcp(TcpListener::bind(fine_assumed_tcp).await?),
            },
        };
        let server = Server { address, listener };
        Ok(server)
    }

    /// Accept connection from a new client
    pub async fn accept(&mut self) -> Result<StreamClient> {
        let (stream, address) = match &self.listener {
            StreamListener::Tcp(listener) => {
                let (stream, address) = listener.accept().await?;
                (ClientStream::Tcp(stream), address.to_string())
            }
            StreamListener::Tls(listener) => {
                let (tcp_stream, address) = listener.stream_accept().await?;
                let stream = tcp_stream.tls_accept().await?;
                (ClientStream::Tls(Box::new(stream)), address.to_string())
            }
            StreamListener::Unix(listener) => {
                let (stream, _) = listener.accept().await?;
                (ClientStream::Unix(stream), self.address.clone())
            }
        };
        Ok(StreamClient { address, stream })
    }
}

/// This is the underlying listener handle for the server
pub enum StreamListener {
    Tcp(TcpListener),
    Tls(TlsListener),
    Unix(UnixListener),
}

/// This structure represents a connected client
pub struct StreamClient {
    pub address: String,
    pub stream: ClientStream,
}

impl StreamClient {
    /// Sends the provided buffer to the connected client
    pub async fn send(&mut self, data: &[u8]) -> Result<()> {
        use ClientStream::*;
        match &mut self.stream {
            Tcp(stream) => {
                stream.write_all(data).await?;
            }
            Tls(stream) => {
                stream.write_all(data).await?;
            }
            Unix(stream) => {
                stream.write_all(data).await?;
            }
        };
        Ok(())
    }

    /// Receives data from the connected client
    pub async fn recv(&mut self) -> Result<Vec<u8>> {
        use ClientStream::*;

        let mut buffer = [0; DEFAULT_BUFFER_SIZE];
        let size = match &mut self.stream {
            Tcp(stream) => stream.read(&mut buffer).await?,
            Tls(stream) => stream.read(&mut buffer).await?,
            Unix(stream) => stream.read(&mut buffer).await?,
        };
        Ok(buffer[0..size].to_vec())
    }
}

/// Holds the underlying stream handle for a connected client
pub enum ClientStream {
    /// Handle for a TCP connected client
    Tcp(TcpStream),
    /// Handle for a TLS connected client
    Tls(Box<TlsStream<TcpStream>>),
    /// Handle for a Unix socket connected client
    Unix(UnixStream),
}

/// Simple structure that holds file paths to the TLS certificate and key
pub struct CertAndKeyFilePaths {
    /// Path to TLS certificate file
    pub cert: String,
    /// Path to TLS key file
    pub key: String,
}

impl CertAndKeyFilePaths {
    /// Creates a new Cert/Key file path pair
    pub fn new<T, U>(cert: T, key: U) -> Self
    where
        T: ToString,
        U: ToString,
    {
        CertAndKeyFilePaths {
            cert: cert.to_string(),
            key: key.to_string(),
        }
    }
}
