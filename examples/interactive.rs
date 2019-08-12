#![feature(async_await)]

use std::error::Error;

use tokio::codec::{FramedRead, LinesCodec};
use tokio::io::stdin;
use tokio::prelude::*;

use protobuf::Chars;
use rustmann::protos::riemann::Event;
use rustmann::{Client, ClientOptions};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let mut client = Client::new(&ClientOptions::default());
    let mut input = FramedRead::new(stdin(), LinesCodec::new());

    while let Some(Ok(line)) = input.next().await {
        let mut event = Event::new();
        event.set_host(Chars::from("thinkless"));
        event.set_service(Chars::from("rustmann_interactive"));
        event.set_description(line.into());

        let response = client.send_events(vec![event]).await?;
        println!("{:?}", response);
    }
    Ok(())
}
