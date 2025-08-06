use thiserror::Error;

#[derive(Error, Debug)]
pub enum RexerError {
    #[error("Configuration file not found: {0}")]
    ConfigNotFound(String),
    
    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),
    
    #[error("Git operation failed: {0}")]
    GitError(String),
    
    #[error("Extension not found: {0}")]
    ExtensionNotFound(String),
    
    #[error("Environment not found: {0}")]
    EnvironmentNotFound(String),
    
    #[error("Lock file error: {0}")]
    LockFileError(String),
    
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    
    #[error("Git error: {0}")]
    Git2Error(#[from] git2::Error),
    
    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),
    
    #[error("Anyhow error: {0}")]
    AnyhowError(#[from] anyhow::Error),
}

pub type Result<T> = std::result::Result<T, RexerError>;