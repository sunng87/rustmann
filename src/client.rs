use std::collections::VecDeque;
use std::io;
use std::net::SocketAddr;

use futures::future;
use futures::sync::mpsc::{self, UnboundedSender};
use futures::sync::oneshot::{self, Sender};
use futures::{Async, Future, Sink, Stream};
use protobuf::RepeatedField;
use tokio;
use tokio::codec::{Decoder, Framed};
use tokio::net::TcpStream;

use codec::MsgCodec;
use protos::riemann::{Event, Msg};

#[derive(Debug)]
pub struct Client {
    sender_queue: UnboundedSender<(Msg, Sender<Msg>)>,
    // callback_queue: VecDeque<Sender<Msg>>,
}

impl Client {
    pub fn connect(addr: &SocketAddr) -> impl Future<Item = Client, Error = io::Error> {
        TcpStream::connect(addr).map(|socket| {
            let framed = Framed::new(socket, MsgCodec);
            let (mut conn_sender, conn_receiver) = framed.split();
            let mut callback_queue: VecDeque<Sender<Msg>> = VecDeque::new();
            let f = conn_receiver.for_each(move |frame| {
                if let Some(inner_tx) = callback_queue.pop_front() {
                    inner_tx.send(frame);
                }
                Ok(())
            }).map_err(|e| eprintln!("{:?}", e));
            tokio::spawn(f);

            let (sender_queue, receiver) = mpsc::unbounded();
            receiver.for_each(move |(msg, callback)| {
                callback_queue.push_back(callback);
                conn_sender.start_send(msg);
                conn_sender.poll_complete();
                Ok(())
            });

            Client {
                sender_queue,
                // callback_queue,
            }
        })
    }

    pub fn send_events(&mut self, events: Vec<Event>) -> impl Future<Item = Msg, Error = io::Error> {
        let mut msg = Msg::new();
        msg.set_events(RepeatedField::from_vec(events));

        let (tx, rx) = oneshot::channel::<Msg>();
        self.sender_queue.start_send((msg, tx));
        self.sender_queue.poll_complete();
        rx.map_err(|e| io::Error::new(io::ErrorKind::UnexpectedEof, e))
    }
}
