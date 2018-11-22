use std::collections::VecDeque;
use std::io;
use std::net::SocketAddr;
use std::sync::{Arc, RwLock};

use derive_builder::Builder;
use futures::sync::mpsc::{self, UnboundedSender};
use futures::sync::oneshot::{self, Sender};
use futures::{Future, Sink, Stream};
use protobuf::RepeatedField;
use tokio;
use tokio::codec::Framed;
use tokio::net::TcpStream;

use crate::connection::Connection;
use crate::protos::riemann::{Event, Msg};

#[derive(Debug)]
pub struct Client {
    // connection: Arc<Connection>,
}

#[derive(Debug, Builder)]
#[builder(setter(into))]
pub struct ClientOptions {
    address: SocketAddr,
    connect_timeout_ms: u64,
    socket_timeout_ms: u64,
}

impl Client {
    pub fn new(options: &ClientOptions) -> Client {
        Client {}
    }
}
