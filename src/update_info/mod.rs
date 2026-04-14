mod forge;
mod generic;
mod parser;

pub use forge::GitHubUpdateInfo;
pub use generic::GenericUpdateInfo;

use crate::error::Result;

#[derive(Debug, Clone)]
pub struct UpdateInfo {
    raw: String,
    inner: UpdateInfoInner,
}

#[derive(Debug, Clone)]
enum UpdateInfoInner {
    Generic(GenericUpdateInfo),
    GitHub(GitHubUpdateInfo),
}

impl UpdateInfo {
    pub fn parse(s: &str) -> Result<Self> {
        let inner = parser::parse(s)?;
        Ok(Self {
            raw: s.to_owned(),
            inner,
        })
    }

    pub fn raw(&self) -> &str {
        &self.raw
    }

    pub fn zsync_url(&self) -> Result<String> {
        match &self.inner {
            UpdateInfoInner::Generic(g) => Ok(g.zsync_url().to_owned()),
            UpdateInfoInner::GitHub(g) => g.zsync_url().map(|s| s.to_owned()),
        }
    }
}
