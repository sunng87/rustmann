use std::pin::Pin;
use std::task::{Context, Poll};
use std::io;
use std::net::SocketAddr;
use std::ops::DerefMut;
use std::str::FromStr;
use std::sync::{Arc, Mutex};
use std::future::{Future};

use derive_builder::Builder;

use crate::connection::Connection;
use crate::protos::riemann::{Event, Msg};

#[derive(Clone)]
pub struct Client {
    options: ClientOptions,
    state: Arc<Mutex<ClientState>>,
}

enum ClientState {
    Connected(Arc<Mutex<Connection>>),
    Connecting(Box<dyn Future<Output=Result<Connection, io::Error>> + Send>),
    Disconnected,
}

impl Future for Client {
    type Output = Result<Arc<Mutex<Connection>>, io::Error>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
        let mut inner_state = self.state.lock().unwrap();
        match inner_state.deref_mut() {
            ClientState::Connected(conn) => Ok(Poll::Ready(conn.clone())),
            ClientState::Connecting(ref mut f) => match f.poll() {
                Ok(Poll::Ready(conn)) => {
                    // connected
                    let conn = Arc::new(Mutex::new(conn));
                    *inner_state = ClientState::Connected(conn.clone());
                    Ok(Poll::Ready(conn.clone()))
                }
                Ok(Poll::Pending) => {
                    // still connecting
                    Ok(Poll::Pending)
                }
                Err(e) => {
                    // failed to connect, reset to disconnected
                    *inner_state = ClientState::Disconnected;
                    Err(e)
                }
            },
            ClientState::Disconnected => {
                let mut f =
                    Connection::connect(&self.options.address, self.options.connect_timeout_ms);
                if let Poll::Ready(conn) = f.poll()? {
                    let conn_wrapper = Arc::new(Mutex::new(conn));
                    *inner_state = ClientState::Connected(conn_wrapper.clone());
                    Ok(Poll::Ready(conn_wrapper.clone()))
                } else {
                    *inner_state = ClientState::Connecting(Box::new(f));
                    Ok(Poll::Pending)
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
            options: *options,
        }
    }

    pub async fn send_events(&mut self, events: Vec<Event>) -> Result<Msg, io::Error> {
        let timeout = self.options.socket_timeout_ms;
        let state = self.state.clone();

        let conn = self.await?;

        conn.lock()
            .unwrap()
            .send_events(&events, timeout)
            .await
            .map_err(move |e| {
                *state.lock().unwrap() = ClientState::Disconnected;
                e
            })
    }
}
