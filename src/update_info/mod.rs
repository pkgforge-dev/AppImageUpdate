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
}
