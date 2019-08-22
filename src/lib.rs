#[macro_use]
extern crate failure;

mod client;
mod codec;
mod connection;
mod error;
pub mod protos;

pub use crate::client::{RiemannClient, RiemannClientOptions, RiemannClientOptionsBuilder};
pub use crate::error::RiemannClientError;
