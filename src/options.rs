#[cfg(feature = "tls")]
use std::sync::Arc;

use derive_builder::Builder;
use getset::Getters;
#[cfg(feature = "tls")]
use tokio_rustls::rustls::{ClientConfig, OwnedTrustAnchor, RootCertStore};

/// Riemann connection options
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
    #[cfg(feature = "tls")]
    use_tls: bool,
    #[cfg(feature = "tls")]
    tls_config: Option<Arc<ClientConfig>>,
}

#[cfg(feature = "tls")]
fn default_tls_config() -> Arc<ClientConfig> {
    let mut root_cert_store = RootCertStore::empty();
    root_cert_store.add_server_trust_anchors(webpki_roots::TLS_SERVER_ROOTS.0.iter().map(|ta| {
        OwnedTrustAnchor::from_subject_spki_name_constraints(
            ta.subject,
            ta.spki,
            ta.name_constraints,
        )
    }));

    let tls_config = ClientConfig::builder()
        .with_safe_defaults()
        .with_root_certificates(root_cert_store)
        .with_no_client_auth();
    Arc::new(tls_config)
}

impl RiemannClientOptionsBuilder {
    #[cfg(feature = "tls")]
    fn tls_enabled(&self) -> bool {
        self.use_tls.unwrap_or(false)
    }

    #[cfg(not(feature = "tls"))]
    #[inline]
    fn tls_enabled(&self) -> bool {
        false
    }

    #[cfg(feature = "tls")]
    fn get_tls_config(&self) -> Option<Arc<ClientConfig>> {
        self.tls_config.as_ref().cloned().unwrap_or_else(|| {
            if self.tls_enabled() {
                Some(default_tls_config())
            } else {
                None
            }
        })
    }

    pub fn build(self) -> RiemannClientOptions {
        let use_tls = self.tls_enabled();
        let udp = if use_tls {
            false
        } else {
            self.use_udp.unwrap_or(false)
        };

        RiemannClientOptions {
            host: self.host.clone().unwrap_or_else(|| "127.0.0.1".to_owned()),
            port: self.port.unwrap_or(5555),
            connect_timeout_ms: self.connect_timeout_ms.unwrap_or(2000),
            socket_timeout_ms: self.connect_timeout_ms.unwrap_or(3000),
            use_udp: udp,
            #[cfg(feature = "tls")]
            use_tls,
            #[cfg(feature = "tls")]
            tls_config: self.get_tls_config(),
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
            #[cfg(feature = "tls")]
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
