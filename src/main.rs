mod cli;
mod commands;
mod config;
mod extension;
mod git;
mod error;

use cli::Cli;
use clap::Parser;
use error::Result;

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();
    
    let cli = Cli::parse();
    cli.execute().await
}