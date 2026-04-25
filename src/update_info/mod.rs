mod forge;
mod generic;
mod parser;

pub use forge::ForgeKind;
pub use forge::ForgeUpdateInfo;
pub use forge::ReleaseInfo;
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

    pub fn forge_info(&self) -> Option<&ForgeUpdateInfo> {
        match &self.inner {
            UpdateInfoInner::Forge(f) => Some(f),
            UpdateInfoInner::Generic(_) => None,
        }
    }

    pub fn generic_info(&self) -> Option<&GenericUpdateInfo> {
        match &self.inner {
            UpdateInfoInner::Generic(g) => Some(g),
            UpdateInfoInner::Forge(_) => None,
        }
    }

    pub fn type_label(&self) -> &'static str {
        match &self.inner {
            UpdateInfoInner::Generic(_) => "zsync",
            UpdateInfoInner::Forge(f) => f.kind.label(),
        }
    }

    pub fn type_display_name(&self) -> &'static str {
        match &self.inner {
            UpdateInfoInner::Generic(_) => "Generic ZSync",
            UpdateInfoInner::Forge(f) => f.kind.display_name(),
        }
    }

    pub fn list_releases(&self) -> Result<Vec<ReleaseInfo>> {
        match &self.inner {
            UpdateInfoInner::Generic(_) => Err(crate::error::Error::InvalidUpdateInfo(
                "--list-releases is only supported for forge-based update info".into(),
            )),
            UpdateInfoInner::Forge(f) => f.list_releases(),
        }
    }

    pub fn zsync_url(&self) -> Result<String> {
        match &self.inner {
            UpdateInfoInner::Generic(g) => Ok(g.zsync_url().to_owned()),
            UpdateInfoInner::Forge(f) => f.zsync_url().map(|s| s.to_owned()),
        }
    }
}
