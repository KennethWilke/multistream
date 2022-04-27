mod client;
mod server;

pub use client::*;
pub use server::*;

const DEFAULT_BUFFER_SIZE: usize = 8 * 1024;
