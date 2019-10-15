use rustmann::*;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
struct ConnOpt {
    #[structopt(short, default_value = "127.0.0.1")]
    /// Riemann host
    host: String,
    #[structopt(short, default_value = "5555")]
    /// Riemann port
    port: u16,
}

#[derive(Debug, StructOpt)]
#[structopt(
    name = "riemannc",
    about = "A simple commandline interface for riemann."
)]
enum Opt {
    /// Query riemann for data
    Query {
        /// The riemann query string
        query: String,
        #[structopt(flatten)]
        conn: ConnOpt,
    },

    /// Send event to riemann
    Send {
        #[structopt(long)]
        /// The hostname field of this event
        hostname: Option<String>,
        #[structopt(short)]
        /// The time of this event
        time: Option<i64>,
        /// Service name for the event
        service: String,
        #[structopt(long)]
        /// Attached tags, comma separated
        tags: Option<String>,
        #[structopt(long)]
        /// Event ttl on riemann index
        ttl: Option<f32>,
        #[structopt(long)]
        /// The state field of riemann event
        state: Option<String>,
        /// The metric value
        metric: f64,
        #[structopt(flatten)]
        conn: ConnOpt,
    },
}

fn to_client_options(conn: &ConnOpt) -> RiemannClientOptions {
    RiemannClientOptionsBuilder::default()
        .host(&conn.host)
        .port(conn.port)
        .build()
}

#[tokio::main]
async fn main() -> Result<(), RiemannClientError> {
    let opt = Opt::from_args();

    match opt {
        Opt::Query { query, conn } => {
            let mut client = RiemannClient::new(&to_client_options(&conn));
            let resp = client.send_query(query).await?;
            println!("{:?}", resp);
        }
        Opt::Send {
            service,
            metric,
            hostname,
            time,
            state,
            ttl,
            tags,
            conn,
        } => {
            let mut client = RiemannClient::new(&to_client_options(&conn));

            let mut event_builder = EventBuilder::new();
            event_builder = event_builder.service(service).metric_d(metric);
            if let Some(hostname) = hostname {
                event_builder = event_builder.host(hostname);
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
