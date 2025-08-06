use clap::{Parser, Subcommand};
use crate::commands::*;
use crate::error::Result;

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
    /// Create a new .extensions.rb file
    Init,
    
    /// Install the definitions in .extensions.rb for the specified environment
    Install {
        /// Environment to install (default: "default")
        env: Option<String>,
    },
    
    /// Uninstall extensions for the currently installed environment
    Uninstall,
    
    /// Reinstall specific extension
    Reinstall {
        /// Extension name to reinstall
        extension: String,
    },
    
    /// Switch to different environment
    Switch {
        /// Environment to switch to (default: "default")
        env: Option<String>,
    },
    
    /// Update extensions to latest versions
    Update {
        /// Specific extensions to update (default: all)
        extensions: Vec<String>,
    },
    
    /// Show current state of installed extensions
    State,
    
    /// Show list of environments and their extensions
    Envs,
    
    /// Edit .extensions.rb file
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
        
        let command = self.command.unwrap_or(Commands::Install { env: Some("default".to_string()) });
        
        match command {
            Commands::Init => init().await,
            Commands::Install { env } => install(env.unwrap_or("default".to_string())).await,
            Commands::Uninstall => uninstall().await,
            Commands::Reinstall { extension } => reinstall(extension).await,
            Commands::Switch { env } => switch(env.unwrap_or("default".to_string())).await,
            Commands::Update { extensions } => update(extensions).await,
            Commands::State => state().await,
            Commands::Envs => envs().await,
            Commands::Edit => edit().await,
            Commands::Version => {
                println!("{}", env!("CARGO_PKG_VERSION"));
                Ok(())
            }
        }
    }
}