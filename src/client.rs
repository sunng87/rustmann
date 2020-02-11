use std::sync::{Arc, Mutex};

use crate::error::RiemannClientError;
use crate::options::RiemannClientOptions;
use crate::protos::riemann::{Event, Query};
use crate::state::{ClientState, Inner};

#[derive(Clone)]
pub struct RiemannClient {
    inner: Inner,
}

impl RiemannClient {
    /// Create `RiemannClient` from options.
    pub fn new(options: &RiemannClientOptions) -> Self {
        RiemannClient {
            inner: Inner {
                state: Arc::new(Mutex::new(ClientState::Disconnected)),
                options: options.clone(),
            },
        }
    }

    /// Send events to riemann via this client.
    pub async fn send_events(&mut self, events: Vec<Event>) -> Result<(), RiemannClientError> {
        let timeout = *self.inner.options.socket_timeout_ms();
        let state = self.inner.state.clone();
        let inner = &mut self.inner;

        let conn_wrapper = inner.await?;
        let mut conn = conn_wrapper.lock().unwrap();

        conn.send_events(events, timeout)
            .await
            .map_err(move |e| {
                *state.lock().unwrap() = ClientState::Disconnected;
                RiemannClientError::from(e)
            })
            .and_then(|msg| {
                if msg.ok.unwrap_or(false) {
                    Ok(())
                } else {
                    Err(RiemannClientError::RiemannError(
                        msg.error.unwrap_or_else(|| "".to_owned()),
                    ))
                }
            })
    }

    /// Query riemann server by riemann query syntax via this client.
    pub async fn send_query<S>(&mut self, query_string: S) -> Result<Vec<Event>, RiemannClientError>
    where
        S: AsRef<str>,
    {
        let timeout = *self.inner.options.socket_timeout_ms();
        let state = self.inner.state.clone();
        let inner = &mut self.inner;

        let conn_wrapper = inner.await?;
        let mut conn = conn_wrapper.lock().unwrap();

        let query = Query {
            string: Some(query_string.as_ref().to_owned()),
        };

        conn.query(query, timeout)
            .await
            .map_err(move |e| {
                *state.lock().unwrap() = ClientState::Disconnected;
                RiemannClientError::from(e)
            })
            .and_then(|msg| {
                if msg.ok.unwrap_or(false) {
                    Ok(msg.events)
                } else {
                    Err(RiemannClientError::RiemannError(
                        msg.error.unwrap_or_else(|| "".to_owned()),
                    ))
                }
            })
    }
}
