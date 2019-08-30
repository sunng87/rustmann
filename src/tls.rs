use std::fs::File;
use std::io::{self, BufReader};
use std::sync::Arc;

use tokio::net::TcpStream;
use tokio_rustls::rustls::ClientConfig;
use tokio_rustls::{Connect, TlsConnector};

use webpki::DNSNameRef;
use webpki_roots;

use crate::options::RiemannClientOptions;

fn setup_tls_config(options: &RiemannClientOptions) -> Result<ClientConfig, io::Error> {
    // FIXME: reuse
    let mut tls_config = ClientConfig::new();
    if let Some(ca_file) = options.client_cert() {
        let mut pem = BufReader::new(File::open(ca_file)?);
        tls_config
            .root_store
            .add_pem_file(&mut pem)
            .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "invalid cert"))?;
    } else {
        tls_config
            .root_store
            .add_server_trust_anchors(&webpki_roots::TLS_SERVER_ROOTS);
    }

    Ok(tls_config)
}

pub(crate) fn setup_tls_client(
    socket: TcpStream,
    options: &RiemannClientOptions,
) -> Result<Connect<TcpStream>, io::Error> {
    let tls_config = setup_tls_config(options)?;
    let connector = TlsConnector::from(Arc::new(tls_config));

    let dns_name = DNSNameRef::try_from_ascii_str(options.host())
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
    Ok(connector.connect(dns_name, socket))
}
