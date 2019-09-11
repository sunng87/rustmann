use protobuf::Chars;
use rustmann::protos::riemann::Event;
use rustmann::{RiemannClient, RiemannClientError, RiemannClientOptionsBuilder};

#[tokio::main]
async fn main() -> Result<(), RiemannClientError> {
    let options = RiemannClientOptionsBuilder::default()
        .host("localhost")
        .port(5555 as u16)
        .use_udp(true)
        .build();

    let mut client = RiemannClient::new(&options);

    let mut event = Event::new();
    event.set_service(Chars::from("riemann_test"));
    event.set_state(Chars::from("ok"));
    event.set_metric_f(123.4);

    client.send_events(vec![event]).await?;
    println!("done");

    Ok(())
}
