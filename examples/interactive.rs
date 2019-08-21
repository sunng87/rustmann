#![feature(async_await)]

use tokio::codec::{FramedRead, LinesCodec};
use tokio::io::stdin;
use tokio::prelude::*;
use tokio::sync::mpsc;

use protobuf::Chars;
use rustmann::protos::riemann::Event;
use rustmann::{RiemannClient, RiemannClientOptions, RiemannClientError};

#[tokio::main]
async fn main() -> Result<(), RiemannClientError> {
    let mut client = RiemannClient::new(&RiemannClientOptions::default());
    let mut input = FramedRead::new(stdin(), LinesCodec::new());

    let (mut tx, mut rx) = mpsc::unbounded_channel();

    tokio::spawn(async move {
        tx.send_all(&mut input).await.unwrap();
    });

    while let Some(Ok(line)) = rx.next().await {
        let mut event = Event::new();
        event.set_host(Chars::from("thinkless"));
        event.set_service(Chars::from("rustmann_interactive"));
        event.set_description(line.into());

        let response = client.send_events(vec![event]).await?;
        println!("{:?}", response);
    }
    Ok(())
}
