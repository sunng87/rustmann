use rustmann::{EventBuilder, RiemannClient, RiemannClientError, RiemannClientOptionsBuilder};

#[tokio::main]
async fn main() -> Result<(), RiemannClientError> {
    let options = RiemannClientOptionsBuilder::default()
        .host("localhost")
        .port(5555 as u16)
        .use_udp(true)
        .build();

    let mut client = RiemannClient::new(&options);

    let event = EventBuilder::new()
        .service("riemann_test")
        .state("ok")
        .metric_f(123.4)
        .build();

    client.send_events(vec![event]).await?;
    println!("done");

    Ok(())
}
