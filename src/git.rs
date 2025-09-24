use crate::error::{Result, RexerError};
use crate::extension::Source;
use log::{debug, info};
use std::path::Path;
use std::process::Command;

pub struct GitManager;

impl GitManager {
    /// Run a git command in the specified directory and return the output
    fn run_git_command(args: &[&str], working_dir: Option<&Path>) -> Result<String> {
        let mut cmd = Command::new("git");
        cmd.args(args);

        if let Some(dir) = working_dir {
            cmd.current_dir(dir);
        }

        let output = cmd
            .output()
            .map_err(|e| RexerError::GitError(format!("Failed to execute git command: {}", e)))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let stdout = String::from_utf8_lossy(&output.stdout);
            return Err(RexerError::GitError(format!(
                "Git command failed: git {}\nError: {}\nOutput: {}",
                args.join(" "),
                stderr.trim(),
                stdout.trim()
            )));
        }

        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    }

    /// Run a git command without capturing output, just checking status
    fn run_git_command_status(args: &[&str], working_dir: Option<&Path>) -> Result<()> {
        let mut cmd = Command::new("git");
        cmd.args(args);

        if let Some(dir) = working_dir {
            cmd.current_dir(dir);
        }

        let output = cmd
            .output()
            .map_err(|e| RexerError::GitError(format!("Failed to execute git command: {}", e)))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(RexerError::GitError(format!(
                "Git command failed: git {}\nError: {}",
                args.join(" "),
                stderr.trim()
            )));
        }

        Ok(())
    }
    pub fn clone_or_update(source: &Source, destination: &Path) -> Result<String> {
        if destination.exists() {
            Self::update_repository(source, destination)
        } else {
            Self::clone_repository(source, destination)
        }
    }

    fn clone_repository(source: &Source, destination: &Path) -> Result<String> {
        let url = source.full_url();
        info!("Cloning {} to {}", url, destination.display());

        // Clone the repository
        Self::run_git_command_status(&["clone", &url, &destination.to_string_lossy()], None)?;

        // Checkout specific reference if provided
        if let Some(reference) = source.reference() {
            Self::checkout_reference(destination, &reference)?;
        }

        Self::get_current_commit_hash(destination)
    }

    fn update_repository(source: &Source, destination: &Path) -> Result<String> {
        let url = source.full_url();
        info!("Updating {} at {}", url, destination.display());

        // Fetch latest changes from origin
        Self::run_git_command_status(&["fetch", "origin"], Some(destination))?;

        if let Some(reference) = source.reference() {
            Self::checkout_reference(destination, &reference)?;
        } else {
            // Checkout default branch (pull latest changes)
            Self::checkout_default_branch(destination)?;
        }

        Self::get_current_commit_hash(destination)
    }

    fn checkout_reference(repo_path: &Path, reference: &str) -> Result<()> {
        debug!("Checking out reference: {reference}");

        // First try to checkout the reference directly (works for branches, tags, commits)
        if Self::run_git_command_status(&["checkout", reference], Some(repo_path)).is_ok() {
            return Ok(());
        }

        // Try remote branch - create local tracking branch if it doesn't exist
        let remote_branch = format!("origin/{reference}");
        if Self::run_git_command_status(
            &["checkout", "-b", reference, &remote_branch],
            Some(repo_path),
        )
        .is_ok()
        {
            return Ok(());
        }

        // If that failed, maybe the local branch exists but isn't tracking - try to switch to it
        if Self::run_git_command_status(&["checkout", reference], Some(repo_path)).is_ok() {
            return Ok(());
        }

        // Try as a tag
        if Self::run_git_command_status(
            &["checkout", &format!("refs/tags/{reference}")],
            Some(repo_path),
        )
        .is_ok()
        {
            return Ok(());
        }

        Err(RexerError::GitError(format!(
            "Reference '{reference}' not found"
        )))
    }

    fn checkout_default_branch(repo_path: &Path) -> Result<()> {
        // First try to get the default branch from the remote HEAD
        let default_branch = Self::run_git_command(
            &["symbolic-ref", "refs/remotes/origin/HEAD"],
            Some(repo_path),
        )
        .or_else(|_| {
            // If that fails, try to set it based on the remote info
            Self::run_git_command_status(
                &["remote", "set-head", "origin", "--auto"],
                Some(repo_path),
            )
            .ok();
            Self::run_git_command(
                &["symbolic-ref", "refs/remotes/origin/HEAD"],
                Some(repo_path),
            )
        })
        .unwrap_or_else(|_| "origin/main".to_string());

        // Extract branch name from "refs/remotes/origin/branch_name" or "origin/branch_name"
        let branch_name = default_branch
            .strip_prefix("refs/remotes/origin/")
            .or_else(|| default_branch.strip_prefix("origin/"))
            .unwrap_or("main");

        // Try to checkout the branch, create it if it doesn't exist locally
        if Self::run_git_command_status(&["checkout", branch_name], Some(repo_path)).is_err() {
            // Try to create and checkout a local branch tracking the remote
            Self::run_git_command_status(
                &[
                    "checkout",
                    "-b",
                    branch_name,
                    &format!("origin/{}", branch_name),
                ],
                Some(repo_path),
            )?;
        }

        // Pull latest changes
        if Self::run_git_command_status(&["pull"], Some(repo_path)).is_err() {
            // If pull fails, just log and continue - might be in detached head state
            debug!("Pull failed, might be in detached HEAD state");
        }

        Ok(())
    }

    fn get_current_commit_hash(repo_path: &Path) -> Result<String> {
        Self::run_git_command(&["rev-parse", "HEAD"], Some(repo_path))
    }

    #[allow(dead_code)]
    pub fn get_latest_commit_hash(source: &Source) -> Result<String> {
        // Clone to a temporary directory to get the latest hash
        let temp_dir = tempfile::tempdir()?;
        let temp_path = temp_dir.path();

        let url = source.full_url();
        Self::run_git_command_status(&["clone", &url, &temp_path.to_string_lossy()], None)?;

        if let Some(reference) = source.reference() {
            Self::checkout_reference(temp_path, &reference)?;
        }

        Self::get_current_commit_hash(temp_path)
    }
}
