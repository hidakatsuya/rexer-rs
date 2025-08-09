use crate::error::{Result, RexerError};
use crate::extension::Source;
use anyhow::Context;
use git2::{BranchType, Repository};
use log::{debug, info, warn};
use std::path::Path;
use std::process::Command;

pub struct GitManager;

impl GitManager {
    pub fn clone_or_update(source: &Source, destination: &Path) -> Result<String> {
        if destination.exists() {
            Self::update_repository(source, destination)
        } else {
            Self::clone_repository(source, destination)
        }
    }

    fn clone_repository(source: &Source, destination: &Path) -> Result<String> {
        // Try using git2 library first
        match Self::clone_repository_with_git2(source, destination) {
            Ok(hash) => Ok(hash),
            Err(e) => {
                warn!("git2 clone failed, falling back to command-line git: {e}");
                Self::clone_repository_with_cli(source, destination)
            }
        }
    }

    fn clone_repository_with_git2(source: &Source, destination: &Path) -> Result<String> {
        let url = source.full_url();
        info!("Cloning {} to {} using git2", url, destination.display());

        // Ensure parent directory exists
        if let Some(parent) = destination.parent() {
            std::fs::create_dir_all(parent)?;
        }

        debug!("Starting git clone operation");
        let repo = Repository::clone(&url, destination).map_err(|e| {
            RexerError::GitError(format!("Failed to clone repository {url}: {e}"))
        })?;
        debug!("Git clone completed successfully");

        if let Some(reference) = source.reference() {
            debug!("Checking out reference: {reference}");
            Self::checkout_reference(&repo, &reference)?;
            debug!("Reference checkout completed");
        }

        debug!("Getting current commit hash");
        let commit_hash = Self::get_current_commit_hash(&repo)?;
        debug!("Commit hash retrieved: {commit_hash}");

        Ok(commit_hash)
    }

    fn clone_repository_with_cli(source: &Source, destination: &Path) -> Result<String> {
        let url = source.full_url();
        info!(
            "Cloning {} to {} using command-line git",
            url,
            destination.display()
        );

        // Ensure parent directory exists
        if let Some(parent) = destination.parent() {
            std::fs::create_dir_all(parent)?;
        }

        // Clone the repository
        let mut cmd = Command::new("git");
        cmd.arg("clone").arg(&url).arg(destination);

        let output = cmd
            .output()
            .with_context(|| "Failed to execute git clone command")?;

        if !output.status.success() {
            return Err(RexerError::GitError(format!(
                "git clone failed: {}",
                String::from_utf8_lossy(&output.stderr)
            )));
        }

        // Checkout specific reference if needed
        if let Some(reference) = source.reference() {
            debug!("Checking out reference: {reference}");
            Self::checkout_reference_with_cli(destination, &reference)?;
        }

        // Get current commit hash
        Self::get_current_commit_hash_with_cli(destination)
    }

    fn checkout_reference_with_cli(repo_dir: &Path, reference: &str) -> Result<()> {
        let mut cmd = Command::new("git");
        cmd.arg("-C").arg(repo_dir).arg("checkout").arg(reference);

        let output = cmd
            .output()
            .with_context(|| format!("Failed to checkout reference {reference}"))?;

        if !output.status.success() {
            return Err(RexerError::GitError(format!(
                "git checkout failed: {}",
                String::from_utf8_lossy(&output.stderr)
            )));
        }

        Ok(())
    }

    fn get_current_commit_hash_with_cli(repo_dir: &Path) -> Result<String> {
        let mut cmd = Command::new("git");
        cmd.arg("-C").arg(repo_dir).arg("rev-parse").arg("HEAD");

        let output = cmd
            .output()
            .with_context(|| "Failed to get current commit hash")?;

        if !output.status.success() {
            return Err(RexerError::GitError(format!(
                "git rev-parse failed: {}",
                String::from_utf8_lossy(&output.stderr)
            )));
        }

        let hash = String::from_utf8_lossy(&output.stdout).trim().to_string();
        Ok(hash)
    }

    fn update_repository(source: &Source, destination: &Path) -> Result<String> {
        // Try using git2 library first
        match Self::update_repository_with_git2(source, destination) {
            Ok(hash) => Ok(hash),
            Err(e) => {
                warn!(
                    "git2 update failed, falling back to command-line git: {e}"
                );
                Self::update_repository_with_cli(source, destination)
            }
        }
    }

    fn update_repository_with_git2(source: &Source, destination: &Path) -> Result<String> {
        let url = source.full_url();
        info!("Updating {} at {} using git2", url, destination.display());

        let repo = Repository::open(destination)?;

        // Fetch latest changes
        let mut remote = repo.find_remote("origin")?;
        remote.fetch(&[] as &[&str], None, None)?;

        if let Some(reference) = source.reference() {
            Self::checkout_reference(&repo, &reference)?;
        } else {
            // Checkout default branch
            Self::checkout_default_branch(&repo)?;
        }

        Self::get_current_commit_hash(&repo)
    }

    fn update_repository_with_cli(source: &Source, destination: &Path) -> Result<String> {
        let url = source.full_url();
        info!(
            "Updating {} at {} using command-line git",
            url,
            destination.display()
        );

        // Fetch latest changes
        let mut cmd = Command::new("git");
        cmd.arg("-C").arg(destination).arg("fetch").arg("origin");

        let output = cmd
            .output()
            .with_context(|| "Failed to fetch from origin")?;

        if !output.status.success() {
            return Err(RexerError::GitError(format!(
                "git fetch failed: {}",
                String::from_utf8_lossy(&output.stderr)
            )));
        }

        // Checkout specific reference if needed
        if let Some(reference) = source.reference() {
            Self::checkout_reference_with_cli(destination, &reference)?;
        } else {
            // Reset to origin/main or origin/master
            Self::reset_to_default_branch_with_cli(destination)?;
        }

        Self::get_current_commit_hash_with_cli(destination)
    }

