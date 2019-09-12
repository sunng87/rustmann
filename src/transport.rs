use std::io;
use std::time::Duration;

use futures_util::stream::SplitSink;
use futures_util::TryFutureExt;
use protobuf::{RepeatedField};
use tokio::codec::{Framed};
use tokio::net::{TcpStream, UdpSocket};
use tokio::prelude::*;
use tokio::sync::mpsc::{self, UnboundedSender};
use tokio::sync::oneshot::{self, Sender};

#[cfg(feature = "tls")]
use tokio_rustls::client::TlsStream;

use crate::codec::{encode_for_udp, MsgCodec};
use crate::options::RiemannClientOptions;
use crate::protos::riemann::{Event, Msg, Query};
#[cfg(feature = "tls")]
use crate::tls::setup_tls_client;

#[derive(Debug)]
pub(crate) enum Transport {
    PLAIN(TcpTransportInner<TcpStream>),
    #[cfg(feature = "tls")]
    TLS(TcpTransportInner<TlsStream<TcpStream>>),
    UDP(UdpTransportInner),
}

#[derive(Debug)]
pub(crate) struct TcpTransportInner<S>
where
    S: AsyncRead + AsyncWrite + Unpin + Send + 'static,
{
    sender_queue: UnboundedSender<Sender<Msg>>,
    socket_sender: SplitSink<Framed<S, MsgCodec>, Msg>,
}

impl<S: AsyncRead + AsyncWrite + Unpin + Send> TcpTransportInner<S> {
    fn sender_queue_mut(&mut self) -> &mut UnboundedSender<Sender<Msg>> {
        &mut self.sender_queue
    }

    fn socket_sender_mut(&mut self) -> &mut SplitSink<Framed<S, MsgCodec>, Msg> {
        &mut self.socket_sender
    }

    fn setup_conn(socket: S) -> TcpTransportInner<S> {
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

        TcpTransportInner {
            sender_queue: cb_queue_tx,
            socket_sender: conn_sender,
        }
    }

    async fn send_for_response(&mut self, msg: Msg, socket_timeout: u64) -> Result<Msg, io::Error> {
        let (tx, rx) = oneshot::channel::<Msg>();
        self.sender_queue_mut()
            .send(tx)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))
            .await?;

        self.socket_sender_mut()
            .send(msg)
            .map_err(|e| io::Error::new(io::ErrorKind::UnexpectedEof, e))
            .await?;

        rx.timeout(Duration::from_millis(socket_timeout))
            .await?
            .map_err(|e| io::Error::new(io::ErrorKind::TimedOut, e))
    }
}

#[derive(Debug)]
pub(crate) struct UdpTransportInner {
    socket: UdpSocket,
}

impl UdpTransportInner {
    async fn new(options: &RiemannClientOptions) -> Result<UdpTransportInner, io::Error> {
        let socket = UdpSocket::bind("0.0.0.0:0").await?;
        socket.connect(options.to_socket_addr_string()).await?;

        Ok(UdpTransportInner {
            socket: socket,
        })
    }

    async fn send_without_response(&mut self, msg: Msg) -> Result<(), io::Error> {
        let buf = encode_for_udp(&msg)?;

        self.socket.send(buf.as_ref()).await.map(|_| ())
    }
}

impl Transport {
    pub(crate) async fn connect(options: RiemannClientOptions) -> Result<Transport, io::Error> {
        if *options.use_tls() {
            #[cfg(feature = "tls")]
            {
                Self::connect_tls(options).await
            }
            #[cfg(not(feature = "tls"))]
            {
                unreachable!("enable tls feature for tls support")
            }
        } else if *options.use_udp() {
            Self::connect_udp(options).await
        } else {
            Self::connect_plain(options).await
        }
    }

    async fn connect_udp(options: RiemannClientOptions) -> Result<Transport, io::Error> {
        let udp_transport = UdpTransportInner::new(&options).await?;
        Ok(Transport::UDP(udp_transport))
    }

    async fn connect_plain(options: RiemannClientOptions) -> Result<Transport, io::Error> {
        let addr = options.to_socket_addr_string();
        TcpStream::connect(&addr)
            .timeout(Duration::from_millis(*options.connect_timeout_ms()))
            .map_err(|e| io::Error::new(io::ErrorKind::TimedOut, e))
            .await?
            .and_then(|socket| {
                socket.set_nodelay(true)?;

                let conn = TcpTransportInner::setup_conn(socket);
                Ok(Transport::PLAIN(conn))
            })
    }

    #[cfg(feature = "tls")]
    async fn connect_tls(options: RiemannClientOptions) -> Result<Transport, io::Error> {
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
                let conn = TcpTransportInner::setup_conn(socket);
                Ok(Transport::TLS(conn))
            })
    }

    pub(crate) async fn send_events(
        &mut self,
        events: &[Event],
        socket_timeout: u64,
    ) -> Result<Msg, io::Error> {
        let mut msg = Msg::new();
        msg.set_events(RepeatedField::from_slice(events));

        match self {
            Transport::PLAIN(ref mut inner) => inner.send_for_response(msg, socket_timeout).await,
            #[cfg(feature = "tls")]
            Transport::TLS(ref mut inner) => inner.send_for_response(msg, socket_timeout).await,
            Transport::UDP(ref mut inner) => {
                inner.send_without_response(msg).await?;
                let mut ok_msg = Msg::new();
                ok_msg.set_ok(true);
                Ok(ok_msg)
            }
        }
    }

    pub(crate) async fn query(
        &mut self,
        query: Query,
        socket_timeout: u64,
    ) -> Result<Msg, io::Error> {
        let mut msg = Msg::new();
        msg.set_query(query);

        match self {
            Transport::PLAIN(ref mut inner) => inner.send_for_response(msg, socket_timeout).await,
            #[cfg(feature = "tls")]
            Transport::TLS(ref mut inner) => inner.send_for_response(msg, socket_timeout).await,
            Transport::UDP(_) => Err(io::Error::new(io::ErrorKind::Other, "Unsupported.")),
        }
    }
}
