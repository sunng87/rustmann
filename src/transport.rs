use std::io;
use std::time::Duration;

use futures::stream::{SplitSink, StreamExt};
use futures::{SinkExt, TryFutureExt};
use tokio::net::{TcpStream, UdpSocket};
use tokio::prelude::*;
use tokio::sync::mpsc::{self, UnboundedSender};
use tokio::sync::oneshot::{self, Sender};
use tokio::time::timeout;
use tokio_util::codec::Framed;

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
    fn setup_conn(socket: S) -> TcpTransportInner<S> {
        let framed = Framed::new(socket, MsgCodec::default());
        let (conn_sender, mut conn_receiver) = framed.split();
        let (cb_queue_tx, mut cb_queue_rx) = mpsc::unbounded_channel::<Sender<Msg>>();

        let receiver_loop = async move {
            loop {
                let frame = conn_receiver.next().await;
                let cb = cb_queue_rx.recv().await;
                if let (Some(Ok(frame)), Some(cb)) = (frame, cb) {
                    let r = cb.send(frame);
                    if r.is_err() {
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
        self.sender_queue
            .send(tx)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

        self.socket_sender
            .send(msg)
            .map_err(|e| io::Error::new(io::ErrorKind::UnexpectedEof, e))
            .await?;

        timeout(Duration::from_millis(socket_timeout), rx)
            .await?
            .map_err(|e| io::Error::new(io::ErrorKind::BrokenPipe, e))
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

        Ok(UdpTransportInner { socket })
    }

    async fn send_without_response(&mut self, msg: Msg) -> Result<(), io::Error> {
        let buf = encode_for_udp(&msg)?;

        self.socket.send(buf.as_ref()).await.map(|_| ())
    }
}

impl Transport {
    pub(crate) async fn connect(options: RiemannClientOptions) -> Result<Transport, io::Error> {
        #[cfg(feature = "tls")]
        {
            if *options.use_tls() {
                return Self::connect_tls(options).await;
            }
        }

        if *options.use_udp() {
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
        timeout(
            Duration::from_millis(*options.connect_timeout_ms()),
            TcpStream::connect(addr),
        )
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
        timeout(
            Duration::from_millis(*options.connect_timeout_ms()),
            TcpStream::connect(addr),
        )
        .map_err(|e| io::Error::new(io::ErrorKind::TimedOut, e))
        .await?
        .and_then(|socket| {
            socket.set_nodelay(true)?;

            setup_tls_client(socket, &options)
        })?
        .await
        .map(|socket| {
            let conn = TcpTransportInner::setup_conn(socket);
            Transport::TLS(conn)
        })
    }

    pub(crate) async fn send_events(
        &mut self,
        events: Vec<Event>,
        socket_timeout: u64,
    ) -> Result<Msg, io::Error> {
        let msg = Msg {
            events,
            ..Default::default()
        };

        match self {
            Transport::PLAIN(ref mut inner) => inner.send_for_response(msg, socket_timeout).await,
            #[cfg(feature = "tls")]
            Transport::TLS(ref mut inner) => inner.send_for_response(msg, socket_timeout).await,
            Transport::UDP(ref mut inner) => {
                inner.send_without_response(msg).await?;
                let ok_msg = Msg {
                    ok: Some(true),
                    ..Default::default()
                };
                Ok(ok_msg)
            }
        }
    }

    pub(crate) async fn query(
        &mut self,
        query: Query,
        socket_timeout: u64,
    ) -> Result<Msg, io::Error> {
        let msg = Msg {
            query: Some(query),
            ..Default::default()
        };

        match self {
            Transport::PLAIN(ref mut inner) => inner.send_for_response(msg, socket_timeout).await,
            #[cfg(feature = "tls")]
            Transport::TLS(ref mut inner) => inner.send_for_response(msg, socket_timeout).await,
            Transport::UDP(_) => Err(io::Error::new(io::ErrorKind::Other, "Unsupported.")),
        }
    }
}
