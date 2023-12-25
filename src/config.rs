// config.rs
// CLI options and logging setup
use serde_derive::Deserialize;
use structopt::StructOpt;
use clap::Parser;

/// Restaurant API
#[derive(Parser, Debug, Deserialize, StructOpt)]
#[structopt(name = "restaurant-api")]
pub struct Opt {
    /// Server address
    #[structopt(short, long, default_value = "127.0.0.1")]
    pub address: String,
    /// Server port 0-65535
    #[structopt(short, long, default_value = "3000")]
    pub port: u16,
    #[structopt(short, long, default_value = "10")]
    pub num_clients: u16
}
