use std::collections::VecDeque;
use std::io;
use std::net::SocketAddr;
use std::ops::Deref;
use std::sync::{Arc, Mutex};

use derive_builder::Builder;
use futures::future::{self, IntoFuture};
use futures::sync::mpsc::{self, UnboundedSender};
use futures::sync::oneshot::{self, Sender};
use futures::{Async, Future, Sink, Stream};
use protobuf::RepeatedField;
use tokio;
use tokio::codec::Framed;
use tokio::net::TcpStream;

use crate::connection::Connection;
use crate::protos::riemann::{Event, Msg};

#[derive(Debug)]
pub struct Client {
    connection: Arc<Mutex<Option<Connection>>>,

    options: ClientOptions,
}

#[derive(Debug, Builder, Clone)]
#[builder(setter(into))]
pub struct ClientOptions {
    address: SocketAddr,
    connect_timeout_ms: u64,
    socket_timeout_ms: u64,
}

pub struct RustmannFuture {
    inner: Box<Future<Item = Msg, Error = io::Error>>,
}

impl RustmannFuture {
    pub fn new<F>(f: F) -> RustmannFuture
    where
        F: Future<Item = Msg, Error = io::Error> + 'static,
    {
        RustmannFuture { inner: Box::new(f) }
    }
}

impl Future for RustmannFuture {
    type Item = Msg;
    type Error = io::Error;

    fn poll(&mut self) -> Result<Async<Self::Item>, Self::Error> {
        self.inner.poll()
    }
}

impl Client {
    pub fn new(options: &ClientOptions) -> Client {
        Client {
            connection: Arc::new(Mutex::new(None)),
            options: options.clone(),
        }
    }

    pub fn send_events(&mut self, events: Vec<Event>) -> RustmannFuture {
        let conn = self.connection.clone();
        let read_timeout = self.options.socket_timeout_ms;

        if let Ok(conn_ref) = conn.lock() {
            if let Some(mut conn_ref) = *conn_ref {
                // TODO: error handling
                RustmannFuture::new(conn_ref.send_events(events, read_timeout))
            } else {
                RustmannFuture::new(
                    // TODO: modify client
                    Connection::connect(&self.options.address, self.options.connect_timeout_ms)
                        .and_then(|mut conn| conn.send_events(events, read_timeout)),
                )
            }
        } else {
            RustmannFuture::new(future::err(io::Error::new(
                io::ErrorKind::Other,
                "Can not get lock for client connection.",
            )))
        }
    }
}
