use crate::config::Config;
use crate::error::{Result, RexerError};
use crate::extension::{Extension, ExtensionType, LockedExtension};
use chrono::Utc;
use colored::*;
use std::fs;

pub async fn reinstall(extension_name: String) -> Result<()> {
    let config = Config::new()?;

    let lock_file = config
        .load_lock_file()?
        .ok_or_else(|| RexerError::LockFileError("No lock file found".to_string()))?;

    let extension = lock_file
        .extensions
        .iter()
        .find(|ext| ext.name == extension_name)
        .ok_or_else(|| RexerError::ExtensionNotFound(extension_name.clone()))?;

    uninstall_extension(&config, extension).await?;

    let ext_type = extension.extension_type;
    let commit_hash = install_extension(
        &config,
        &Extension {
            name: extension.name.clone(),
            source: extension.source.clone(),
        },
        ext_type,
    )
    .await?;

    // Update lock file with new commit hash
    let mut updated_lock = lock_file.clone();
    if let Some(locked_ext) = updated_lock
        .extensions
        .iter_mut()
        .find(|e| e.name == extension_name)
    {
        locked_ext.commit_hash = Some(commit_hash);
        locked_ext.installed_at = Utc::now().to_rfc3339();
    }
    config.save_lock_file(&updated_lock)?;

    println!("Reinstalled {}", extension_name.blue());
    Ok(())
}

async fn uninstall_extension(config: &Config, extension: &LockedExtension) -> Result<()> {
    let dest_dir = match extension.extension_type {
        ExtensionType::Plugin => config.plugins_dir().join(&extension.name),
        ExtensionType::Theme => config.themes_dir().join(&extension.name),
    };

    if dest_dir.exists() {
        fs::remove_dir_all(&dest_dir)?;
    }

    Ok(())
}

async fn install_extension(
    config: &Config,
    extension: &Extension,
    ext_type: ExtensionType,
) -> Result<String> {
    let dest_dir = match ext_type {
        ExtensionType::Plugin => config.plugins_dir().join(&extension.name),
        ExtensionType::Theme => config.themes_dir().join(&extension.name),
    };

    // Create parent directories if they don't exist
    if let Some(parent) = dest_dir.parent() {
        fs::create_dir_all(parent)?;
    }

    let commit_hash = crate::git::GitManager::clone_or_update(&extension.source, &dest_dir)?;

    // For plugins, run bundle install and migrations if applicable
    if matches!(ext_type, ExtensionType::Plugin) {
        crate::commands::utils::run_plugin_setup(&dest_dir, config).await?;
    }

    Ok(commit_hash)
}
