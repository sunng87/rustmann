use std::io;
use std::net::SocketAddr;
use std::sync::Arc;
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
use tokio_rustls::rustls::ClientConfig;
use tokio_rustls::TlsConnector;

use webpki::DNSNameRef;
use webpki_roots;

use crate::codec::MsgCodec;
use crate::options::RiemannClientOptions;
use crate::protos::riemann::{Event, Msg, Query};

#[derive(Debug)]
pub(crate) enum Connection {
    PLAIN(ConnectionInner<TcpStream>),
    TLS(ConnectionInner<TlsStream<TcpStream>>),
}

#[derive(Debug)]
pub(crate) struct ConnectionInner<S>
where
    S: AsyncRead + AsyncWrite,
{
    sender_queue: UnboundedSender<Sender<Msg>>,
    socket_sender: SplitSink<Framed<S, MsgCodec>, Msg>,
}

impl<S> ConnectionInner<S> {
    fn sender_queue_mut(&mut self) -> &mut UnboundedSender<Sender<Msg>> {
        self.sender_queue
    }

    fn socket_sender_mut(&mut self) -> &mut SplitSink<Framed<S, MsgCodec>, Msg> {
        self.socket_sender
    }
}

impl Connection {
    pub(crate) async fn connect(
        addr: SocketAddr,
        options: &RiemannClientOptions,
    ) -> Result<Connection, io::Error> {
        if *options.use_tls() {
            Self::connect_tls(addr, options).await
        } else {
            Self::connect_plain(addr, options).await
        }
    }

    async fn connect_plain(
        addr: SocketAddr,
        options: &RiemannClientOptions,
    ) -> Result<Connection, io::Error> {
        TcpStream::connect(&addr)
            .timeout(Duration::from_millis(*options.connect_timeout_ms()))
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

                Ok(Connection::PLAIN(ConnectionInner {
                    sender_queue: cb_queue_tx,
                    socket_sender: conn_sender,
                }))
            })
    }

    async fn connect_tls(
        addr: SocketAddr,
        options: &RiemannClientOptions,
    ) -> Result<Connection, io::Error> {
        TcpStream::connect(&addr)
            .timeout(Duration::from_millis(*options.connect_timeout_ms()))
            .map_err(|e| io::Error::new(io::ErrorKind::TimedOut, e))
            .await?
            .and_then(|socket| {
                socket.set_nodelay(true)?;

                let tls_config = ClientConfig::new();
                tls_config
                    .root_store
                    .add_server_trust_anchors(&webpki_roots::TLS_SERVER_ROOTS);
                let connector = TlsConnector::from(Arc::new(tls_config));

                let dns_name = DNSNameRef::try_from_ascii_str(options.host())
                    .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
                Ok(connector.connect(dns_name, socket))
            })?
            .await
            .and_then(|socket| {
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

                Ok(Connection::TLS(ConnectionInner {
                    sender_queue: cb_queue_tx,
                    socket_sender: conn_sender,
                }))
            })
    }

    async fn send(&mut self, msg: Msg, socket_timeout: u64) -> Result<Msg, io::Error> {
        let (tx, rx) = oneshot::channel::<Msg>();

        self.sender_queue()
            .send(tx)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))
            .await?;

        self.socket_sender()
            .send(msg)
            .map_err(|e| io::Error::new(io::ErrorKind::UnexpectedEof, e))
            .await?;

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
