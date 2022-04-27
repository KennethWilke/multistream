# Multistream

Multistream creates a common socket stream client/server interface across
plaintext TCP, TLS encrypted TCP and UNIX sockets.

Below are the main files from [the examples](https://github.com/KennethWilke/multistream/tree/main/examples/)

## Client Example

```rust
use anyhow::Result;

use multistream::Client;

const HTTP_REQUEST: &[u8] = b"GET / HTTP/1.1\r\nHost: suchprogramming.com\r\n\r\n";
// const REDIS_REQUEST: &[u8] = b"PING\r\n";

#[tokio::main]
pub async fn main() -> Result<()> {

    // Will use TLS unless given tcp://, unix:// or other handled prefixes
    let mut client = Client::connect("suchprogramming.com:443").await?;
    // let mut client = Client::connect("tls://suchprogramming.com:443").await?; # Same as above
    // let mut client = Client::connect("tcp://suchprogramming:80").await?;

    client.send(HTTP_REQUEST).await?;

    loop {
        let buffer = client.recv().await?;
        let html = std::str::from_utf8(&buffer).unwrap();
        print!("{}", html);
        if html.contains("</html>") {
            return Ok(());
        }
    }

    /*

    // Or for redis via unix socket
    let mut client = Client::connect("unix:///var/run/redis/redis-server.sock").await?;

    client.send(REDIS_REQUEST).await?;
    let buffer = client.recv().await?;
    println!("{:?}", std::str::from_utf8(&buffer).unwrap());
    Ok(())

    */
}
```

## Server Example

```rust
use anyhow::Result;

use multistream::{Server, CertAndKeyFilePaths};

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
```
