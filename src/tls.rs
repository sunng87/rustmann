use std::io;

use tokio::net::TcpStream;
use tokio_rustls::{Connect, TlsConnector};
use webpki::DNSNameRef;

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

    let dns_name = DNSNameRef::try_from_ascii_str(options.host())
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
    Ok(connector.connect(dns_name, socket))
}
