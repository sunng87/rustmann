use std::borrow::Borrow;
use std::io;
use std::net::SocketAddr;
use std::ops::DerefMut;
use std::sync::{Arc, Mutex, MutexGuard};

use derive_builder::Builder;
use futures::future::{self, Either, Loop};
use futures::sync::mpsc;
use futures::sync::oneshot::{self, Sender};
use futures::{Async, Future, Poll, Sink, Stream};
use tokio;

use crate::connection::Connection;
use crate::protos::riemann::{Event, Msg};

pub type RustmannResult = Result<Msg, io::Error>;
pub type RustmannFuture = Box<dyn Future<Item = Msg, Error = io::Error>>;

pub struct Client {
    // connection: Arc<Mutex<Option<Connection>>>,
    options: ClientOptions,
    state: ClientInnerState,
}

pub struct ClientInnerState {
    inner: Arc<Mutex<ClientState>>,
}

enum ClientState {
    Connected(Connection),
    Connecting(Box<Future<Item = Connection, Error = io::Error>>),
    Disconnected,
}

impl Future for ClientInnerState {
    type Item = Connection;
    type Error = io::Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        let mut inner_state = self.inner.lock().unwrap();
        match inner_state {
            ClientState::Connected(ref conn) => Ok(Async::Ready(conn)),
            ClientState::Connecting(f) => {
                match f.poll() {
                    Ok(Async::Ready(conn)) => {
                        // TODO: transform state
                        *inner_state = ClientState::Connected(conn);
                        Ok(Async::Ready(conn))
                    }
                    Ok(Async::NotReady) => Ok(Async::NotReady),
                    Err(e) => Err(e),
                }
            }
            ClientState::Disconnected => {
                *inner_state = ClientState::Connecting(Connection::new(/*TODO*/));
                Ok(Async::NotReady)
            }
        }
    }
}

#[derive(Debug, Builder, Clone)]
#[builder(setter(into))]
pub struct ClientOptions {
    address: SocketAddr,
    connect_timeout_ms: u64,
    socket_timeout_ms: u64,
}

impl Client {
    pub fn new(options: &ClientOptions) -> Self {
        Client {
            state: ClientInnerState {
                inner: Arc::new(Mutex::new(ClientState::Disconnected)),
            },
            options: options.clone(),
        }
    }

    // // a private method to get or create a connection
    // // this implement would block the thread
    // fn get_connection(&mut self) -> Result<MutexGuard<Option<Connection>>, io::Error> {
    //     let conn = self.connection.clone();

    //     let conn_opt = conn.lock().unwrap();
    //     if conn_opt.is_some() {
    //         Ok(conn_opt)
    //     } else {
    //         let new_conn =
    //             Connection::connect(&self.options.address, self.options.connect_timeout_ms)
    //                 .wait()?;
    //         *conn_opt = Some(new_conn);
    //         Ok(conn_opt)
    //     }
    // }

    pub fn send_events(&self, events: Vec<Event>) -> impl Future<Item = Msg, Error = io::Error> {
        // TODO: on error
        self.state
            .and_then(|conn| conn.send_events(&events, self.options.socket_timeout_ms))
    }
}
