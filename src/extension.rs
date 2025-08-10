use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::git::GitManager;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Extension {
    pub name: String,
    #[serde(flatten)]
    pub source: Source,
}

impl Extension {
    pub fn load(&self, path: &Path) -> Result<()> {
        if let Some(git) = self.source.git() {
            GitManager::clone_or_update(source, destination)
            OK(())
        }

        if let Some(github) = self.source.github() {
            // Load extension from GitHub source
            OK(())
        }
    }
}

trait SourceLoader {
    fn load(&self, path: &Path) -> Result<()>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Git {
    pub url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub branch: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tag: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub commit: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHub {
    pub repo: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub branch: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tag: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub commit: Option<String>,
}

impl GitHub {
    pub fn url(&self) -> String {
        format!("https://github.com/{}.git", self.repo)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Source {
    #[serde(rename = "git")]
    Git(Git),
    #[serde(rename = "github")]
    GitHub(GitHub),
}

impl Source {
    pub fn git(&self) -> Option<&Git> {
        match self {
            Source::Git(git) => Some(git),
            _ => None,
        }
    }

    pub fn github(&self) -> Option<&GitHub> {
        match self {
            Source::GitHub(github) => Some(github),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtensionsConfig {
    #[serde(default)]
    pub plugins: Vec<Extension>,
    #[serde(default)]
    pub themes: Vec<Extension>,
}

impl ExtensionsConfig {
    pub fn all_extensions(&self) -> impl Iterator<Item = (&Extension, ExtensionType)> {
        self.plugins
            .iter()
            .map(|e| (e, ExtensionType::Plugin))
            .chain(self.themes.iter().map(|e| (e, ExtensionType::Theme)))
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum ExtensionType {
    Plugin,
    Theme,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LockFile {
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
