use metrics_core::{Builder, Drain, Observe, Observer};

use crate::{RiemannClient, RiemannClientError};

pub struct RiemannExporter<B, C> {
    controller: C,
    builder: B,
    client: RiemannClient,
}

impl<B, C> RiemannExporter<B, C>
where
    C: Observe + Send + Sync + 'static,
    B: Builder + Send + Sync + 'static,
    B::Output: Drain<String> + Observer,
{
    pub fn new(client: RiemannClient) -> RiemannExporter {
        RiemannExporter { client }
    }

    pub async fn start_async(self) -> Result<(), RiemannClientError> {
        let builder = Arc::new(self.builder);
        let controller = Arc::new(self.controller);

        let make_svc = make_service_fn(move |_| {
            let builder = builder.clone();
            let controller = controller.clone();

            async move {
                Ok::<_, Error>(service_fn(move |_| {
                    let builder = builder.clone();
                    let controller = controller.clone();

                    async move {
                        let mut observer = builder.build();
                        controller.observe(&mut observer);
                        let output = observer.drain();
                        Ok::<_, Error>(Response::new(Body::from(output)))
                    }
                }))
            }
        });
    }
}
