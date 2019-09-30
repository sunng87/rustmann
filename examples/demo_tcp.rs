use rustmann::{EventBuilder, RiemannClient, RiemannClientError, RiemannClientOptions};

#[tokio::main]
async fn main() -> Result<(), RiemannClientError> {
    let mut client = RiemannClient::new(&RiemannClientOptions::default());

    let event = EventBuilder::new()
        .service("riemann_test")
        .state("ok")
        .metric_f(123.4)
        .build();

    let response = client.send_events(vec![event]).await?;
    println!("{:?}", response);

    let query_response = client.send_query("service = \"riemann_test\"").await?;
    println!("{:?}", query_response);
    Ok(())
}
