extern crate futures;
extern crate protobuf;
extern crate rustmann;
extern crate tokio;

use std::error::Error;

use futures::Future;

use protobuf::Chars;
use rustmann::protos::riemann::Event;
use rustmann::Connection;

fn main() -> Result<(), Box<Error>> {
    let client = Connection::connect(&"127.0.0.1:5555".parse()?);
    let f = client
        .and_then(move |mut c| {
            let mut event = Event::new();
            event.set_service(Chars::from("test"));

            c.send_events(vec![event])
        })
        .and_then(|r| {
            println!("{:?}", r);
            Ok(())
        })
        .map_err(|e| eprintln!("{:?}", e));

    tokio::run(f);
    Ok(())
}
