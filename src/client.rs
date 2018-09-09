use std::io;
use std::net::SocketAddr;

use futures::{Future, Sink};
use tokio::codec::{Decoder, Framed};
use tokio::net::TcpStream;

use codec::MsgCodec;
use protos::riemann::{Event, Msg};

#[derive(Debug)]
struct Client {
    connection: Framed<TcpStream, MsgCodec>,
}

impl Client {
    pub fn connect(addr: &SocketAddr) -> impl Future<Item = Client, Error = io::Error> {
        TcpStream::connect(addr)
            .map(|socket| MsgCodec.framed(socket))
            .map(|cf| Client { connection: cf })
    }

    pub fn sendEvents(&self, events: &Vec<Event>) -> impl Future<Item = Msg, Error = io::Error> {
        let msg = Msg::new();
        self.connection.send(msg)
    }
}
