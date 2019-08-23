use std::str::FromStr;

use derive_builder::Builder;
use getset::Getters;
use std::net::SocketAddr;

#[derive(Debug, Builder, Clone, Copy, Getters)]
#[builder(setter(into))]
#[get = "pub"]
pub struct RiemannClientOptions {
    address: SocketAddr,
    connect_timeout_ms: u64,
    socket_timeout_ms: u64,
    use_tls: bool,
    // TODO: the type
    client_cert: Option<bool>,
}

impl Default for RiemannClientOptions {
    fn default() -> RiemannClientOptions {
        Self {
            address: SocketAddr::from_str("127.0.0.1:5555").unwrap(),
            connect_timeout_ms: 2000,
            socket_timeout_ms: 3000,
            #[cfg(feature = "tls")]
            use_tls: true,
            #[cfg(feature = "tls")]
            client_cert: None,
        }
    }
}
