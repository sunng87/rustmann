use std::future::Future;
use std::io;
use std::net::SocketAddr;
use std::ops::DerefMut;
use std::pin::Pin;
use std::str::FromStr;
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll};

use derive_builder::Builder;
use futures_core::future::BoxFuture;
use futures_util::FutureExt;
use protobuf::Chars;

use crate::connection::Connection;
use crate::protos::riemann::{Event, Query};

#[derive(Clone)]
pub struct RiemannClient {
    inner: Inner,
}

#[derive(Clone)]
struct Inner {
    options: RiemannClientOptions,
    state: Arc<Mutex<ClientState>>,
}

enum ClientState {
    Connected(Arc<Mutex<Connection>>),
    Connecting(BoxFuture<'static, Result<Connection, io::Error>>),
    Disconnected,
}

impl Future for Inner {
    type Output = Result<Arc<Mutex<Connection>>, io::Error>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
        let mut inner_state = self.state.lock().unwrap();
        match inner_state.deref_mut() {
            ClientState::Connected(conn) => Poll::Ready(Ok(conn.clone())),
            ClientState::Connecting(ref mut f) => match f.poll_unpin(cx) {
                Poll::Ready(Ok(conn)) => {
                    // connected
                    let conn = Arc::new(Mutex::new(conn));
                    *inner_state = ClientState::Connected(conn.clone());
                    Poll::Ready(Ok(conn.clone()))
                }
                Poll::Ready(Err(e)) => {
                    // failed to connect, reset to disconnected
                    *inner_state = ClientState::Disconnected;
                    Poll::Ready(Err(e))
                }
                Poll::Pending => {
                    // still connecting
                    Poll::Pending
                }
            },
            ClientState::Disconnected => {
                let mut f =
                    Connection::connect(self.options.address, self.options.connect_timeout_ms)
                        .boxed();
                if let Poll::Ready(Ok(conn)) = f.poll_unpin(cx) {
                    let conn_wrapper = Arc::new(Mutex::new(conn));
                    *inner_state = ClientState::Connected(conn_wrapper.clone());
                    Poll::Ready(Ok(conn_wrapper.clone()))
                } else {
                    *inner_state = ClientState::Connecting(f);
                    Poll::Pending
                }
            }
        }
    }
}

#[derive(Debug, Builder, Clone, Copy)]
#[builder(setter(into))]
pub struct RiemannClientOptions {
    address: SocketAddr,
    connect_timeout_ms: u64,
    socket_timeout_ms: u64,
}

impl Default for RiemannClientOptions {
    fn default() -> RiemannClientOptions {
        Self {
            address: SocketAddr::from_str("127.0.0.1:5555").unwrap(),
            connect_timeout_ms: 2000,
            socket_timeout_ms: 3000,
        }
    }
}

impl RiemannClient {
    pub fn new(options: &RiemannClientOptions) -> Self {
        RiemannClient {
            inner: Inner {
                state: Arc::new(Mutex::new(ClientState::Disconnected)),
                options: *options,
            },
        }
    }

    pub async fn send_events(&mut self, events: Vec<Event>) -> Result<bool, io::Error> {
        let timeout = self.inner.options.socket_timeout_ms;
        let state = self.inner.state.clone();
        let inner = &mut self.inner;

        let conn_wrapper = inner.await?;
        let mut conn = conn_wrapper.lock().unwrap();

        conn.send_events(&events, timeout).await.map_err(move |e| {
            *state.lock().unwrap() = ClientState::Disconnected;
            e
        }).map(|msg| msg.get_ok())
    }

    pub async fn send_query(&mut self, query_string: &str) -> Result<Vec<Event>, io::Error> {
        let timeout = self.inner.options.socket_timeout_ms;
        let state = self.inner.state.clone();
        let inner = &mut self.inner;

        let conn_wrapper = inner.await?;
        let mut conn = conn_wrapper.lock().unwrap();

        let mut query = Query::new();
        query.set_string(Chars::from(query_string));

        conn.query(query, timeout)
            .await
            .map_err(move |e| {
                *state.lock().unwrap() = ClientState::Disconnected;
                e
            })
            .map(|msg| Vec::from(msg.get_events()))
    }
}
