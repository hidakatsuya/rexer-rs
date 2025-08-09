use crate::config::Config;
use crate::error::{Result, RexerError};
use crate::extension::{LockFile, LockedExtension};
use colored::*;
use std::fs;

pub async fn uninstall() -> Result<()> {
    let config = Config::new()?;

    let lock_file = config
        .load_lock_file()?
        .ok_or_else(|| RexerError::LockFileError("No lock file found".to_string()))?;

    uninstall_all_extensions(&config, &lock_file).await?;
    config.delete_lock_file()?;

    println!("Uninstalled all extensions");
    Ok(())
}

async fn uninstall_all_extensions(config: &Config, lock_file: &LockFile) -> Result<()> {
    for extension in &lock_file.extensions {
        println!("Uninstalling {}...", extension.name.blue());
        uninstall_extension(config, extension).await?;
    }
    Ok(())
}

async fn uninstall_extension(config: &Config, extension: &LockedExtension) -> Result<()> {
    let dest_dir = match extension.extension_type {
        crate::extension::ExtensionType::Plugin => config.plugins_dir().join(&extension.name),
        crate::extension::ExtensionType::Theme => config.themes_dir().join(&extension.name),
    };

    if dest_dir.exists() {
        fs::remove_dir_all(&dest_dir)?;
    }

    Ok(())
}