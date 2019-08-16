#![feature(async_await)]

use std::error::Error;

use protobuf::Chars;
use rustmann::protos::riemann::Event;
use rustmann::{RiemannClient, RiemannClientOptions};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let mut client = RiemannClient::new(&RiemannClientOptions::default());

    let mut event = Event::new();
    event.set_service(Chars::from("test"));
    event.set_state(Chars::from("1"));

    let response = client.send_events(vec![event]).await?;
    println!("{:?}", response);

    let query_response = client.send_query("state = \"ok\"").await?;
    println!("{:?}", query_response);
    Ok(())
}
