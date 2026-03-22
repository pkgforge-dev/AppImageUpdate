mod generic;
mod github;
mod parser;

pub use generic::GenericUpdateInfo;
pub use github::GitHubUpdateInfo;

use crate::error::Result;

#[derive(Debug, Clone)]
pub enum UpdateInfo {
    Generic(GenericUpdateInfo),
    GitHub(GitHubUpdateInfo),
}

impl UpdateInfo {
    pub fn parse(s: &str) -> Result<Self> {
        parser::parse(s)
    }

    pub fn zsync_url(&self) -> Result<String> {
        match self {
            UpdateInfo::Generic(g) => Ok(g.zsync_url().to_owned()),
            UpdateInfo::GitHub(g) => g.zsync_url().map(|s| s.to_owned()),
        }
    }
}
