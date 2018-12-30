use std::error::Error;

use futures::Future;

use protobuf::Chars;
use rustmann::protos::riemann::Event;
use rustmann::{Client, ClientOptions};

fn main() -> Result<(), Box<Error>> {
    let client = Client::new(&ClientOptions::default());

    let mut event = Event::new();
    event.set_service(Chars::from("test"));

    let f = client
        .send_events(vec![event])
        .and_then(|r| {
            println!("{:?}", r);
            Ok(())
        })
        .map_err(|e| eprintln!("{:?}", e));

    tokio::run(f);
    Ok(())
}
