use futures_util::stream::StreamExt;

use tokio::io::stdin;
use tokio::sync::mpsc;
use tokio_util::codec::{FramedRead, LinesCodec};

use rustmann::{EventBuilder, RiemannClient, RiemannClientError, RiemannClientOptions};

#[tokio::main]
async fn main() -> Result<(), RiemannClientError> {
    let mut client = RiemannClient::new(&RiemannClientOptions::default());
    let mut input = FramedRead::new(stdin(), LinesCodec::new());

    let (tx, mut rx) = mpsc::unbounded_channel();

    tokio::spawn(async move {
        while let Some(line) = input.next().await {
            if let Err(_) = tx.send(line) {
                break;
            };
        }
    });

    while let Some(Ok(line)) = rx.next().await {
        let event = EventBuilder::new()
            .host("thinkless")
            .service("rustmann_interactive")
            .description(line)
            .build();

        let response = client.send_events(vec![event]).await;
        println!("{:?}", response);
    }
    Ok(())
}
