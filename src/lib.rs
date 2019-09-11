#[macro_use]
extern crate failure;

mod client;
mod codec;
mod error;
mod options;
pub mod protos;
mod tls;
mod transport;

pub use crate::client::RiemannClient;
pub use crate::error::RiemannClientError;
pub use crate::options::{RiemannClientOptions, RiemannClientOptionsBuilder};

pub use tokio_rustls::rustls::ClientConfig;
