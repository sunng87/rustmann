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

    let response = client.send_events(vec![event]).await?;
    println!("{:?}", response);
    Ok(())
}
