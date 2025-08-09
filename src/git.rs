use crate::error::{Result, RexerError};
use crate::extension::Source;
use git2::{BranchType, Repository};
use log::{debug, info, warn};
use std::path::Path;
use std::process::Command;

pub struct GitManager;

impl GitManager {
    pub fn clone_or_update(source: &Source, destination: &Path) -> Result<String> {
        // Try git2 first, fall back to command-line git if it fails
        match Self::clone_or_update_with_git2(source, destination) {
            Ok(hash) => {
                debug!("git2 operation successful");
                Ok(hash)
            }
            Err(e) => {
                warn!("git2 operation failed, falling back to CLI: {}", e);
                Self::clone_or_update_with_cli(source, destination)
            }
        }
    }

    fn clone_or_update_with_git2(source: &Source, destination: &Path) -> Result<String> {
        if destination.exists() {
            Self::update_repository_with_git2(source, destination)
        } else {
            Self::clone_repository_with_git2(source, destination)
        }
    }

    fn clone_or_update_with_cli(source: &Source, destination: &Path) -> Result<String> {
        if destination.exists() {
            Self::update_repository_with_cli(source, destination)
        } else {
            Self::clone_repository_with_cli(source, destination)
        }
    }

    fn clone_repository_with_git2(source: &Source, destination: &Path) -> Result<String> {
        let url = source.full_url();
        info!("Cloning {} to {} using git2", url, destination.display());

        // Ensure parent directory exists
        if let Some(parent) = destination.parent() {
            std::fs::create_dir_all(parent)?;
        }

        // Use a more careful approach to cloning
        let repo = {
            debug!("Starting repository clone");
            let result = Repository::clone(&url, destination);
            match result {
                Ok(repo) => {
                    debug!("Repository clone successful");
                    repo
                }
                Err(e) => {
                    return Err(RexerError::GitError(format!("Failed to clone {}: {}", url, e)));
                }
            }
        };

        // Handle reference checkout
        if let Some(reference) = source.reference() {
            debug!("Checking out reference: {}", reference);
            Self::checkout_reference(&repo, &reference)?;
        }

        // Get commit hash and ensure repo is properly finalized
        let commit_hash = Self::get_current_commit_hash(&repo)?;
        
        // Explicitly drop the repository to ensure cleanup
        drop(repo);
        
        debug!("Clone operation completed successfully");
        Ok(commit_hash)
    }

    fn clone_repository_with_cli(source: &Source, destination: &Path) -> Result<String> {
        let url = source.full_url();
        info!("Cloning {} to {} using CLI git", url, destination.display());

        // Ensure parent directory exists
        if let Some(parent) = destination.parent() {
            std::fs::create_dir_all(parent)?;
        }

        // Clone repository using git command
        let output = Command::new("git")
            .args(&["clone", &url])
            .arg(destination)
            .output()
            .map_err(|e| RexerError::GitError(format!("Failed to execute git clone: {}", e)))?;

        if !output.status.success() {
            return Err(RexerError::GitError(format!(
                "git clone failed: {}",
                String::from_utf8_lossy(&output.stderr)
            )));
        }

        // Checkout specific reference if needed
        if let Some(reference) = source.reference() {
            Self::checkout_reference_with_cli(destination, &reference)?;
        }

        Self::get_current_commit_hash_with_cli(destination)
    }

    fn checkout_reference_with_cli(repo_path: &Path, reference: &str) -> Result<()> {
        let output = Command::new("git")
            .args(&["-C"])
            .arg(repo_path)
            .args(&["checkout", reference])
            .output()
            .map_err(|e| RexerError::GitError(format!("Failed to execute git checkout: {}", e)))?;

        if !output.status.success() {
            return Err(RexerError::GitError(format!(
                "git checkout failed: {}",
                String::from_utf8_lossy(&output.stderr)
            )));
        }

        Ok(())
    }

    fn get_current_commit_hash_with_cli(repo_path: &Path) -> Result<String> {
        let output = Command::new("git")
            .args(&["-C"])
            .arg(repo_path)
            .args(&["rev-parse", "HEAD"])
            .output()
            .map_err(|e| RexerError::GitError(format!("Failed to execute git rev-parse: {}", e)))?;

        if !output.status.success() {
            return Err(RexerError::GitError(format!(
                "git rev-parse failed: {}",
                String::from_utf8_lossy(&output.stderr)
            )));
        }

        let hash = String::from_utf8_lossy(&output.stdout)
            .trim()
            .to_string();
        Ok(hash)
    }

