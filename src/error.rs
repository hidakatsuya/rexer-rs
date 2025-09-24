use thiserror::Error;

#[derive(Error, Debug)]
pub enum RexerError {
    #[error("Configuration file not found: {0}")]
    ConfigNotFound(String),

    #[error("Invalid configuration: {0}")]
    #[allow(dead_code)]
    InvalidConfig(String),

    #[error("Git operation failed: {0}")]
    GitError(String),

    #[error("Extension not found: {0}")]
    ExtensionNotFound(String),

    #[error("Lock file error: {0}")]
    LockFileError(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),

    #[error("YAML error: {0}")]
    YamlError(#[from] serde_yaml::Error),

    #[error("Anyhow error: {0}")]
    AnyhowError(#[from] anyhow::Error),
}

pub type Result<T> = std::result::Result<T, RexerError>;
