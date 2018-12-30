use std::error::Error;

use futures::{Future, Stream};
use tokio::codec::{FramedRead, LinesCodec};
use tokio::io::stdin;

use protobuf::Chars;
use rustmann::protos::riemann::Event;
use rustmann::{Client, ClientOptions};

fn main() -> Result<(), Box<Error>> {
    let client = Client::new(&ClientOptions::default());
    let input = FramedRead::new(stdin(), LinesCodec::new());

    let readloop = input
        .for_each(move |line| {
            let mut event = Event::new();
            event.set_host(Chars::from("thinkless"));
            event.set_service(Chars::from("rustmann_interactive"));
            event.set_description(line.into());

            tokio::spawn(
                client
                    .send_events(vec![event])
                    .and_then(|r| {
                        println!("{:?}", r);
                        Ok(())
                    })
                    .map_err(|e| eprintln!("{:?}", e)),
            );
            Ok(())
        })
        .map_err(|e| eprintln!("{:?}", e));

    tokio::run(readloop);
    Ok(())
}
