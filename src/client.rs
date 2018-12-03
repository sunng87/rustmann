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

pub type RustmannResult = Result<Msg, io::Error>;
pub type RustmannFuture = Box<dyn Future<Item = Msg, Error = io::Error>>;

pub struct Client {
    // connection: Arc<Mutex<Option<Connection>>>,
    queue: mpsc::UnboundedSender<(Vec<Event>, Sender<RustmannResult>)>,
    options: ClientOptions,
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
        let (tx, rx) = mpsc::unbounded();

        let ClientOptions {
            address,
            connect_timeout_ms,
            socket_timeout_ms,
        } = options.clone();

        let mut connection: Option<Connection> = None;
        let queue_future = rx.for_each(
            move |(events, sender): (Vec<Event>, Sender<RustmannResult>)| {
                let looper = future::loop_fn(connection, move |conn_opt| {
                    if let Some(conn) = conn_opt {
                        Either::A(
                            conn.send_events(&events, socket_timeout_ms)
                                .then(|r| match r {
                                    Ok(msg) => {
                                        Ok(Loop::Break((Some(conn), Ok(msg))))
                                    }
                                    Err(e) => Ok(Loop::Break((None, Err(e)))),
                                }),
                        )
                    } else {
                        Either::B(Connection::connect(&address, connect_timeout_ms).then(
                            |r| match r {
                                Ok(conn) => Ok(Loop::Continue(Some(conn))),
                                Err(_) => Ok(Loop::Continue(None)),
                            },
                        ))
                    }
                });

                looper.and_then(|(new_conn, result)| {
                    connection = new_conn;
                    sender.send(result).map_err(|_| ());
                    Ok(())
                })
            },
        );
        tokio::spawn(queue_future);

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

        self.queue.unbounded_send((events, tx));
        rx.map_err(|_| io::Error::new(io::ErrorKind::Other, "Canceled"))
            .and_then(future::result)
    }
}
