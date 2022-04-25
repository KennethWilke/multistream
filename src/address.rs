use std::net::SocketAddr;

use anyhow::Result;
use easy_tokio_rustls::{resolve_address, TlsClient, TlsServer};
use tokio::net::{TcpListener, TcpStream, UnixListener, UnixStream};

use crate::{MultiStreamError, StreamServer};

use super::MultiStream;

pub enum StreamAddress {
    Tcp(SocketAddr),
    Tls(TlsType),
    Unix(String),
}

pub enum TlsType {
    Client(SocketAddr),
    Server(String, ServerKey),
}

pub struct ServerKey {
    cert: String,
    key: String,
}

impl StreamAddress {
    pub async fn tcp<T>(host: T) -> Result<Self> 
    where T: tokio::net::ToSocketAddrs + ToString + Copy
    {
        let address = resolve_address(host).await?;
        Ok(StreamAddress::Tcp(address))
    }

    pub async fn tls<T>(host: T, server_key: Option<ServerKey>) -> Result<Self> 
    where T: tokio::net::ToSocketAddrs + ToString + Copy 
    {
        let tls_type = match server_key {
            Some(key) => TlsType::Server(host.to_string(), key),
            None => TlsType::Client(resolve_address(host).await?),
        };
        Ok(StreamAddress::Tls(tls_type))
    }

    pub async fn unix<T>(path: T) -> Result<Self> 
    where T: ToString
    {
        Ok(StreamAddress::Unix(path.to_string()))
    }

    pub async fn connect(&self) -> Result<MultiStream> {
        let client = match &self {
            StreamAddress::Tcp(addr) => {
                let stream = TcpStream::connect(addr).await?;
                MultiStream::Tcp(stream)
            }
            StreamAddress::Tls(tls_type) => match tls_type {
                TlsType::Client(address) => {
                    let client = TlsClient::new(address).await?;
                    let stream = client.connect().await?;
                    MultiStream::Tls(stream)
                }
                TlsType::Server(_, _) => {
                    return Err(MultiStreamError::TlsTypeError(
                        "connect called on Server subvariant",
                    )
                    .into())
                }
            },
            StreamAddress::Unix(path) => {
                let stream = UnixStream::connect(path).await?;
                MultiStream::Unix(stream)
            }
        };
        Ok(client)
    }

    pub async fn listen(&self) -> Result<StreamServer> {
        let listener = match &self {
            StreamAddress::Tcp(address) => {
                let listener = TcpListener::bind(address).await?;
                StreamServer::Tcp(listener)
            }
            StreamAddress::Tls(tls_type) => {
                let server = match tls_type {
                    TlsType::Server(address, server_key) => {
                        TlsServer::new(address, server_key.cert.clone(), server_key.key.clone())
                            .await?
                    }
                    TlsType::Client(_) => {
                        return Err(MultiStreamError::TlsTypeError(
                            "listen called on Client subvariant",
                        )
                        .into())
                    }
                };
                StreamServer::Tls(server.listen().await?)
            }
            StreamAddress::Unix(path) => {
                let listener = UnixListener::bind(path)?;
                StreamServer::Unix(listener, path.to_string())
            }
        };
        Ok(listener)
    }
}
