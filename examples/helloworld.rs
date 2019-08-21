#![feature(async_await)]

use protobuf::Chars;
use rustmann::protos::riemann::Event;
use rustmann::{RiemannClient, RiemannClientOptions, RiemannClientError};

#[tokio::main]
async fn main() -> Result<(), RiemannClientError> {
    let mut client = RiemannClient::new(&RiemannClientOptions::default());

    let mut event = Event::new();
    event.set_service(Chars::from("riemann_test"));
    event.set_state(Chars::from("ok"));
    event.set_metric_f(123.4);

    let response = client.send_events(vec![event]).await?;
    println!("{:?}", response);

    let query_response = client.send_query("service = \"riemann_test\"").await?;
    println!("{:?}", query_response);
    Ok(())
}
