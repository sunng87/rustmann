use std::net::SocketAddr;
use tokio::net::{TcpStream, ConnectFuture};
use futures::future::Future;

#[derive(Debug)]
struct Client {
    connection: Future<TcpStream>
}

impl Client {

    pub fn connect(addr: &SocketAddr) -> Client {

    }
}
