#[macro_use]
extern crate failure;

mod client;
mod codec;
mod connection;
mod error;
mod options;
pub mod protos;

pub use crate::client::RiemannClient;
pub use crate::error::RiemannClientError;
pub use crate::options::{RiemannClientOptions, RiemannClientOptionsBuilder};
