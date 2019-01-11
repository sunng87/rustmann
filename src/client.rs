use std::io;
use std::net::SocketAddr;
use std::ops::DerefMut;
use std::str::FromStr;
use std::sync::{Arc, Mutex};
//use std::future::{Future, Poll};

use derive_builder::Builder;
use futures::{try_ready, Async, Future, Poll};

use crate::connection::Connection;
use crate::protos::riemann::{Event, Msg};

pub struct Client {
    // connection: Arc<Mutex<Option<Connection>>>,
    options: ClientOptions,
    state: Arc<Mutex<ClientState>>,
}

enum ClientState {
    Connected(Arc<Mutex<Connection>>),
    Connecting(Box<Future<Item = Connection, Error = io::Error> + Send>),
    Disconnected,
}

impl Future for Client {
    type Item = Arc<Mutex<Connection>>;
    type Error = io::Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        let mut inner_state = self.state.lock().unwrap();
        match inner_state.deref_mut() {
            ClientState::Connected(conn) => Ok(Async::Ready(conn.clone())),
            ClientState::Connecting(ref mut f) => {
                let conn = Arc::new(Mutex::new(try_ready!(f.poll())));
                *inner_state = ClientState::Connected(conn.clone());
                Ok(Async::Ready(conn.clone()))
            }
            ClientState::Disconnected => {
                let mut f = Connection::connect(&self.options.address, self.options.connect_timeout_ms);
                if let Async::Ready(conn) = f.poll()? {
                    let conn_wrapper = Arc::new(Mutex::new(conn));
                    *inner_state = ClientState::Connected(conn_wrapper.clone());
                    Ok(Async::Ready(conn_wrapper.clone()))
                } else {
                    *inner_state = ClientState::Connecting(Box::new(f));
                    Ok(Async::NotReady)
                }
            }
        }
    }
}

#[derive(Debug, Builder, Clone, Copy)]
#[builder(setter(into))]
pub struct ClientOptions {
    address: SocketAddr,
    connect_timeout_ms: u64,
    socket_timeout_ms: u64,
}

impl Default for ClientOptions {
    fn default() -> ClientOptions {
        ClientOptions {
            address: SocketAddr::from_str("127.0.0.1:5555").unwrap(),
            connect_timeout_ms: 2000,
            socket_timeout_ms: 3000,
        }
    }
}

impl Client {
    pub fn new(options: &ClientOptions) -> Self {
        Client {
            state: Arc::new(Mutex::new(ClientState::Disconnected)),
            options: options.clone(),
        }
    }

    pub async fn send_events(&mut self, events: Vec<Event>) -> Result<Msg, io::Error> {
        let timeout = self.options.socket_timeout_ms;
        let state = self.state.clone();

        let conn = await!(self)?;

        await!(conn.lock().unwrap().send_events(&events, timeout))
            .map_err(move |e| {
                *state.lock().unwrap() = ClientState::Disconnected;
                e
            })
    }
}
