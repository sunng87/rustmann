
use rustmann::*;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = "riemannc", about = "An example riemann cli.")]
enum Opt {

    Query {
        query: String
    },

    Send {
        #[structopt(long)]
        host: Option<String>,
        #[structopt(short)]
        time: Option<i64>,
        service: String,
        #[structopt(long)]
        tags: Option<String>,
        #[structopt(long)]
        ttl: Option<f32>,
        #[structopt(long)]
        state: Option<String>,
        metric: f64,
    }
}

#[tokio::main]
async fn main() -> Result<(), RiemannClientError> {
    let opt = Opt::from_args();

    match opt {
        Opt::Query{query} => {
            let mut client = RiemannClient::new(&RiemannClientOptions::default());
            let resp = client.send_query(query).await?;
            println!("{:?}", resp);
        },
        Opt::Send{service, metric, host, time, state, ttl, tags} => {
            let mut client = RiemannClient::new(&RiemannClientOptions::default());

            let mut event_builder = EventBuilder::new();
            event_builder = event_builder
                .service(service)
                .metric_d(metric);
            if let Some(host) = host {
                event_builder = event_builder.host(host);
            }
            if let Some(time) = time {
                event_builder = event_builder.time(time);
            }
            if let Some(state) = state {
                event_builder = event_builder.state(state);
            }
            if let Some(ttl) = ttl {
                event_builder = event_builder.ttl(ttl);
            }
            if let Some(tags) = tags {
                for t in tags.split(",") {
                    event_builder = event_builder.add_tag(t);
                }
            }

            let resp = client.send_events(vec![event_builder.build()]).await?;
            println!("{:?}", resp);
        }
    }

    Ok(())
}
