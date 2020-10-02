use derive_builder::Builder as DeriveBuilder;
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

#[derive(DeriveBuilder)]
#[builder(setter(into))]
#[builder(build_fn(skip))]
pub struct RiemannObserverBuilder {
    host: String,
    tags: Vec<String>,
}

impl Builder for RiemannObserverBuilder {
    type Output = RiemannObserver;

    fn build(&self) -> Self::Output {
        RiemannObserver {
            host: self.host.clone(),
            tags: self.tags.clone(),
            events: Vec::new(),
        }
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
        let size = values.len();

        if size == 0 {
            return;
        }

        let mut value_vec = Vec::from(values);
        value_vec.sort();

        // min
        self.events
            .push(self.create_event(format!("{}.min", key.name()), values[0] as f64));
        // max
        self.events
            .push(self.create_event(format!("{}.max", key.name()), values[size - 1] as f64));
        // mean
        self.events.push(self.create_event(
            format!("{}.mean", key.name()),
            value_vec.iter().sum::<u64>() as f64 / size as f64,
        ));
        // p50
        self.events.push(self.create_event(
            format!("{}.p50", key.name()),
            values[(size as f64 * 0.5) as usize] as f64,
        ));
        // p90
        self.events.push(self.create_event(
            format!("{}.p90", key.name()),
            values[(size as f64 * 0.9) as usize] as f64,
        ));
        // p99
        self.events.push(self.create_event(
            format!("{}.p99", key.name()),
            values[(size as f64 * 0.99) as usize] as f64,
        ));
        // p999
        self.events.push(self.create_event(
            format!("{}.p999", key.name()),
            values[(size as f64 * 0.999) as usize] as f64,
        ));
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
