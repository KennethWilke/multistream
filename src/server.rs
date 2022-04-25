use std::net::SocketAddr;

use anyhow::Result;
use easy_tokio_rustls::{TlsListener, TlsStream};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream, UnixListener, UnixStream},
};

pub enum StreamServer {
    Tcp(TcpListener),
    Tls(TlsListener),
    Unix(UnixListener, String),
}

impl StreamServer {
    pub async fn accept(&mut self) -> Result<StreamServerClient> {
        let client = match &self {
            StreamServer::Tcp(listener) => {
                let (stream, address) = listener.accept().await?;
                StreamServerClient::Tcp(stream, address)
            }
            StreamServer::Tls(listener) => {
                let (tcp_stream, address) = listener.stream_accept().await?;
                let stream = tcp_stream.tls_accept().await?;
                StreamServerClient::Tls(stream, address)
            }
            StreamServer::Unix(listener, path) => {
                let (stream, _) = listener.accept().await?;
                StreamServerClient::Unix(stream, path.clone())
            }
        };
        Ok(client)
    }
}

pub enum StreamServerClient {
    Tcp(TcpStream, SocketAddr),
    Tls(TlsStream<TcpStream>, SocketAddr),
    Unix(UnixStream, String),
}

impl StreamServerClient {
    pub async fn send(&mut self, data: &[u8]) -> Result<()> {
        use StreamServerClient::*;
        match self {
            Tcp(stream, _) => {
                stream.write_all(data).await?;
                stream.flush().await?
            }
            Tls(stream, _) => {
                stream.write_all(data).await?;
                stream.flush().await?
            }
            Unix(stream, _) => {
                stream.write_all(data).await?;
                stream.flush().await?
            }
        };
        Ok(())
    }

    pub async fn recv(&mut self) -> Result<Vec<u8>> {
        use StreamServerClient::*;

        let mut buffer = [0; 8192];
        let size = match self {
            Tcp(stream, _) => stream.read(&mut buffer).await?,
            Tls(stream, _) => stream.read(&mut buffer).await?,
            Unix(stream, _) => stream.read(&mut buffer).await?,
        };
        Ok(buffer[0..size].to_vec())
    }

    pub fn get_address(&self) -> String {
        use StreamServerClient::*;
        match self {
            Tcp(_, address) => address.to_string(),
            Tls(_, address) => address.to_string(),
            Unix(_, address) => address.to_string(),
        }
    }
}