    fn reset_to_default_branch_with_cli(repo_dir: &Path) -> Result<()> {
        // Try origin/main first, then origin/master
        for branch in &["origin/main", "origin/master"] {
            let mut cmd = Command::new("git");
            cmd.arg("-C")
                .arg(repo_dir)
                .arg("reset")
                .arg("--hard")
                .arg(branch);

            let output = cmd.output().ok();
            if let Some(output) = output {
                if output.status.success() {
                    return Ok(());
                }
            }
        }

        Err(RexerError::GitError(
            "Failed to reset to default branch".to_string(),
        ))
    }

    fn checkout_reference(repo: &Repository, reference: &str) -> Result<()> {
        debug!("Checking out reference: {reference}");

        // Try to find the reference as a branch first
        if let Ok(branch) = repo.find_branch(reference, BranchType::Local) {
            debug!("Found local branch: {reference}");
            let commit = branch.get().peel_to_commit().map_err(|e| {
                RexerError::GitError(format!(
                    "Failed to get commit for local branch {reference}: {e}"
                ))
            })?;
            repo.checkout_tree(commit.as_object(), None).map_err(|e| {
                RexerError::GitError(format!(
                    "Failed to checkout tree for local branch {reference}: {e}"
                ))
            })?;
            repo.set_head(&format!("refs/heads/{reference}"))
                .map_err(|e| {
                    RexerError::GitError(format!(
                        "Failed to set head for local branch {reference}: {e}"
                    ))
                })?;
            debug!("Successfully checked out local branch: {reference}");
            return Ok(());
        }

        // Try remote branch
        if let Ok(branch) = repo.find_branch(&format!("origin/{reference}"), BranchType::Remote) {
            debug!("Found remote branch: origin/{reference}");
            let commit = branch.get().peel_to_commit().map_err(|e| {
                RexerError::GitError(format!(
                    "Failed to get commit for remote branch {reference}: {e}"
                ))
            })?;
            repo.checkout_tree(commit.as_object(), None).map_err(|e| {
                RexerError::GitError(format!(
                    "Failed to checkout tree for remote branch {reference}: {e}"
                ))
            })?;

            // Create local branch tracking remote
            repo.branch(reference, &commit, false).map_err(|e| {
                RexerError::GitError(format!(
                    "Failed to create local branch {reference}: {e}"
                ))
            })?;
            repo.set_head(&format!("refs/heads/{reference}"))
                .map_err(|e| {
                    RexerError::GitError(format!(
                        "Failed to set head for remote branch {reference}: {e}"
                    ))
                })?;
            debug!("Successfully checked out remote branch: {reference}");
            return Ok(());
        }

        // Try as a tag
        if let Ok(tag_ref) = repo.find_reference(&format!("refs/tags/{reference}")) {
            debug!("Found tag: {reference}");
            let commit = tag_ref.peel_to_commit().map_err(|e| {
                RexerError::GitError(format!("Failed to get commit for tag {reference}: {e}"))
            })?;
            repo.checkout_tree(commit.as_object(), None).map_err(|e| {
                RexerError::GitError(format!(
                    "Failed to checkout tree for tag {reference}: {e}"
                ))
            })?;
            repo.set_head_detached(commit.id()).map_err(|e| {
                RexerError::GitError(format!(
                    "Failed to set detached head for tag {reference}: {e}"
                ))
            })?;
            debug!("Successfully checked out tag: {reference}");
            return Ok(());
        }

        // Try as a commit hash
        if let Ok(oid) = git2::Oid::from_str(reference) {
            if let Ok(commit) = repo.find_commit(oid) {
                debug!("Found commit: {reference}");
                repo.checkout_tree(commit.as_object(), None).map_err(|e| {
                    RexerError::GitError(format!(
                        "Failed to checkout tree for commit {reference}: {e}"
                    ))
                })?;
                repo.set_head_detached(commit.id()).map_err(|e| {
                    RexerError::GitError(format!(
                        "Failed to set detached head for commit {reference}: {e}"
                    ))
                })?;
                debug!("Successfully checked out commit: {reference}");
                return Ok(());
            }
        }

        Err(RexerError::GitError(format!(
            "Reference '{reference}' not found"
        )))
    }

    fn checkout_default_branch(repo: &Repository) -> Result<()> {
        let head = repo.head()?;
        let commit = head.peel_to_commit()?;
        repo.checkout_tree(commit.as_object(), None)?;
        Ok(())
    }

    fn get_current_commit_hash(repo: &Repository) -> Result<String> {
        debug!("Getting current commit hash");
        let head = repo
            .head()
            .map_err(|e| RexerError::GitError(format!("Failed to get repository head: {e}")))?;
        let commit = head
            .peel_to_commit()
            .map_err(|e| RexerError::GitError(format!("Failed to get commit from head: {e}")))?;
        let hash = commit.id().to_string();
        debug!("Current commit hash: {hash}");
        Ok(hash)
    }

    #[allow(dead_code)]
    pub fn get_latest_commit_hash(source: &Source) -> Result<String> {
        // For now, we'll need to clone to a temporary directory to get the latest hash
        let temp_dir = tempfile::tempdir()?;
        let temp_path = temp_dir.path();

        let url = source.full_url();
        let repo = Repository::clone(&url, temp_path)?;

        if let Some(reference) = source.reference() {
            Self::checkout_reference(&repo, &reference)?;
        }

        Self::get_current_commit_hash(&repo)
    }
}
