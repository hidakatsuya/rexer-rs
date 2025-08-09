use crate::config::Config;
use crate::error::Result;
use crate::extension::{ExtensionType, Source};

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