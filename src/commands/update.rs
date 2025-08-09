use crate::config::Config;
use crate::error::{Result, RexerError};
use crate::extension::LockedExtension;
use crate::git::GitManager;
use colored::*;

pub async fn update(extension_names: Vec<String>) -> Result<()> {
    let config = Config::new()?;

    let lock_file = config
        .load_lock_file()?
        .ok_or_else(|| RexerError::LockFileError("No lock file found".to_string()))?;

    let extensions_to_update = if extension_names.is_empty() {
        lock_file.extensions.clone()
    } else {
        lock_file
            .extensions
            .iter()
            .filter(|ext| extension_names.contains(&ext.name))
            .cloned()
            .collect()
    };

    for ext in &extensions_to_update {
        println!("Updating {}...", ext.name.blue());
        update_extension(&config, ext).await?;
    }

    Ok(())
}

async fn update_extension(config: &Config, extension: &LockedExtension) -> Result<()> {
    let dest_dir = match extension.extension_type {
        crate::extension::ExtensionType::Plugin => config.plugins_dir().join(&extension.name),
        crate::extension::ExtensionType::Theme => config.themes_dir().join(&extension.name),
    };

    if dest_dir.exists() {
        GitManager::clone_or_update(&extension.source, &dest_dir)?;

        if matches!(
            extension.extension_type,
            crate::extension::ExtensionType::Plugin
        ) {
            crate::commands::utils::run_plugin_setup(&dest_dir, config).await?;
        }
    }

    Ok(())
}
