use crate::config::Config;
use crate::error::Result;
use anyhow::Context;
use std::process::Command;

pub async fn edit() -> Result<()> {
    let config = Config::new()?;
    let path = config.extensions_file_path();

    let editor = std::env::var("EDITOR").unwrap_or_else(|_| "vim".to_string());

    Command::new(&editor)
        .arg(&path)
        .status()
        .context("Failed to open editor")?;

    Ok(())
}
