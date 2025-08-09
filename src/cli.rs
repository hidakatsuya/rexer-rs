use crate::commands::*;
use crate::error::Result;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "rex")]
#[command(about = "Redmine Extension (Plugin and Theme) manager")]
#[command(version)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,

    /// Detailed output
    #[arg(short, long, global = true)]
    pub verbose: bool,

    /// Minimal output  
    #[arg(short, long, global = true)]
    pub quiet: bool,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Create a new .extensions.yml file
    Init,

    /// Install extensions defined in .extensions.yml
    Install,

    /// Uninstall all currently installed extensions
    Uninstall,

    /// Reinstall specific extension
    Reinstall {
        /// Extension name to reinstall
        extension: String,
    },

    /// Update extensions to latest versions from lock file sources
    Update {
        /// Specific extensions to update (default: all)
        extensions: Vec<String>,
    },

    /// Show current state of installed extensions
    State,

    /// Edit .extensions.yml file
    Edit,

    /// Show version information
    Version,
}

impl Cli {
    pub async fn execute(self) -> Result<()> {
        // Set up logging level based on verbosity flags
        if self.verbose {
            std::env::set_var("RUST_LOG", "debug");
        } else if self.quiet {
            std::env::set_var("RUST_LOG", "error");
        } else {
            std::env::set_var("RUST_LOG", "info");
        }

        let command = self.command.unwrap_or(Commands::Install);

        match command {
            Commands::Init => init().await,
            Commands::Install => install().await,
            Commands::Uninstall => uninstall().await,
            Commands::Reinstall { extension } => reinstall(extension).await,
            Commands::Update { extensions } => update(extensions).await,
            Commands::State => state().await,
            Commands::Edit => edit().await,
            Commands::Version => {
                println!("rex {}", env!("CARGO_PKG_VERSION"));
                Ok(())
            }
        }
    }
}
