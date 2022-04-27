use anyhow::Result;

use multistream::{CertAndKeyFilePaths, Server};

const RESPONSE: &[u8] = b"HTTP/1.1 200 OK\r\nServer: a very great server\r\n\r\n";

#[tokio::main]
pub async fn main() -> Result<()> {
    let cert_and_key = CertAndKeyFilePaths::new("cert.pem", "privkey.pem");
    let mut server = Server::listen("0.0.0.0:8443", Some(cert_and_key)).await?;

    let mut client = server.accept().await?;

    let buffer = client.recv().await?;
    println!("{:#?}", std::str::from_utf8(&buffer)?);
    client.send(RESPONSE).await?;

    Ok(())
}
