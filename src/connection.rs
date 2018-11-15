use std::collections::VecDeque;
use std::io;
use std::net::SocketAddr;
use std::sync::{Arc, RwLock};

use futures::sync::mpsc::{self, UnboundedSender};
use futures::sync::oneshot::{self, Sender};
use futures::{Future, Sink, Stream};
use protobuf::RepeatedField;
use tokio;
use tokio::codec::Framed;
use tokio::net::TcpStream;

use codec::MsgCodec;
use protos::riemann::{Event, Msg};

#[derive(Debug)]
pub struct Connection {
    sender_queue: UnboundedSender<(Msg, Sender<Msg>)>,
    // callback_queue: VecDeque<Sender<Msg>>,
}

impl Connection {
    pub fn connect(addr: &SocketAddr) -> impl Future<Item = Connection, Error = io::Error> {
        TcpStream::connect(addr).map(|socket| {
            let framed = Framed::new(socket, MsgCodec);
            let (mut conn_sender, conn_receiver) = framed.split();
            let callback_queue: Arc<RwLock<VecDeque<Sender<Msg>>>> =
                Arc::new(RwLock::new(VecDeque::new()));

            let cq1 = callback_queue.clone();
            let f = conn_receiver
                .for_each(move |frame| {
                    if let Some(inner_tx) = cq1.write().unwrap().pop_front() {
                        if let Err(e) = inner_tx.send(frame) {
                            eprintln!("{:?}", e);
                        };
                    }
                    Ok(())
                })
                .map_err(|e| eprintln!("{:?}", e));
            tokio::spawn(f);

            let (sender_queue, receiver) = mpsc::unbounded();
            let f2 = receiver.for_each(move |(msg, callback)| {
                callback_queue.write().unwrap().push_back(callback);
                if let Err(e) = conn_sender
                    .start_send(msg)
                    .and_then(|_| conn_sender.poll_complete())
                {
                    eprintln!("{:?}", e);
                };
                Ok(())
            });
            tokio::spawn(f2);

            Client {
                sender_queue,
                // callback_queue,
            }
        })
    }

    pub fn send_events(
        &mut self,
        events: Vec<Event>,
    ) -> impl Future<Item = Msg, Error = io::Error> {
        let mut msg = Msg::new();
        msg.set_events(RepeatedField::from_vec(events));

        let (tx, rx) = oneshot::channel::<Msg>();
        if let Err(e) = self
            .sender_queue
            .start_send((msg, tx))
            .and_then(|_| self.sender_queue.poll_complete())
        {
            eprintln!("{:?}", e);
        };
        rx.map_err(|e| io::Error::new(io::ErrorKind::UnexpectedEof, e))
    }
}
