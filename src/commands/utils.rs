use crate::config::Config;
use crate::error::{Result, RexerError};
use anyhow::Context;
use log::info;
use std::path::Path;
use std::process::Command;

pub async fn run_plugin_setup(plugin_dir: &Path, config: &Config) -> Result<()> {
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

pub fn run_command(
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
