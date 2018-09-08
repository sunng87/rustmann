use std::net::SocketAddr;
use tokio::net::{TcpStream, ConnectFuture};
use futures::future::Future;

#[derive(Debug)]
struct Client {
    connection: ConnectFuture;
}

impl Client {

    pub fn connect(addr: &SocketAddr) -> Client {
        let cf = TcpStream::connect(addr);
        Client {
            connection: cf
        }
    }
}
