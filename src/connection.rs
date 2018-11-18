use std::io;
use std::net::SocketAddr;

use futures::future;
use futures::stream::SplitSink;
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
    sender_queue: UnboundedSender<Sender<Msg>>,
    socket_sender: SplitSink<Framed<TcpStream, MsgCodec>>,
    // callback_queue: VecDeque<Sender<Msg>>,
}

impl Connection {
    pub fn connect(addr: &SocketAddr) -> impl Future<Item = Connection, Error = io::Error> {
        TcpStream::connect(addr).map(|socket| {
            let framed = Framed::new(socket, MsgCodec);
            let (conn_sender, conn_receiver) = framed.split();

            let (cb_queue_tx, cb_queue_rx) = mpsc::unbounded::<Sender<Msg>>();

            let receiver_loop = conn_receiver
                .zip(
                    cb_queue_rx
                        .map_err(|_| io::Error::new(io::ErrorKind::Other, "callback queue error")),
                )
                .for_each(move |(frame, cb)| {
                    cb.send(frame).map_err(|e| {
                        io::Error::new(
                            io::ErrorKind::Other,
                            format!("failed to deliver msg to callback {:?}", e),
                        )
                    })
                })
                .map_err(|e| eprintln!("{:?}", e));
            tokio::spawn(receiver_loop);

            Connection {
                sender_queue: cb_queue_tx,
                socket_sender: conn_sender,
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

        let send_result = self
            .sender_queue
            .unbounded_send(tx)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))
            .and_then(move |_| {
                self.socket_sender
                    .start_send(msg)
                    .and_then(|_| self.socket_sender.poll_complete())
            });

        future::result(send_result)
            .and_then(|_| rx.map_err(|e| io::Error::new(io::ErrorKind::UnexpectedEof, e)))
    }
}
