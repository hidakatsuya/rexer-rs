use crate::config::Config;
use crate::error::{Result, RexerError};
use crate::extension::{Extension, ExtensionType, LockFile, LockedExtension, Source};
use crate::git::GitManager;
use anyhow::Context;
use chrono::Utc;
use colored::*;
use log::info;
use std::fs;
use std::path::Path;
use std::process::Command;

pub async fn init() -> Result<()> {
    let config = Config::new()?;

    if config.extensions_file_path().exists() {
        println!("{} already exists", config.extensions_file_path().display());
        return Ok(());
    }

    config.create_initial_config()?;
    println!("Created {}", config.extensions_file_path().display());
    Ok(())
}

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

pub async fn state() -> Result<()> {
    let config = Config::new()?;

    match config.load_lock_file()? {
        Some(lock_file) => {
            // Show version similar to Ruby rexer
            println!("Rexer: {}", env!("CARGO_PKG_VERSION"));

            // Group by type like Ruby rexer
            let plugins: Vec<_> = lock_file
                .extensions
                .iter()
                .filter(|ext| matches!(ext.extension_type, ExtensionType::Plugin))
                .collect();
            let themes: Vec<_> = lock_file
                .extensions
                .iter()
                .filter(|ext| matches!(ext.extension_type, ExtensionType::Theme))
                .collect();

            if !plugins.is_empty() {
                println!("\nPlugins:");
                for ext in &plugins {
                    let source_info = format_source_info(&ext.source, &ext.commit_hash);
                    println!(" * {} ({})", ext.name, source_info);
                }
            }

            if !themes.is_empty() {
                println!("\nThemes:");
                for ext in &themes {
                    let source_info = format_source_info(&ext.source, &ext.commit_hash);
                    println!(" * {} ({})", ext.name, source_info);
                }
            }

            if plugins.is_empty() && themes.is_empty() {
                println!("No extensions installed");
            }
        }
        None => {
            println!("No lock file found");
        }
    }

    Ok(())
}

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

fn format_source_info(source: &Source, commit_hash: &Option<String>) -> String {
    let base_info = match source {
        Source::Git {
            url,
            branch,
            tag,
            commit,
        } => {
            if let Some(commit) = commit {
                format!("git: {url} at {commit}")
            } else if let Some(tag) = tag {
                format!("git: {url} at tag {tag}")
            } else if let Some(branch) = branch {
                format!("git: {url} at branch {branch}")
            } else {
                format!("git: {url}")
            }
        }
        Source::GitHub {
            repo,
            branch,
            tag,
            commit,
        } => {
            if let Some(commit) = commit {
                format!("github: {repo} at {commit}")
            } else if let Some(tag) = tag {
                format!("github: {repo} at tag {tag}")
            } else if let Some(branch) = branch {
                format!("github: {repo} at branch {branch}")
            } else {
                format!("github: {repo}")
            }
        }
    };

    // If we have the actual installed commit, show it too
    if let Some(installed_commit) = commit_hash {
        format!("{base_info}, installed: {}", &installed_commit[..8])
    } else {
        base_info
    }
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

async fn uninstall_all_extensions(config: &Config, lock_file: &LockFile) -> Result<()> {
    for extension in &lock_file.extensions {
        println!("Uninstalling {}...", extension.name.blue());
        uninstall_extension(config, extension).await?;
    }
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
        run_plugin_setup(&dest_dir, config).await?;
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

async fn update_extension_and_get_hash(
    config: &Config,
    extension: &LockedExtension,
) -> Result<String> {
    let dest_dir = match extension.extension_type {
        ExtensionType::Plugin => config.plugins_dir().join(&extension.name),
        ExtensionType::Theme => config.themes_dir().join(&extension.name),
    };

    if dest_dir.exists() {
        let commit_hash = GitManager::clone_or_update(&extension.source, &dest_dir)?;

        if matches!(extension.extension_type, ExtensionType::Plugin) {
            run_plugin_setup(&dest_dir, config).await?;
        }

        Ok(commit_hash)
    } else {
        Err(RexerError::ExtensionNotFound(format!(
            "Extension directory not found: {}",
            dest_dir.display()
        )))
    }
}

async fn run_plugin_setup(plugin_dir: &Path, config: &Config) -> Result<()> {
    let gemfile_path = plugin_dir.join("Gemfile");

    if gemfile_path.exists() {
        info!(
            "Running bundle install for plugin at {}",
            plugin_dir.display()
        );
        run_command("bundle", &["install"], Some(plugin_dir), config)?;
    }

    // Check for migrations
    let migrations_dir = plugin_dir.join("db").join("migrate");
    if migrations_dir.exists() && migrations_dir.read_dir()?.next().is_some() {
        info!("Running migrations for plugin at {}", plugin_dir.display());
        let plugin_name = plugin_dir
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown");

        run_command(
            "bundle",
            &[
                "exec",
                "rake",
                "redmine:plugins:migrate",
                &format!("NAME={plugin_name}"),
            ],
            Some(&config.redmine_root),
            config,
        )?;
    }

    Ok(())
}

fn run_command(
    command: &str,
    args: &[&str],
    working_dir: Option<&Path>,
    config: &Config,
) -> Result<()> {
    let mut cmd = Command::new(command);
    cmd.args(args);

    if let Some(dir) = working_dir {
        cmd.current_dir(dir);
    }

    // Apply command prefix if set
    if let Some(prefix) = &config.command_prefix {
        let prefix_parts: Vec<&str> = prefix.split_whitespace().collect();
        if !prefix_parts.is_empty() {
            let mut prefixed_cmd = Command::new(prefix_parts[0]);
            prefixed_cmd.args(&prefix_parts[1..]);
            prefixed_cmd.arg(command);
            prefixed_cmd.args(args);

            if let Some(dir) = working_dir {
                prefixed_cmd.current_dir(dir);
            }

            cmd = prefixed_cmd;
        }
    }

    let status = cmd.status().context("Failed to execute command")?;

    if !status.success() {
        return Err(RexerError::GitError(format!(
            "Command failed: {command} {args:?}"
        )));
    }

    Ok(())
}
