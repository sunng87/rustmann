#![feature(async_await)]

use std::error::Error;

use protobuf::Chars;
use rustmann::protos::riemann::Event;
use rustmann::{Client, ClientOptions};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let mut client = Client::new(&ClientOptions::default());

    let mut event = Event::new();
    event.set_service(Chars::from("test"));

    let response = client.send_events(vec![event]).await?;
    println!("{:?}", response);
    Ok(())
}
