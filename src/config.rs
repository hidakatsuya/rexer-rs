use crate::error::{Result, RexerError};
use crate::extension::{ExtensionsConfig, LockFile};
use std::fs;
use std::path::PathBuf;

pub const EXTENSIONS_FILE: &str = ".extensions.yml";
pub const LOCK_FILE: &str = ".extensions.lock";

pub struct Config {
    pub command_prefix: Option<String>,
    pub redmine_root: PathBuf,
}

impl Config {
    pub fn new() -> Result<Self> {
        let redmine_root = std::env::current_dir()?;
        let command_prefix = std::env::var("REXER_COMMAND_PREFIX").ok();

        Ok(Self {
            command_prefix,
            redmine_root,
        })
    }

    pub fn extensions_file_path(&self) -> PathBuf {
        self.redmine_root.join(EXTENSIONS_FILE)
    }

    pub fn lock_file_path(&self) -> PathBuf {
        self.redmine_root.join(LOCK_FILE)
    }

    pub fn plugins_dir(&self) -> PathBuf {
        self.redmine_root.join("plugins")
    }

    pub fn themes_dir(&self) -> PathBuf {
        self.redmine_root.join("themes")
    }

    pub fn load_extensions_config(&self) -> Result<ExtensionsConfig> {
        let path = self.extensions_file_path();
        if !path.exists() {
            return Err(RexerError::ConfigNotFound(path.display().to_string()));
        }

        let content = fs::read_to_string(&path)?;
        let config: ExtensionsConfig = serde_yaml::from_str(&content)?;
        Ok(config)
    }

    #[allow(dead_code)]
    pub fn save_extensions_config(&self, config: &ExtensionsConfig) -> Result<()> {
        let path = self.extensions_file_path();
        let content = serde_yaml::to_string(config)?;
        fs::write(&path, content)?;
        Ok(())
    }

    pub fn load_lock_file(&self) -> Result<Option<LockFile>> {
        let path = self.lock_file_path();
        if !path.exists() {
            return Ok(None);
        }

        let content = fs::read_to_string(&path)?;
        let lock_file: LockFile = serde_json::from_str(&content)?;
        Ok(Some(lock_file))
    }

    pub fn save_lock_file(&self, lock_file: &LockFile) -> Result<()> {
        let path = self.lock_file_path();
        
        // Use defensive serialization approach for release builds
        let content = if cfg!(debug_assertions) {
            // Debug build: use pretty printing for better debugging
            serde_json::to_string_pretty(lock_file)
                .map_err(|e| RexerError::LockFileError(format!("JSON serialization failed: {e}")))?
        } else {
            // Release build: use compact serialization to minimize memory issues
            serde_json::to_string(lock_file)
                .map_err(|e| RexerError::LockFileError(format!("JSON serialization failed: {e}")))?
        };
        
        // Defensive file writing with atomic operation
        let temp_path = path.with_extension("lock.tmp");
        
        // Write to temporary file first
        fs::write(&temp_path, &content)
            .map_err(|e| RexerError::LockFileError(format!("Failed to write temporary lock file: {e}")))?;
        
        // Atomically move to final location
        fs::rename(&temp_path, &path)
            .map_err(|e| RexerError::LockFileError(format!("Failed to finalize lock file: {e}")))?;
        
        Ok(())
    }

    pub fn delete_lock_file(&self) -> Result<()> {
        let path = self.lock_file_path();
        if path.exists() {
            fs::remove_file(&path)?;
        }
        Ok(())
    }

    pub fn create_initial_config(&self) -> Result<()> {
        // Add example configuration in YAML format
        let example_content = r#"# Redmine Extensions Configuration
# Define plugins and themes to be managed by rexer

plugins:
  # Example plugin from GitHub
  # - name: redmine_issues_panel
  #   github:
  #     repo: "redmica/redmine_issues_panel"
  #     tag: "v1.0.2"

themes:
  # Example theme from Git repository  
  # - name: my_theme
  #   git:
  #     url: "https://github.com/user/my_theme.git"
  #     branch: "main"
"#;

        let path = self.extensions_file_path();
        fs::write(&path, example_content)?;
        Ok(())
    }
}
