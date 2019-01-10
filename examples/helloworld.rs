#![feature(await_macro, async_await, futures_api)]

#[macro_use]
extern crate tokio;

use std::error::Error;

use protobuf::Chars;
use rustmann::protos::riemann::Event;
use rustmann::{Client, ClientOptions};

fn main() -> Result<(), Box<Error>> {
    let mut client = Client::new(&ClientOptions::default());

    let mut event = Event::new();
    event.set_service(Chars::from("test"));

    tokio::run_async(async move {
        let response = await!(client.send_events(vec![event]));

        println!("{:?}", response);
    });
    Ok(())
}
