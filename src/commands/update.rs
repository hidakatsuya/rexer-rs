use crate::config::Config;
use crate::error::{Result, RexerError};
use crate::extension::LockedExtension;
use crate::git::GitManager;
use chrono::Utc;
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

    if extensions_to_update.is_empty() {
        println!("No extensions to update");
        return Ok(());
    }

    // Track updates for lock file
    let mut updated_lock = lock_file.clone();
    let mut any_updated = false;

    for ext in &extensions_to_update {
        println!("Updating {}...", ext.name.blue());
        let new_commit_hash = update_extension_and_get_hash(&config, ext).await?;

        // Update the lock file entry if commit hash changed
        if let Some(locked_ext) = updated_lock
            .extensions
            .iter_mut()
            .find(|e| e.name == ext.name)
        {
            if locked_ext.commit_hash.as_ref() != Some(&new_commit_hash) {
                locked_ext.commit_hash = Some(new_commit_hash);
                locked_ext.installed_at = Utc::now().to_rfc3339();
                any_updated = true;
            }
        }
    }

    // Save updated lock file if any changes were made
    if any_updated {
        config.save_lock_file(&updated_lock)?;
        println!("Updated {} extension(s)", extensions_to_update.len());
    } else {
        println!("All extensions are already up to date");
    }

    Ok(())
}

async fn update_extension_and_get_hash(
    config: &Config,
    extension: &LockedExtension,
) -> Result<String> {
    let dest_dir = match extension.extension_type {
        crate::extension::ExtensionType::Plugin => config.plugins_dir().join(&extension.name),
        crate::extension::ExtensionType::Theme => config.themes_dir().join(&extension.name),
    };

    if dest_dir.exists() {
        let commit_hash = GitManager::clone_or_update(&extension.source, &dest_dir)?;

        if matches!(
            extension.extension_type,
            crate::extension::ExtensionType::Plugin
        ) {
            crate::commands::utils::run_plugin_setup(&dest_dir, config).await?;
        }

        Ok(commit_hash)
    } else {
        Err(RexerError::ExtensionNotFound(format!(
            "Extension directory not found: {}",
            dest_dir.display()
        )))
    }
}
