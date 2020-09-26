use metrics_core::{Builder, Drain, Key, Observe, Observer};

use crate::protos::riemann::Event;
use crate::{EventBuilder, RiemannClient, RiemannClientError};

pub struct RiemannExporter<C> {
    client: RiemannClient,
    builder: RiemannObserverBuilder,
    controller: C,
}

#[derive(Default)]
pub struct RiemannObserver {
    host: String,
    tags: Vec<String>,
    events: Vec<Event>,
}

// TODO: tags, service, host
pub struct RiemannObserverBuilder {}

impl Builder for RiemannObserverBuilder {
    type Output = RiemannObserver;

    fn build(&self) -> Self::Output {
        RiemannObserver::default()
    }
}

impl Observer for RiemannObserver {
    fn observe_counter(&mut self, key: Key, value: u64) {
        let event = self.create_event(key.name().into_owned(), value as f64);
        self.events.push(event);
    }

    fn observe_gauge(&mut self, key: Key, value: i64) {
        let event = self.create_event(key.name().into_owned(), value as f64);
        self.events.push(event);
    }

    fn observe_histogram(&mut self, key: Key, values: &[u64]) {
        // TODO: p99, max, mean
    }
}

impl RiemannObserver {
    fn create_event(&self, service: String, value: f64) -> Event {
        let mut eb = EventBuilder::new()
            .service(service)
            .metric_d(value)
            .host(&self.host);

        for s in &self.tags {
            eb = eb.add_tag(s);
        }

        eb.build()
    }
}

impl Drain<Vec<Event>> for RiemannObserver {
    fn drain(&mut self) -> Vec<Event> {
        let events = self.events.clone();
        self.events = vec![];

        events
    }
}

impl<C> RiemannExporter<C>
where
    C: Observe + Send + Sync + 'static,
{
    pub fn new(
        client: RiemannClient,
        builder: RiemannObserverBuilder,
        controller: C,
    ) -> RiemannExporter<C> {
        RiemannExporter {
            client,
            builder,
            controller,
        }
    }

    pub async fn start_async(mut self) -> Result<(), RiemannClientError> {
        let mut observer = self.builder.build();
        loop {
            self.controller.observe(&mut observer);
            let events = observer.drain();

            let _ = self.client.send_events(events).await;
        }
    }
}
