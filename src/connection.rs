use std::io;
use std::time::Duration;

use futures_util::stream::SplitSink;
use futures_util::TryFutureExt;
use protobuf::RepeatedField;
use tokio::codec::Framed;
use tokio::net::TcpStream;
use tokio::prelude::*;
use tokio::sync::mpsc::{self, UnboundedSender};
use tokio::sync::oneshot::{self, Sender};

use tokio_rustls::client::TlsStream;

use crate::codec::MsgCodec;
use crate::options::RiemannClientOptions;
use crate::protos::riemann::{Event, Msg, Query};
use crate::tls::setup_tls_client;

#[derive(Debug)]
pub(crate) enum Connection {
    PLAIN(ConnectionInner<TcpStream>),
    TLS(ConnectionInner<TlsStream<TcpStream>>),
}

#[derive(Debug)]
pub(crate) struct ConnectionInner<S>
where
    S: AsyncRead + AsyncWrite + Unpin + Send + 'static,
{
    sender_queue: UnboundedSender<Sender<Msg>>,
    socket_sender: SplitSink<Framed<S, MsgCodec>, Msg>,
}

impl<S: AsyncRead + AsyncWrite + Unpin + Send> ConnectionInner<S> {
    fn sender_queue_mut(&mut self) -> &mut UnboundedSender<Sender<Msg>> {
        &mut self.sender_queue
    }

    fn socket_sender_mut(&mut self) -> &mut SplitSink<Framed<S, MsgCodec>, Msg> {
        &mut self.socket_sender
    }
}

impl Connection {
    pub(crate) async fn connect(options: RiemannClientOptions) -> Result<Connection, io::Error> {
        if *options.use_tls() {
            Self::connect_tls(options).await
        } else {
            Self::connect_plain(options).await
        }
    }

    fn setup_conn<S>(socket: S) -> ConnectionInner<S>
    where
        S: AsyncRead + AsyncWrite + Unpin + Send,
    {
        let framed = Framed::new(socket, MsgCodec);
        let (conn_sender, mut conn_receiver) = framed.split();
        let (cb_queue_tx, mut cb_queue_rx) = mpsc::unbounded_channel::<Sender<Msg>>();

        let receiver_loop = async move {
            loop {
                let frame = conn_receiver.next().await;
                let cb = cb_queue_rx.recv().await;
                if let (Some(Ok(frame)), Some(cb)) = (frame, cb) {
                    let r = cb.send(frame);
                    if let Err(_) = r {
                        // eprintln!("failed to deliver msg to callback {:?}", e);
                        break;
                    }
                } else {
                    // eprintln!("failed to deliver msg to callback.");
                    break;
                }
            }
        };
        tokio::spawn(receiver_loop);

        ConnectionInner {
            sender_queue: cb_queue_tx,
            socket_sender: conn_sender,
        }
    }

    async fn connect_plain(options: RiemannClientOptions) -> Result<Connection, io::Error> {
        let addr = options.to_socket_addr_string();
        TcpStream::connect(&addr)
            .timeout(Duration::from_millis(*options.connect_timeout_ms()))
            .map_err(|e| io::Error::new(io::ErrorKind::TimedOut, e))
            .await?
            .and_then(|socket| {
                socket.set_nodelay(true)?;

                let conn = Self::setup_conn(socket);
                Ok(Connection::PLAIN(conn))
            })
    }

    async fn connect_tls(options: RiemannClientOptions) -> Result<Connection, io::Error> {
        let addr = options.to_socket_addr_string();
        TcpStream::connect(&addr)
            .timeout(Duration::from_millis(*options.connect_timeout_ms()))
            .map_err(|e| io::Error::new(io::ErrorKind::TimedOut, e))
            .await?
            .and_then(|socket| {
                socket.set_nodelay(true)?;

                setup_tls_client(socket, &options)
            })?
            .await
            .and_then(|socket| {
                let conn = Self::setup_conn(socket);
                Ok(Connection::TLS(conn))
            })
    }

    async fn send(&mut self, msg: Msg, socket_timeout: u64) -> Result<Msg, io::Error> {
        let (tx, rx) = oneshot::channel::<Msg>();

        match self {
            Connection::PLAIN(ref mut inner) => {
                inner
                    .sender_queue_mut()
                    .send(tx)
                    .map_err(|e| io::Error::new(io::ErrorKind::Other, e))
                    .await?;

                inner
                    .socket_sender_mut()
                    .send(msg)
                    .map_err(|e| io::Error::new(io::ErrorKind::UnexpectedEof, e))
                    .await?;
            }
            Connection::TLS(ref mut inner) => {
                inner
                    .sender_queue_mut()
                    .send(tx)
                    .map_err(|e| io::Error::new(io::ErrorKind::Other, e))
                    .await?;

                inner
                    .socket_sender_mut()
                    .send(msg)
                    .map_err(|e| io::Error::new(io::ErrorKind::UnexpectedEof, e))
                    .await?;
            }
        }

        rx.timeout(Duration::from_millis(socket_timeout))
            .await?
            .map_err(|e| io::Error::new(io::ErrorKind::TimedOut, e))
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
