use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Extension {
    pub name: String,
    pub extension_type: ExtensionType,
    pub source: Source,
    pub hooks: Option<Hooks>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExtensionType {
    Plugin,
    Theme,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Source {
    pub source_type: SourceType,
    pub url: String,
    pub reference: Option<String>, // branch, tag, or commit
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SourceType {
    Git,
    GitHub,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Hooks {
    pub installed: Option<String>,
    pub uninstalled: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Environment {
    pub name: String,
    pub extensions: Vec<Extension>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtensionsConfig {
    pub environments: HashMap<String, Vec<Extension>>,
}

impl ExtensionsConfig {
    pub fn new() -> Self {
        Self {
            environments: HashMap::new(),
        }
    }
    
    pub fn get_environment(&self, name: &str) -> Option<&Vec<Extension>> {
        self.environments.get(name)
    }
    
    pub fn add_extension_to_env(&mut self, env_name: &str, extension: Extension) {
        self.environments
            .entry(env_name.to_string())
            .or_insert_with(Vec::new)
            .push(extension);
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LockFile {
    pub environment: String,
    pub extensions: Vec<LockedExtension>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LockedExtension {
    pub name: String,
    pub extension_type: ExtensionType,
    pub source: Source,
    pub commit_hash: Option<String>,
    pub installed_at: String,
}

impl Source {
    pub fn full_url(&self) -> String {
        match self.source_type {
            SourceType::Git => self.url.clone(),
            SourceType::GitHub => {
                if self.url.starts_with("http") {
                    self.url.clone()
                } else {
                    format!("https://github.com/{}.git", self.url)
                }
            }
        }
    }
}