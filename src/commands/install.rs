use crate::config::Config;
use crate::error::Result;
use crate::extension::{Extension, ExtensionType, LockFile, LockedExtension, Source};
use crate::git::GitManager;
use chrono::Utc;
use colored::*;
use std::fs;

pub async fn install() -> Result<()> {
    let config = Config::new()?;
    let extensions_config = config.load_extensions_config()?;

    let current_lock = config.load_lock_file()?;

    // Determine what needs to be done
    if let Some(lock_file) = &current_lock {
        // Update existing installation
        update_installation(&config, &extensions_config, lock_file).await?;
    } else {
        // Fresh install
        install_all_extensions(&config, &extensions_config).await?;
    }

    Ok(())
}

async fn install_all_extensions(
    config: &Config,
    extensions_config: &crate::extension::ExtensionsConfig,
) -> Result<()> {
    let mut locked_extensions = Vec::new();

    for (extension, ext_type) in extensions_config.all_extensions() {
        println!("Installing {}...", extension.name.blue());
        let commit_hash = install_extension(config, extension, ext_type).await?;

        locked_extensions.push(LockedExtension {
            name: extension.name.clone(),
            extension_type: ext_type,
            source: extension.source.clone(),
            commit_hash: Some(commit_hash),
            installed_at: Utc::now().to_rfc3339(),
        });
    }

    let lock_file = LockFile {
        extensions: locked_extensions,
    };

    config.save_lock_file(&lock_file)?;
    println!("Installed {} extensions", lock_file.extensions.len());

    Ok(())
}

async fn update_installation(
    config: &Config,
    extensions_config: &crate::extension::ExtensionsConfig,
    lock_file: &LockFile,
) -> Result<()> {
    let diff = calculate_diff(extensions_config, lock_file);

    // Install new extensions
    let mut new_locked_extensions = Vec::new();
    for (extension, ext_type) in &diff.added {
        println!("Installing {}...", extension.name.blue());
        let commit_hash = install_extension(config, extension, *ext_type).await?;
        new_locked_extensions.push(LockedExtension {
            name: extension.name.clone(),
            extension_type: *ext_type,
            source: extension.source.clone(),
            commit_hash: Some(commit_hash),
            installed_at: Utc::now().to_rfc3339(),
        });
    }

    // Update extensions where source changed
    let mut updated_locked_extensions = Vec::new();
    for (extension, ext_type, old_locked) in &diff.source_changed {
        println!("Updating {} (source changed)...", extension.name.blue());
        // Uninstall old version first
        uninstall_extension(config, old_locked).await?;
        // Install new version
        let commit_hash = install_extension(config, extension, *ext_type).await?;
        updated_locked_extensions.push(LockedExtension {
            name: extension.name.clone(),
            extension_type: *ext_type,
            source: extension.source.clone(),
            commit_hash: Some(commit_hash),
            installed_at: Utc::now().to_rfc3339(),
        });
    }

    // Uninstall removed extensions
    for locked_ext in &diff.removed {
        println!("Uninstalling {}...", locked_ext.name.blue());
        uninstall_extension(config, locked_ext).await?;
    }

    // Build new lock file with updated state
    let mut final_extensions = Vec::new();

    // Add unchanged extensions
    for locked_ext in &lock_file.extensions {
        if !diff.removed.iter().any(|r| r.name == locked_ext.name)
            && !diff
                .source_changed
                .iter()
                .any(|(_, _, old)| old.name == locked_ext.name)
        {
            final_extensions.push(locked_ext.clone());
        }
    }

    // Add new and updated extensions
    final_extensions.extend(new_locked_extensions);
    final_extensions.extend(updated_locked_extensions);

    let updated_lock = LockFile {
        extensions: final_extensions,
    };

    config.save_lock_file(&updated_lock)?;

    if diff.added.is_empty() && diff.removed.is_empty() && diff.source_changed.is_empty() {
        println!("Extensions are up to date");
    } else {
        println!("Installation updated successfully");
    }

    Ok(())
}

#[derive(Debug)]
struct InstallDiff<'a> {
    added: Vec<(&'a Extension, ExtensionType)>,
    removed: Vec<&'a LockedExtension>,
    source_changed: Vec<(&'a Extension, ExtensionType, &'a LockedExtension)>,
}

fn calculate_diff<'a>(
    extensions_config: &'a crate::extension::ExtensionsConfig,
    lock_file: &'a LockFile,
) -> InstallDiff<'a> {
    let mut added = Vec::new();
    let mut removed = Vec::new();
    let mut source_changed = Vec::new();

    // Find extensions to add or update
    for (extension, ext_type) in extensions_config.all_extensions() {
        if let Some(locked_ext) = lock_file
            .extensions
            .iter()
            .find(|le| le.name == extension.name)
        {
            // Extension exists, check if source changed
            if !sources_equal(&extension.source, &locked_ext.source) {
                source_changed.push((extension, ext_type, locked_ext));
            }
        } else {
            // Extension doesn't exist in lock file, add it
            added.push((extension, ext_type));
        }
    }

    // Find extensions to remove
    for locked_ext in &lock_file.extensions {
        let exists_in_config = extensions_config
            .all_extensions()
            .any(|(ext, _)| ext.name == locked_ext.name);
        if !exists_in_config {
            removed.push(locked_ext);
        }
    }

    InstallDiff {
        added,
        removed,
        source_changed,
    }
}

fn sources_equal(source1: &Source, source2: &Source) -> bool {
    // Compare sources to see if they're effectively the same
    match (source1, source2) {
        (
            Source::Git {
                url: url1,
                branch: b1,
                tag: t1,
                commit: c1,
            },
            Source::Git {
                url: url2,
                branch: b2,
                tag: t2,
                commit: c2,
            },
        ) => url1 == url2 && b1 == b2 && t1 == t2 && c1 == c2,
        (
            Source::GitHub {
                repo: r1,
                branch: b1,
                tag: t1,
                commit: c1,
            },
            Source::GitHub {
                repo: r2,
                branch: b2,
                tag: t2,
                commit: c2,
            },
        ) => r1 == r2 && b1 == b2 && t1 == t2 && c1 == c2,
        _ => false, // Different source types are not equal
    }
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

    let commit_hash = GitManager::clone_or_update(&extension.source, &dest_dir)?;

    // For plugins, run bundle install and migrations if applicable
    if matches!(ext_type, ExtensionType::Plugin) {
        crate::commands::utils::run_plugin_setup(&dest_dir, config).await?;
    }

    Ok(commit_hash)
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