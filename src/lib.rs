//! Multistream provides some simple abstrations for streaming client and
//! servers. Check out [the examples](https://github.com/KennethWilke/multistream/tree/main/examples/)
//! to help you get started.

mod client;
mod server;

pub use client::*;
pub use server::*;

const DEFAULT_BUFFER_SIZE: usize = 8 * 1024;
