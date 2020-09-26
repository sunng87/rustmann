use metrics_core::{Builder, Drain, Key, Observe, Observer};

use crate::protos::riemann::Event;
use crate::{RiemannClient, RiemannClientError};

pub struct RiemannExporter<C> {
    client: RiemannClient,
    builder: RiemannObserverBuilder,
    controller: C,
}

#[derive(Default)]
pub struct RiemannObserver {
    // TODO: tags, service, host
    events: Vec<Event>,
}

pub struct RiemannObserverBuilder {}

impl Builder for RiemannObserverBuilder {
    type Output = RiemannObserver;

    fn build(&self) -> Self::Output {
        RiemannObserver::default()
    }
}

impl Observer for RiemannObserver {
    fn observe_counter(&mut self, key: Key, value: u64) {}

    fn observe_gauge(&mut self, key: Key, value: i64) {}

    fn observe_histogram(&mut self, key: Key, values: &[u64]) {}
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

            self.client.send_events(events);
        }
    }
}
