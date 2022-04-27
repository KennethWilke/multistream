use anyhow::Result;
use easy_tokio_rustls::{TlsListener, TlsStream};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream, UnixListener, UnixStream},
};

use easy_tokio_rustls::{resolve_address, TlsServer};

use crate::DEFAULT_BUFFER_SIZE;

pub struct Server {
    pub address: String,
    pub listener: StreamListener,
    pub buffer_size: usize,
}

impl Server {
    pub async fn listen<T>(address: T, cert_and_key: Option<CertAndKey>) -> Result<Server>
    where
        T: ToString,
    {
        use StreamListener::*;
        let address = address.to_string();
        let listener = match &address {
            addr if addr.starts_with("tcp://") => {
                let addr = resolve_address(&addr[6..]).await?;
                Tcp(TcpListener::bind(addr).await?)
            }
            host if host.starts_with("tls://") && cert_and_key.is_some() => {
                let host = resolve_address(&host[6..]).await?;
                let cert_file = &cert_and_key.as_ref().unwrap().cert;
                let key_file = &cert_and_key.as_ref().unwrap().key;
                Tls(TlsServer::new(host, cert_file, key_file)
                    .await?
                    .listen()
                    .await?)
            }
            path if path.starts_with("unix://") => Unix(UnixListener::bind(&path[7..])?),
            // A nice default
            id_like_to_think_tls if cert_and_key.is_some() => {
                match id_like_to_think_tls.contains("://") {
                    true => {
                        let addr = id_like_to_think_tls.split_once("://").unwrap().1;
                        Tcp(TcpListener::bind(addr).await?)
                    }
                    _ => Tcp(TcpListener::bind(id_like_to_think_tls).await?),
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
        let server = Server {
            address,
            listener,
            buffer_size: DEFAULT_BUFFER_SIZE,
        };
        Ok(server)
    }

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

pub enum StreamListener {
    Tcp(TcpListener),
    Tls(TlsListener),
    Unix(UnixListener),
}

pub struct StreamClient {
    pub address: String,
    pub stream: ClientStream,
}

impl StreamClient {
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

        let mut buffer = [0; DEFAULT_BUFFER_SIZE];
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

pub struct CertAndKey {
    pub cert: String,
    pub key: String,
}

impl CertAndKey {
    pub fn new<T, U>(cert: T, key: U) -> Self
    where
        T: ToString,
        U: ToString,
    {
        CertAndKey {
            cert: cert.to_string(),
            key: key.to_string(),
        }
    }
}
