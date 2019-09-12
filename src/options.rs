#[cfg(feature = "tls")]
use std::sync::Arc;

use derive_builder::Builder;
use getset::Getters;
#[cfg(feature = "tls")]
use tokio_rustls::rustls::ClientConfig;

#[derive(Builder, Clone, Getters)]
#[builder(setter(into))]
#[builder(build_fn(skip))]
#[builder(pattern = "owned")]
#[get = "pub"]
pub struct RiemannClientOptions {
    host: String,
    port: u16,
    connect_timeout_ms: u64,
    socket_timeout_ms: u64,
    use_udp: bool,
    use_tls: bool,
    #[cfg(feature = "tls")]
    tls_config: Option<Arc<ClientConfig>>,
}

impl RiemannClientOptionsBuilder {
    pub fn build(self) -> RiemannClientOptions {
        let use_tls = self.use_tls.unwrap_or(false);
        #[cfg(feature = "tls")]
        let tls_config = self.tls_config.unwrap_or_else(|| {
            if use_tls {
                let mut tls_config = ClientConfig::new();
                tls_config
                    .root_store
                    .add_server_trust_anchors(&webpki_roots::TLS_SERVER_ROOTS);
                Some(Arc::new(tls_config))
            } else {
                None
            }
        });

        let udp = if use_tls {
            false
        } else {
            self.use_udp.unwrap_or(false)
        };

        RiemannClientOptions {
            host: self.host.unwrap_or_else(|| "127.0.0.1".to_owned()),
            port: self.port.unwrap_or(5555),
            connect_timeout_ms: self.connect_timeout_ms.unwrap_or(2000),
            socket_timeout_ms: self.connect_timeout_ms.unwrap_or(3000),
            use_udp: udp,
            use_tls: use_tls,
            #[cfg(feature = "tls")]
            tls_config: tls_config,
        }
    }
}

impl Default for RiemannClientOptions {
    fn default() -> RiemannClientOptions {
        Self {
            host: "127.0.0.1".to_owned(),
            port: 5555,
            connect_timeout_ms: 2000,
            socket_timeout_ms: 3000,
            use_udp: false,
            use_tls: false,
            #[cfg(feature = "tls")]
            tls_config: None,
        }
    }
}

impl RiemannClientOptions {
    pub(crate) fn to_socket_addr_string(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }
}
