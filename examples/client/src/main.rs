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
