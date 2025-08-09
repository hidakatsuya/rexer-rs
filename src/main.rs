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

    // Initialize git2 library with safer defaults
    unsafe {
        git2::opts::set_verify_owner_validation(false).ok();
    }

    let cli = Cli::parse();
    cli.execute().await
}
