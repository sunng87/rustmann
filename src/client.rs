use std::io;
use std::net::SocketAddr;
use std::ops::DerefMut;
use std::sync::{Arc, Mutex, MutexGuard};

use derive_builder::Builder;
use futures::future::{self, Either, Loop};
use futures::sync::mpsc;
use futures::sync::oneshot::{self, Sender};
use futures::{Async, Future, Sink, Stream};
use tokio;

use crate::connection::Connection;
use crate::protos::riemann::{Event, Msg};

pub struct Client {
    // connection: Arc<Mutex<Option<Connection>>>,
    queue: mpsc::UnboundedSender<(Vec<Event>, Sender<Result<Msg, io::Error>>)>,
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
        let (tx, rx) = mpsc::unbounded();

        let mut connection: Option<Connection> = None;
        let queue_future = rx.for_each(move |(events, sender)| {
            let (new_conn, result) = future::loop_fn(connection, |conn_opt| {
                if let Some(conn) = conn_opt {
                    return conn
                        .send_events(events, options.socket_timeout_ms)
                        .then(|r| {
                            if r.is_ok() {
                                Ok(Loop::Break((Some(conn), r)))
                            } else {
                                Ok(Loop::Break((None, r)))
                            }
                        });
                } else {
                    return Connection::connect(&options.address, options.connect_timeout_ms).then(
                        |r| match r {
                            Ok(conn) => Ok(Loop::Continue(Some(conn))),
                            Err(_) => Ok(Loop::Continue(None)),
                        },
                    );
                }
            });

            connection = new_conn;
            sender.send(result).map_err(|_| ());
            Ok(())
        });

        Client {
            queue: tx,
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
        let (tx, rx) = oneshot::channel::<Result<Msg, io::Error>>();

        self.queue.send((events, tx));
        rx.map_err(|e| io::Error::new(io::ErrorKind::Other, "Canceled"))
            .and_then(future::result)
    }
}