    fn update_repository_with_git2(source: &Source, destination: &Path) -> Result<String> {
        let url = source.full_url();
        info!("Updating {} at {} using git2", url, destination.display());

        let repo = Repository::open(destination)
            .map_err(|e| RexerError::GitError(format!("Failed to open repository: {}", e)))?;

        // Fetch latest changes
        {
            let mut remote = repo.find_remote("origin")
                .map_err(|e| RexerError::GitError(format!("Failed to find origin remote: {}", e)))?;
            remote.fetch(&[] as &[&str], None, None)
                .map_err(|e| RexerError::GitError(format!("Failed to fetch: {}", e)))?;
        }

        if let Some(reference) = source.reference() {
            Self::checkout_reference(&repo, &reference)?;
        } else {
            // Checkout default branch
            Self::checkout_default_branch(&repo)?;
        }

        let commit_hash = Self::get_current_commit_hash(&repo)?;
        
        // Explicitly drop repository
        drop(repo);
        
        Ok(commit_hash)
    }

    fn update_repository_with_cli(source: &Source, destination: &Path) -> Result<String> {
        let url = source.full_url();
        info!("Updating {} at {} using CLI git", url, destination.display());

        // Fetch latest changes
        let output = Command::new("git")
            .args(&["-C"])
            .arg(destination)
            .args(&["fetch", "origin"])
            .output()
            .map_err(|e| RexerError::GitError(format!("Failed to execute git fetch: {}", e)))?;

        if !output.status.success() {
            return Err(RexerError::GitError(format!(
                "git fetch failed: {}",
                String::from_utf8_lossy(&output.stderr)
            )));
        }

        if let Some(reference) = source.reference() {
            Self::checkout_reference_with_cli(destination, &reference)?;
        } else {
            // Reset to default branch
            Self::reset_to_default_branch_with_cli(destination)?;
        }

        Self::get_current_commit_hash_with_cli(destination)
    }

    fn reset_to_default_branch_with_cli(repo_path: &Path) -> Result<()> {
        // Try origin/main first, then origin/master
        for branch in &["origin/main", "origin/master"] {
            let output = Command::new("git")
                .args(&["-C"])
                .arg(repo_path)
                .args(&["reset", "--hard", branch])
                .output();

            if let Ok(output) = output {
                if output.status.success() {
                    return Ok(());
                }
            }
        }

        Err(RexerError::GitError("Failed to reset to default branch".to_string()))
    }

    fn checkout_reference(repo: &Repository, reference: &str) -> Result<()> {
        debug!("Checking out reference: {reference}");

        // Try to find the reference as a branch first
        if let Ok(branch) = repo.find_branch(reference, BranchType::Local) {
            let commit = branch.get().peel_to_commit()?;
            repo.checkout_tree(commit.as_object(), None)?;
            repo.set_head(&format!("refs/heads/{reference}"))?;
            return Ok(());
        }

        // Try remote branch
        if let Ok(branch) = repo.find_branch(&format!("origin/{reference}"), BranchType::Remote) {
            let commit = branch.get().peel_to_commit()?;
            repo.checkout_tree(commit.as_object(), None)?;

            // Create local branch tracking remote
            repo.branch(reference, &commit, false)?;
            repo.set_head(&format!("refs/heads/{reference}"))?;
            return Ok(());
        }

        // Try as a tag
        if let Ok(tag_ref) = repo.find_reference(&format!("refs/tags/{reference}")) {
            let commit = tag_ref.peel_to_commit()?;
            repo.checkout_tree(commit.as_object(), None)?;
            repo.set_head_detached(commit.id())?;
            return Ok(());
        }

        // Try as a commit hash
        if let Ok(oid) = git2::Oid::from_str(reference) {
            if let Ok(commit) = repo.find_commit(oid) {
                repo.checkout_tree(commit.as_object(), None)?;
                repo.set_head_detached(commit.id())?;
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
        let head = repo.head()?;
        let commit = head.peel_to_commit()?;
        Ok(commit.id().to_string())
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
