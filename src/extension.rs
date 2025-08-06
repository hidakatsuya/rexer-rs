use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Extension {
    pub name: String,
    #[serde(flatten)]
    pub source: Source,
    pub hooks: Option<Hooks>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "source_type")]
pub enum Source {
    #[serde(rename = "git")]
    Git { url: String, reference: Option<String> },
    #[serde(rename = "github")]
    GitHub { repo: String, reference: Option<String> },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Hooks {
    pub installed: Option<String>,
    pub uninstalled: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtensionsConfig {
    #[serde(default)]
    pub plugins: Vec<Extension>,
    #[serde(default)]
    pub themes: Vec<Extension>,
}

impl ExtensionsConfig {
    pub fn new() -> Self {
        Self {
            plugins: Vec::new(),
            themes: Vec::new(),
        }
    }

    pub fn all_extensions(&self) -> impl Iterator<Item = (&Extension, ExtensionType)> {
        self.plugins.iter().map(|e| (e, ExtensionType::Plugin))
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

impl Source {
    pub fn full_url(&self) -> String {
        match self {
            Source::Git { url, .. } => url.clone(),
            Source::GitHub { repo, .. } => {
                if repo.starts_with("http") {
                    repo.clone()
                } else {
                    format!("https://github.com/{}.git", repo)
                }
            }
        }
    }

    pub fn reference(&self) -> Option<&String> {
        match self {
            Source::Git { reference, .. } => reference.as_ref(),
            Source::GitHub { reference, .. } => reference.as_ref(),
        }
    }
}
