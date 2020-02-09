use std::future::Future;
use std::io;
use std::ops::DerefMut;
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll};

use futures_core::future::BoxFuture;
use futures_util::FutureExt;

use crate::error::RiemannClientError;
use crate::options::RiemannClientOptions;
use crate::transport::Transport;

pub(crate) enum ClientState {
    Connected(Arc<Mutex<Transport>>),
    Connecting(BoxFuture<'static, Result<Transport, io::Error>>),
    Disconnected,
}

#[derive(Clone)]
pub(crate) struct Inner {
    pub(crate) options: RiemannClientOptions,
    pub(crate) state: Arc<Mutex<ClientState>>,
}

impl Future for Inner {
    type Output = Result<Arc<Mutex<Transport>>, RiemannClientError>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
        let mut inner_state = self.state.lock().unwrap();
        match inner_state.deref_mut() {
            ClientState::Connected(conn) => Poll::Ready(Ok(conn.clone())),
            ClientState::Connecting(ref mut f) => match f.poll_unpin(cx) {
                Poll::Ready(Ok(conn)) => {
                    // connected
                    let conn = Arc::new(Mutex::new(conn));
                    *inner_state = ClientState::Connected(conn.clone());
                    Poll::Ready(Ok(conn))
                }
                Poll::Ready(Err(e)) => {
                    // failed to connect, reset to disconnected
                    *inner_state = ClientState::Disconnected;
                    Poll::Ready(Err(RiemannClientError::from(e)))
                }
                Poll::Pending => {
                    // still connecting
                    Poll::Pending
                }
            },
            ClientState::Disconnected => {
                let mut f = Transport::connect(self.options.clone()).boxed();

                // FIXME: avoid dup code
                match f.poll_unpin(cx) {
                    Poll::Ready(Ok(conn)) => {
                        // connected
                        let conn = Arc::new(Mutex::new(conn));
                        *inner_state = ClientState::Connected(conn.clone());
                        Poll::Ready(Ok(conn))
                    }
                    Poll::Ready(Err(e)) => {
                        // failed to connect, return error
                        Poll::Ready(Err(RiemannClientError::from(e)))
                    }
                    Poll::Pending => {
                        // still connecting
                        *inner_state = ClientState::Connecting(f);
                        Poll::Pending
                    }
                }
            }
        }
    }
}