use std::ops::DerefMut;

use futures::lock::Mutex;

use crate::error::RiemannClientError;
use crate::options::RiemannClientOptions;
use crate::protos::riemann::{Event, Query};
use crate::state::{ClientState, Inner};

pub struct RiemannClient {
    inner: Mutex<Inner>,
    options: RiemannClientOptions,
}

impl RiemannClient {
    /// Create `RiemannClient` from options.
    pub fn new(options: &RiemannClientOptions) -> Self {
        RiemannClient {
            inner: Mutex::new(Inner {
                state: ClientState::Disconnected,
                options: options.clone(),
            }),
            options: options.clone(),
        }
    }

    /// Send events to riemann via this client.
    pub async fn send_events(&self, events: Vec<Event>) -> Result<(), RiemannClientError> {
        let timeout = *self.options.socket_timeout_ms();

        let conn = {
            let mut inner = self.inner.lock().await;
            let i = inner.deref_mut();
            i.await?
        };

        match conn.send_events(events, timeout).await {
            Ok(msg) => {
                if msg.ok.unwrap_or(false) {
                    Ok(())
                } else {
                    Err(RiemannClientError::RiemannError(
                        msg.error.unwrap_or_else(|| "".to_owned()),
                    ))
                }
            }
            Err(e) => {
                let mut inner = self.inner.lock().await;
                let i = inner.deref_mut();
                i.state = ClientState::Disconnected;

                Err(RiemannClientError::from(e))
            }
        }
    }

    /// Query riemann server by riemann query syntax via this client.
    pub async fn send_query<S>(&self, query_string: S) -> Result<Vec<Event>, RiemannClientError>
    where
        S: AsRef<str>,
    {
        let timeout = *self.options.socket_timeout_ms();

        let conn = {
            let mut inner = self.inner.lock().await;
            let i = inner.deref_mut();
            i.await?
        };

        let query = Query {
            string: Some(query_string.as_ref().to_owned()),
        };

        match conn.query(query, timeout).await {
            Ok(msg) => {
                if msg.ok.unwrap_or(false) {
                    Ok(msg.events)
                } else {
                    Err(RiemannClientError::RiemannError(
                        msg.error.unwrap_or_else(|| "".to_owned()),
                    ))
                }
            }
            Err(e) => {
                let mut inner = self.inner.lock().await;
                let i = inner.deref_mut();
                i.state = ClientState::Disconnected;

                Err(RiemannClientError::from(e))
            }
        }
    }
}
