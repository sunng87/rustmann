use metrics_core::{Builder, Drain, Observe, Observer};

use crate::{Event, RiemannClient, RiemannClientError};

pub struct RiemannExporter<B, C> {
    controller: C,
    client: RiemannClient,
}

pub struct RiemannObserver {
    events: Vec<Event>,
}

pub struct RiemannObserverBuilder {}

impl Builder for RiemannObserverBuilder {
    type Output = RiemannObserver;

    fn build(&self) -> Self::Output {
        RiemannObserver
    }
}

impl Observer for RiemannObserver {}

impl Drain for RiemannObserver {}

impl<B, C> RiemannExporter<B, C>
where
    C: Observe + Send + Sync + 'static,
    B: Builder + Send + Sync + 'static,
    B::Output: Drain<String> + Observer,
{
    pub fn new(client: RiemannClient, builder: B, controller: C) -> RiemannExporter {
        RiemannExporter {
            client,
            builder,
            controller,
        }
    }

    pub async fn start_async(self) -> Result<(), RiemannClientError> {
        let observer = self.builder.build();
        loop {
            self.controller.observe(&mut observer);
            let output = observer.drain();

            let event = into_event(output);

            self.client.send_events(vec![event]);
        }
    }
}
