use std::future::Future;
use std::io;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};

use futures::future::BoxFuture;
use futures::FutureExt;

use crate::error::RiemannClientError;
use crate::options::RiemannClientOptions;
use crate::transport::Transport;

pub(crate) enum ClientState {
    Connected(Arc<Transport>),
    Connecting(BoxFuture<'static, Result<Transport, io::Error>>),
    Disconnected,
}

pub(crate) struct Inner {
    pub(crate) options: RiemannClientOptions,
    pub(crate) state: ClientState,
}

impl Future for Inner {
    type Output = Result<Arc<Transport>, RiemannClientError>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
        match &mut self.state {
            ClientState::Connected(conn) => Poll::Ready(Ok(conn.clone())),
            ClientState::Connecting(ref mut f) => match f.poll_unpin(cx) {
                Poll::Ready(Ok(conn)) => {
                    // connected
                    let connection = Arc::new(conn);
                    self.state = ClientState::Connected(connection.clone());
                    Poll::Ready(Ok(connection.clone()))
                }
                Poll::Ready(Err(e)) => {
                    // failed to connect, reset to disconnected
                    self.state = ClientState::Disconnected;
                    Poll::Ready(Err(RiemannClientError::from(e)))
                }
                Poll::Pending => {
                    // still connecting
                    Poll::Pending
                }
            },
            ClientState::Disconnected => {
                let f = Transport::connect(self.options.clone()).boxed();
                self.state = ClientState::Connecting(f);
                cx.waker().clone().wake();
                Poll::Pending
            }
        }
    }
}
