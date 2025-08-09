mod cli;
mod commands;
mod config;
mod error;
mod extension;
mod git;

use clap::Parser;
use cli::Cli;
use error::Result;

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();

    let cli = Cli::parse();
    cli.execute().await
}
