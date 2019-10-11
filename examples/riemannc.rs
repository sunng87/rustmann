
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
    println!("{:?}", opt);
    Ok(())
}
