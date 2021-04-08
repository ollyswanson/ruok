use clap::{AppSettings, Clap};
use std::path::PathBuf;

// TODO: Could be helpful to have a test flag which makes sure that all of the specified services
// and notifications are working.
#[derive(Clap)]
#[clap(version = "0.1", author = "Olly S. <olly.swanson95@gmail.com>")]
#[clap(setting = AppSettings::ColoredHelp)]
pub struct Opts {
    #[clap(parse(from_os_str))]
    pub input: PathBuf,
}
