#![feature(async_await)]

use std::error::Error;

use futures::future::{FutureExt, TryFutureExt};
use protobuf::Chars;
use rustmann::protos::riemann::Event;
use rustmann::{Client, ClientOptions};

fn main() -> Result<(), Box<dyn Error>> {
    let mut client = Client::new(&ClientOptions::default());

    let mut event = Event::new();
    event.set_service(Chars::from("test"));

    let fut = async move {
        let response = client.send_events(vec![event]).await;

        println!("{:?}", response);
    };
    tokio::run(fut.unit_error().boxed().compat());
    Ok(())
}
