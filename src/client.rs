use std::io;
use std::net::SocketAddr;
use std::ops::DerefMut;
use std::sync::{Arc, Mutex};

use derive_builder::Builder;
use futures::future;
use futures::{Async, Future, Sink, Stream};
use tokio;

use crate::connection::Connection;
use crate::protos::riemann::{Event, Msg};

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

pub type RustmannFuture = Box<dyn Future<Item = Msg, Error = io::Error>>;

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

        let conn_lock = conn.lock();
        if let Ok(mut conn_opt_ref) = conn_lock {
            let conn_opt = conn_opt_ref.deref_mut();
            if let Some(conn_ref) = conn_opt {
                let fut = conn_ref.send_events(events, read_timeout);
                // TODO: error handling
                Box::new(fut)
            } else {
                // TODO: modify client
                let conn_result =
                    Connection::connect(&self.options.address, self.options.connect_timeout_ms)
                        .wait();
                match conn_result {
                    Ok(mut new_conn) => {
                        *conn_opt = Some(new_conn);
                        Box::new(new_conn.send_events(events, read_timeout))
                    }
                    Err(e) => Box::new(future::err(e)),
                }
            }
        } else {
            Box::new(future::err(io::Error::new(
                io::ErrorKind::Other,
                "Can not get lock for client connection.",
            )))
        }
    }
}
