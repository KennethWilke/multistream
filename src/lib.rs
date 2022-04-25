use thiserror::Error;

mod address;
mod server;
mod stream;

pub use address::StreamAddress;
pub use server::StreamServer;
pub use stream::MultiStream;

/// Represents custom errors returned directly by this crate
#[derive(Error, Debug)]
pub enum MultiStreamError {
    /// Returned for address resolution failures
    #[error("Tls Type Error: {0}")]
    TlsTypeError(&'static str),
}
