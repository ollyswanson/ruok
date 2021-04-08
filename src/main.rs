use clap::{AppSettings, Clap};
use reqwest::Client;
use ruok::config::Config;
use ruok::startup;
use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;

// TODO: Could be helpful to have a test flag which makes sure that all of the specified services
// and notifications are working.
#[derive(Clap)]
#[clap(version = "0.1", author = "Olly S. <olly.swanson95@gmail.com>")]
#[clap(setting = AppSettings::ColoredHelp)]
pub struct Opts {
    /// Path to yaml config file.
    #[clap(parse(from_os_str))]
    pub input: PathBuf,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let opts: Opts = Opts::parse();
    let f = BufReader::new(File::open(opts.input)?);
    let config = Config::new(f)?;
    let client = Client::new();

    startup::startup(client, config).await;
    Ok(())
}
