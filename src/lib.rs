#![doc(html_root_url = "https://docs.rs/rustmann/")]
//! # Rustmann
//!
//! Rustmann is the rust client for [riemann](https://riemann.io), a popular
//! event aggregator and processor for distributed system.
//!
//! This implementation is based on tokio and using async-await style API.
//!
//! ## Features
//!
//! * Full async-await API
//! * TCP/UDP/TLS transport support
//! * Auto reconnect
//! * Send and query API
//! * EventBuilder
//! * A usable Cli in example
//!
//! ## Quick Start
//!
//! ```no_run
//! use rustmann::{EventBuilder, RiemannClient, RiemannClientError, RiemannClientOptions};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), RiemannClientError> {
//!     // create client with default configuration (to localhost:5555)
//!     let mut client = RiemannClient::new(&RiemannClientOptions::default());
//!
//!     // create a riemann event using evnet builder API
//!     let event = EventBuilder::new()
//!         .service("riemann_test")
//!         .state("ok")
//!         .metric_f(123.4)
//!         .build();
//!
//!     // send event to server
//!     let response = client.send_events(vec![event]).await?;
//!     println!("{:?}", response);
//!
//!     // query riemann
//!     let query_response = client.send_query("service = \"riemann_test\"").await?;
//!     println!("{:?}", query_response);
//!     Ok(())
//! }
//! ```
//!
//! See [examples](https://github.com/sunng87/rustmann/tree/master/examples) for
//! more usage demo.
//!

mod client;
mod codec;
mod error;
mod event;
mod options;
pub mod protos {
    pub mod riemann {
        include!(concat!(env!("OUT_DIR"), "/riemann.rs"));
    }
}
mod state;
#[cfg(feature = "tls")]
mod tls;
mod transport;

pub use crate::client::RiemannClient;
pub use crate::error::RiemannClientError;
pub use crate::event::EventBuilder;
pub use crate::options::{RiemannClientOptions, RiemannClientOptionsBuilder};

#[cfg(feature = "tls")]
pub use tokio_rustls::rustls::ClientConfig;
