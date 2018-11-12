extern crate futures;
extern crate protobuf;
extern crate rustmann;
extern crate tokio;
extern crate tokio_codec;
extern crate tokio_fs;

use std::error::Error;

use futures::{Future, Stream};
use tokio_codec::{FramedRead, LinesCodec};
use tokio_fs::stdin;

use protobuf::Chars;
use rustmann::protos::riemann::Event;
use rustmann::Client;

fn main() -> Result<(), Box<Error>> {
    let client = Client::connect(&"127.0.0.1:5555".parse()?);
    let input = FramedRead::new(stdin(), LinesCodec::new());

    let f = client
        .and_then(move |mut c| {
            let readloop = input
                .for_each(move |line| {
                    let mut event = Event::new();
                    event.set_service(Chars::from(line));

                    tokio::spawn(
                        c.send_events(vec![event])
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
