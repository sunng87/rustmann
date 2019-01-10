#![feature(await_macro, async_await, futures_api)]

#[macro_use]
extern crate tokio;

mod client;
mod codec;
mod connection;
pub mod protos;

pub use crate::client::{Client, ClientOptions, ClientOptionsBuilder};
