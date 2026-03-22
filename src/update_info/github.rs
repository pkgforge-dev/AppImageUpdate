use std::cell::OnceCell;

use crate::error::Result;

#[derive(Debug, Clone)]
pub struct GitHubUpdateInfo {
    pub username: String,
    pub repo: String,
    pub tag: String,
    pub filename: String,
    resolved_url: OnceCell<String>,
}

impl GitHubUpdateInfo {
    pub fn new(username: String, repo: String, tag: String, filename: String) -> Self {
        Self {
            username,
            repo,
            tag,
            filename,
            resolved_url: OnceCell::new(),
        }
    }

    pub fn zsync_url(&self) -> Result<&str> {
        if self.resolved_url.get().is_none() {
            let url = self.resolve_url()?;
            let _ = self.resolved_url.set(url);
        }
        Ok(self.resolved_url.get().unwrap())
    }

    fn resolve_url(&self) -> Result<String> {
        todo!("Implement GitHub API call to resolve zsync URL")
    }
}
