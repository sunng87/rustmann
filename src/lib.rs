#[macro_use]
extern crate failure;

mod client;
mod codec;
mod error;
mod options;
pub mod protos;
#[cfg(feature = "tls")]
mod tls;
mod transport;

pub use crate::client::RiemannClient;
pub use crate::error::RiemannClientError;
pub use crate::options::{RiemannClientOptions, RiemannClientOptionsBuilder};

#[cfg(feature = "tls")]
pub use tokio_rustls::rustls::ClientConfig;
