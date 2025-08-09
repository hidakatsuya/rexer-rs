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

    // Initialize git2 library with comprehensive safety settings
    initialize_git2().unwrap_or_else(|e| {
        eprintln!("Warning: Failed to initialize git2: {e}");
        eprintln!("Falling back to command-line git for all operations");
    });

    let cli = Cli::parse();
    cli.execute().await
}

fn initialize_git2() -> std::result::Result<(), Box<dyn std::error::Error>> {
    unsafe {
        // Disable owner validation that can cause issues
        git2::opts::set_verify_owner_validation(false)?;

        // Configure search paths for git config
        if let Ok(home) = std::env::var("HOME") {
            git2::opts::set_search_path(git2::ConfigLevel::Global, &home)?;
        }
    }
    Ok(())
}
