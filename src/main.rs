use clap::Clap;
use ruok::cli::Opts;
use ruok::config::Config;
use ruok::startup;
use std::fs::File;
use std::io::BufReader;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let opts: Opts = Opts::parse();
    let f = File::open(opts.input)?;
    let f = BufReader::new(f);
    let config = Config::new(f)?;
    startup::startup().await;

    Ok(())
}
