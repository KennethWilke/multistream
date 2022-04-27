use anyhow::Result;
use easy_tokio_rustls::{resolve_address, TlsClient, TlsStream};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpStream, UnixStream},
};

use crate::DEFAULT_BUFFER_SIZE;

/// This structure represents your client connection to the server
pub struct Client {
    /// The address of the server you connected to
    pub address: String,
    /// A handle to the underlying stream, in case you need more power
    pub stream: ClientStream,
}

impl Client {
    /// Connect to that server, be safe out there.
    /// This assumes you want to use TLS, because I like that assumption
    /// You can also use tcp:// or http:// if you're feeling wild
    /// Or unix:// if you've got unix socketry in mind
    pub async fn connect<T>(address: T) -> Result<Self>
    where
        T: ToString,
    {
        use ClientStream::*;
        let address = address.to_string();
        let stream = match &address {
            addr if addr.starts_with("tcp://") || addr.starts_with("http://") => {
                let addr = addr.split_once("://").unwrap().1;
                let addr = resolve_address(&addr).await?;
                Tcp(TcpStream::connect(addr).await?)
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
        let client = Client { address, stream };
        Ok(client)
    }

    /// Say hello to that world, er server....
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

    /// C'mon back, if that makes sense for our protocol
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

/// This enumeration holds the underlying stream handle
pub enum ClientStream {
    /// TCP handle to the server
    Tcp(TcpStream),
    /// TLS handle to the server
    Tls(Box<TlsStream<TcpStream>>),
    /// Unix handle to the server
    Unix(UnixStream),
}
