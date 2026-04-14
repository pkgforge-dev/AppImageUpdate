mod forge;
mod generic;
mod parser;

pub use forge::ForgeUpdateInfo;
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
    Forge(ForgeUpdateInfo),
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

    pub fn with_target_tag(mut self, tag: &str) -> Self {
        if let UpdateInfoInner::Forge(ref mut f) = self.inner {
            f.set_tag(tag.to_owned());
        }
        self
    }

    pub fn is_forge(&self) -> bool {
        matches!(self.inner, UpdateInfoInner::Forge(_))
    }

    pub fn zsync_url(&self) -> Result<String> {
        match &self.inner {
            UpdateInfoInner::Generic(g) => Ok(g.zsync_url().to_owned()),
            UpdateInfoInner::Forge(f) => f.zsync_url().map(|s| s.to_owned()),
        }
    }
}
