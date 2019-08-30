use std::io;
use std::net::{SocketAddr, ToSocketAddrs};

use derive_builder::Builder;
use getset::Getters;

#[derive(Debug, Builder, Clone, Getters)]
#[builder(setter(into))]
#[get = "pub"]
pub struct RiemannClientOptions {
    host: String,
    port: u16,
    connect_timeout_ms: u64,
    socket_timeout_ms: u64,
    use_tls: bool,
    client_cert: Option<String>,
}

impl Default for RiemannClientOptions {
    fn default() -> RiemannClientOptions {
        Self {
            host: "127.0.0.1".to_owned(),
            port: 5555,
            connect_timeout_ms: 2000,
            socket_timeout_ms: 3000,
            use_tls: false,
            client_cert: None,
        }
    }
}

impl RiemannClientOptions {
    // FIXME: async
    pub(crate) fn to_socket_addr(&self) -> Result<SocketAddr, io::Error> {
        (self.host().as_str(), *self.port())
            .to_socket_addrs()
            .and_then(|mut i| {
                i.next().ok_or_else(|| {
                    io::Error::new(io::ErrorKind::AddrNotAvailable, "Host not found")
                })
            })
    }
}