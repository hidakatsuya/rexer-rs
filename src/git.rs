use crate::extension::Source;
use crate::error::{RexerError, Result};
use git2::{Repository, BranchType, ObjectType};
use std::path::Path;
use log::{info, debug};

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
        let url = source.full_url();
        info!("Cloning {} to {}", url, destination.display());
        
        let repo = Repository::clone(&url, destination)?;
        
        if let Some(reference) = &source.reference {
            Self::checkout_reference(&repo, reference)?;
        }
        
        Self::get_current_commit_hash(&repo)
    }
    
    fn update_repository(source: &Source, destination: &Path) -> Result<String> {
        let url = source.full_url();
        info!("Updating {} at {}", url, destination.display());
        
        let repo = Repository::open(destination)?;
        
        // Fetch latest changes
        let mut remote = repo.find_remote("origin")?;
        remote.fetch(&[] as &[&str], None, None)?;
        
        if let Some(reference) = &source.reference {
            Self::checkout_reference(&repo, reference)?;
        } else {
            // Checkout default branch
            Self::checkout_default_branch(&repo)?;
        }
        
        Self::get_current_commit_hash(&repo)
    }
    
    fn checkout_reference(repo: &Repository, reference: &str) -> Result<()> {
        debug!("Checking out reference: {}", reference);
        
        // Try to find the reference as a branch first
        if let Ok(branch) = repo.find_branch(reference, BranchType::Local) {
            let commit = branch.get().peel_to_commit()?;
            repo.checkout_tree(commit.as_object(), None)?;
            repo.set_head(&format!("refs/heads/{}", reference))?;
            return Ok(());
        }
        
        // Try remote branch
        if let Ok(branch) = repo.find_branch(&format!("origin/{}", reference), BranchType::Remote) {
            let commit = branch.get().peel_to_commit()?;
            repo.checkout_tree(commit.as_object(), None)?;
            
            // Create local branch tracking remote
            repo.branch(reference, &commit, false)?;
            repo.set_head(&format!("refs/heads/{}", reference))?;
            return Ok(());
        }
        
        // Try as a tag
        if let Ok(tag_ref) = repo.find_reference(&format!("refs/tags/{}", reference)) {
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
        
        Err(RexerError::GitError(format!("Reference '{}' not found", reference)))
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
    
    pub fn get_latest_commit_hash(source: &Source) -> Result<String> {
        // For now, we'll need to clone to a temporary directory to get the latest hash
        let temp_dir = tempfile::tempdir()?;
        let temp_path = temp_dir.path();
        
        let url = source.full_url();
        let repo = Repository::clone(&url, temp_path)?;
        
        if let Some(reference) = &source.reference {
            Self::checkout_reference(&repo, reference)?;
        }
        
        Self::get_current_commit_hash(&repo)
    }
}