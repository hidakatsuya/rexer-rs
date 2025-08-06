use crate::config::Config;
use crate::error::{Result, RexerError};
use crate::extension::{Extension, ExtensionType, LockFile, LockedExtension};
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

pub async fn install(env_name: String) -> Result<()> {
    let config = Config::new()?;
    let extensions_config = config.load_extensions_config()?;

    let extensions = extensions_config
        .get_environment(&env_name)
        .ok_or_else(|| RexerError::EnvironmentNotFound(env_name.clone()))?;

    let current_lock = config.load_lock_file()?;

    // Determine what needs to be done
    if let Some(lock_file) = &current_lock {
        if lock_file.environment == env_name {
            // Update existing environment
            update_environment(&config, extensions, lock_file).await?;
        } else {
            // Switch environment
            uninstall_environment(&config, lock_file).await?;
            install_environment(&config, &env_name, extensions).await?;
        }
    } else {
        // Fresh install
        install_environment(&config, &env_name, extensions).await?;
    }

    Ok(())
}

pub async fn uninstall() -> Result<()> {
    let config = Config::new()?;

    let lock_file = config
        .load_lock_file()?
        .ok_or_else(|| RexerError::LockFileError("No lock file found".to_string()))?;

    uninstall_environment(&config, &lock_file).await?;
    config.delete_lock_file()?;

    println!("Uninstalled all extensions");
    Ok(())
}

pub async fn state() -> Result<()> {
    let config = Config::new()?;

    match config.load_lock_file()? {
        Some(lock_file) => {
            println!("Environment: {}", lock_file.environment.green());
            println!("Extensions:");
            for ext in &lock_file.extensions {
                let type_str = match ext.extension_type {
                    ExtensionType::Plugin => "plugin",
                    ExtensionType::Theme => "theme",
                };
                println!(
                    "  {} {} ({})",
                    type_str,
                    ext.name.blue(),
                    ext.source.full_url()
                );
                if let Some(commit) = &ext.commit_hash {
                    println!("    Commit: {}", commit.yellow());
                }
            }
        }
        None => {
            println!("No extensions installed");
        }
    }

    Ok(())
}

pub async fn envs() -> Result<()> {
    let config = Config::new()?;
    let extensions_config = config.load_extensions_config()?;

    for (env_name, extensions) in &extensions_config.environments {
        println!("{}:", env_name.green());
        for ext in extensions {
            let type_str = match ext.extension_type {
                ExtensionType::Plugin => "plugin",
                ExtensionType::Theme => "theme",
            };
            println!(
                "  {} {} ({})",
                type_str,
                ext.name.blue(),
                ext.source.full_url()
            );
        }
        println!();
    }

    Ok(())
}

pub async fn switch(env_name: String) -> Result<()> {
    install(env_name).await
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

    for ext in &extensions_to_update {
        println!("Updating {}...", ext.name.blue());
        update_extension(&config, ext).await?;
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
    install_extension(
        &config,
        &Extension {
            name: extension.name.clone(),
            extension_type: extension.extension_type.clone(),
            source: extension.source.clone(),
            hooks: None,
        },
    )
    .await?;

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

async fn install_environment(
    config: &Config,
    env_name: &str,
    extensions: &[Extension],
) -> Result<()> {
    let mut locked_extensions = Vec::new();

    for extension in extensions {
        println!("Installing {}...", extension.name.blue());
        let commit_hash = install_extension(config, extension).await?;

        locked_extensions.push(LockedExtension {
            name: extension.name.clone(),
            extension_type: extension.extension_type.clone(),
            source: extension.source.clone(),
            commit_hash: Some(commit_hash),
            installed_at: Utc::now().to_rfc3339(),
        });
    }

    let lock_file = LockFile {
        environment: env_name.to_string(),
        extensions: locked_extensions,
    };

    config.save_lock_file(&lock_file)?;
    println!(
        "Installed {} extensions for environment '{}'",
        extensions.len(),
        env_name.green()
    );

    Ok(())
}

async fn uninstall_environment(config: &Config, lock_file: &LockFile) -> Result<()> {
    for extension in &lock_file.extensions {
        println!("Uninstalling {}...", extension.name.blue());
        uninstall_extension(config, extension).await?;
    }
    Ok(())
}

async fn update_environment(
    _config: &Config,
    _extensions: &[Extension],
    _lock_file: &LockFile,
) -> Result<()> {
    // This is a simplified version - in a full implementation, you'd compare
    // the current state with the desired state and only make necessary changes
    println!("Environment is already installed. Use 'rex update' to update extensions.");
    Ok(())
}

async fn install_extension(config: &Config, extension: &Extension) -> Result<String> {
    let dest_dir = match extension.extension_type {
        ExtensionType::Plugin => config.plugins_dir().join(&extension.name),
        ExtensionType::Theme => config.themes_dir().join(&extension.name),
    };

    // Create parent directories if they don't exist
    if let Some(parent) = dest_dir.parent() {
        fs::create_dir_all(parent)?;
    }

    let commit_hash = GitManager::clone_or_update(&extension.source, &dest_dir)?;

    // For plugins, run bundle install and migrations if applicable
    if matches!(extension.extension_type, ExtensionType::Plugin) {
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

async fn update_extension(config: &Config, extension: &LockedExtension) -> Result<()> {
    let dest_dir = match extension.extension_type {
        ExtensionType::Plugin => config.plugins_dir().join(&extension.name),
        ExtensionType::Theme => config.themes_dir().join(&extension.name),
    };

    if dest_dir.exists() {
        GitManager::clone_or_update(&extension.source, &dest_dir)?;

        if matches!(extension.extension_type, ExtensionType::Plugin) {
            run_plugin_setup(&dest_dir, config).await?;
        }
    }

    Ok(())
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
        return Err(RexerError::GitError(format!("Command failed: {command} {args:?}")));
    }

    Ok(())
}
