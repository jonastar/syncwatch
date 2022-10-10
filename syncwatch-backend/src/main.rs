use clap::Parser;
use serde::Serialize;

mod config;
mod events;

fn main() {
    let loaded_config = config::Args::parse();

    println!("Starting syncwatch backend")
}
