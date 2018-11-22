extern crate futures;
extern crate protobuf;
extern crate rustmann;
extern crate tokio;

use std::error::Error;

use futures::{Future, Stream};
use tokio::codec::{FramedRead, LinesCodec};
use tokio::io::stdin;

use protobuf::Chars;
use rustmann::protos::riemann::Event;
use rustmann::Connection;

fn main() -> Result<(), Box<Error>> {
    let client = Connection::connect(&"127.0.0.1:5555".parse()?, 5000);
    let input = FramedRead::new(stdin(), LinesCodec::new());

    let f = client
        .and_then(move |mut c| {
            let readloop = input
                .for_each(move |line| {
                    let mut event = Event::new();
                    event.set_host(Chars::from("thinkless"));
                    event.set_service(Chars::from("rustmann_interactive"));
                    event.set_description(line.into());

                    tokio::spawn(
                        c.send_events(vec![event], 5000)
                            .and_then(|r| {
                                println!("{:?}", r);
                                Ok(())
                            })
                            .map_err(|e| eprintln!("{:?}", e)),
                    );
                    Ok(())
                })
                .map_err(|e| eprintln!("{:?}", e));
            tokio::spawn(readloop);
            Ok(())
        })
        .map_err(|e| eprintln!("{:?}", e));

    tokio::run(f);
    Ok(())
}
