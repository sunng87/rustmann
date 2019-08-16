use std::io;
use std::net::SocketAddr;
use std::time::Duration;

use futures_util::stream::SplitSink;
use futures_util::TryFutureExt;
use protobuf::RepeatedField;
use tokio::codec::Framed;
use tokio::net::TcpStream;
use tokio::prelude::*;
use tokio::sync::mpsc::{self, UnboundedSender};
use tokio::sync::oneshot::{self, Sender};

use crate::codec::MsgCodec;
use crate::protos::riemann::{Event, Msg, Query};

#[derive(Debug)]
pub struct Connection {
    sender_queue: UnboundedSender<Sender<Msg>>,
    socket_sender: SplitSink<Framed<TcpStream, MsgCodec>, Msg>,
}

impl Connection {
    pub(crate) async fn connect(
        addr: SocketAddr,
        connect_timeout_ms: u64,
    ) -> Result<Connection, io::Error> {
        TcpStream::connect(&addr)
            .timeout(Duration::from_millis(connect_timeout_ms))
            .map_err(|e| io::Error::new(io::ErrorKind::TimedOut, e))
            .await?
            .and_then(|socket| {
                socket.set_nodelay(true)?;

                let framed = Framed::new(socket, MsgCodec);
                let (conn_sender, mut conn_receiver) = framed.split();
                let (cb_queue_tx, mut cb_queue_rx) = mpsc::unbounded_channel::<Sender<Msg>>();

                let receiver_loop = async move {
                    loop {
                        let frame = conn_receiver.next().await;
                        let cb = cb_queue_rx.recv().await;
                        if let (Some(Ok(frame)), Some(cb)) = (frame, cb) {
                            let r = cb.send(frame);
                            if let Err(e) = r {
                                eprintln!("failed to deliver msg to callback {:?}", e);
                            }
                        } else {
                            eprintln!("failed to deliver msg to callback.");
                        }
                    }
                };
                tokio::spawn(receiver_loop);

                Ok(Connection {
                    sender_queue: cb_queue_tx,
                    socket_sender: conn_sender,
                })
            })
    }

    async fn send(&mut self, msg: Msg, socket_timeout: u64) -> Result<Msg, io::Error> {
        let (tx, rx) = oneshot::channel::<Msg>();

        self.sender_queue
            .send(tx)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))
            .await?;

        self.socket_sender
            .send(msg)
            .map_err(|e| io::Error::new(io::ErrorKind::UnexpectedEof, e))
            .await?;

        let result = rx
            .timeout(Duration::from_millis(socket_timeout))
            .map_err(|e| io::Error::new(io::ErrorKind::TimedOut, e))
            .await?;

        result.map_err(|e| io::Error::new(io::ErrorKind::Other, e))
    }

    pub(crate) async fn send_events(
        &mut self,
        events: &[Event],
        socket_timeout: u64,
    ) -> Result<Msg, io::Error> {
        let mut msg = Msg::new();
        msg.set_events(RepeatedField::from_slice(events));

        self.send(msg, socket_timeout).await
    }

    pub(crate) async fn query(
        &mut self,
        query: Query,
        socket_timeout: u64,
    ) -> Result<Msg, io::Error> {
        let mut msg = Msg::new();
        msg.set_query(query);

        self.send(msg, socket_timeout).await
    }
}
