use anyhow::Result;
use easy_tokio_rustls::TlsStream;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpStream, UnixStream},
};

pub enum MultiStream {
    Tcp(TcpStream),
    Tls(TlsStream<TcpStream>),
    Unix(UnixStream),
}

impl MultiStream {
    pub async fn send(&mut self, data: &[u8]) -> Result<()> {
        use MultiStream::*;
        match self {
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
        use MultiStream::*;

        let mut buffer = [0; 8192];
        let size = match self {
            Tcp(stream) => stream.read(&mut buffer).await?,
            Tls(stream) => stream.read(&mut buffer).await?,
            Unix(stream) => stream.read(&mut buffer).await?,
        };
        Ok(buffer[0..size].to_vec())
    }
}
