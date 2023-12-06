use std::convert::TryFrom;
use std::io;

use rustls_pki_types::ServerName;
use tokio::net::TcpStream;
use tokio_rustls::{Connect, TlsConnector};

use crate::options::RiemannClientOptions;

pub(crate) fn setup_tls_client(
    socket: TcpStream,
    options: &RiemannClientOptions,
) -> Result<Connect<TcpStream>, io::Error> {
    let tls_config = if let Some(tls_config) = options.tls_config() {
        tls_config.clone()
    } else {
        unreachable!("tls_config cannot be None when use_tls is true");
    };
    let connector = TlsConnector::from(tls_config);

    let dns_name = ServerName::try_from(options.host().as_str())
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "Invalid DnsName"))?
        .to_owned();
    Ok(connector.connect(dns_name, socket))
}
