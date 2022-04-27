use anyhow::Result;
use easy_tokio_rustls::{resolve_address, TlsClient, TlsStream};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpStream, UnixStream},
};

use crate::DEFAULT_BUFFER_SIZE;

pub struct Client {
    pub address: String,
    pub stream: ClientStream,
    pub buffer_size: usize,
}

impl Client {
    pub async fn connect<T>(address: T) -> Result<Self>
    where
        T: ToString,
    {
        use ClientStream::*;
        let address = address.to_string();
        let stream = match &address {
            addr if addr.starts_with("tcp://") => {
                let addr = resolve_address(&addr[6..]).await?;
                Tcp(TcpStream::connect(addr).await?)
            }
            host if host.starts_with("tls://") => {
                let host = resolve_address(&host[6..]).await?;
                Tls(Box::new(TlsClient::new(host).await?.connect().await?))
            }
            path if path.starts_with("unix://") => Unix(UnixStream::connect(&path[7..]).await?),
            // A nice default
            assumed_tls => match assumed_tls.contains("://") {
                true => {
                    let host = assumed_tls.split_once("://").unwrap().1;
                    Tls(Box::new(TlsClient::new(host).await?.connect().await?))
                }
                _ => Tls(Box::new(
                    TlsClient::new(assumed_tls).await?.connect().await?,
                )),
            },
        };
        let client = Client {
            address,
            stream,
            buffer_size: DEFAULT_BUFFER_SIZE,
        };
        Ok(client)
    }

    pub async fn send(&mut self, data: &[u8]) -> Result<()> {
        use ClientStream::*;
        match &mut self.stream {
            Tcp(stream) => {
                stream.write_all(data).await?;
                stream.flush().await?
            }
            Tls(stream) => {
                stream.write_all(data).await?;
                stream.flush().await?
            }
            Unix(stream) => {
                stream.write_all(data).await?;
                stream.flush().await?
            }
        };
        Ok(())
    }

    pub async fn recv(&mut self) -> Result<Vec<u8>> {
        use ClientStream::*;

        let mut buffer = [0; 8192];
        let size = match &mut self.stream {
            Tcp(stream) => stream.read(&mut buffer).await?,
            Tls(stream) => stream.read(&mut buffer).await?,
            Unix(stream) => stream.read(&mut buffer).await?,
        };
        Ok(buffer[0..size].to_vec())
    }
}

pub enum ClientStream {
    Tcp(TcpStream),
    Tls(Box<TlsStream<TcpStream>>),
    Unix(UnixStream),
}
