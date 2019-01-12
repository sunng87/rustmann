#![feature(await_macro, async_await, futures_api)]

#[macro_use]
extern crate tokio;

use std::error::Error;

use tokio::codec::{FramedRead, LinesCodec};
use tokio::io::stdin;
use tokio::prelude::*;

use protobuf::Chars;
use rustmann::protos::riemann::Event;
use rustmann::{Client, ClientOptions};

fn main() -> Result<(), Box<Error>> {
    let mut client = Client::new(&ClientOptions::default());
    let mut input = FramedRead::new(stdin(), LinesCodec::new());

    tokio::run_async(
        async move {
            while let Some(Ok(line)) = await!(input.next()) {
                let mut event = Event::new();
                event.set_host(Chars::from("thinkless"));
                event.set_service(Chars::from("rustmann_interactive"));
                event.set_description(line.into());

                let response = await!(client.send_events(vec![event]));
                println!("{:?}", response);
            }
        },
    );
    Ok(())
}
