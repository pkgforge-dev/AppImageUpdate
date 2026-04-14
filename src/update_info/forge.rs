use std::cell::OnceCell;

use releasekit::client::UreqClient;
use releasekit::platform::{GitHub, GitLab};
use releasekit::{Filter, Forge, Release};

use crate::config;
use crate::error::{Error, Result};

#[derive(Debug, Clone)]
pub enum ForgeKind {
    GitHub,
    GitLab,
}

#[derive(Debug, Clone)]
pub struct ForgeUpdateInfo {
    pub kind: ForgeKind,
    pub owner: String,
    pub repo: String,
    pub tag: String,
    pub filename: String,
    resolved_url: OnceCell<String>,
}

impl ForgeUpdateInfo {
    pub fn new(
        kind: ForgeKind,
        owner: String,
        repo: String,
        tag: String,
        filename: String,
    ) -> Self {
        Self {
            kind,
            owner,
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
        match &self.kind {
            ForgeKind::GitHub => self.resolve_github(),
            ForgeKind::GitLab => {
                let gl = GitLab::new(UreqClient).with_token_from_env(&["GITLAB_TOKEN", "GL_TOKEN"]);
                self.fetch_with_forge(gl)
            }
        }
    }

    fn resolve_github(&self) -> Result<String> {
        let proxies = config::get_proxies();

        if proxies.is_empty() {
            let gh = GitHub::new(UreqClient).with_token_from_env(&["GITHUB_TOKEN", "GH_TOKEN"]);
            return self.fetch_with_forge(gh);
        }

        let mut last_error = None;
        for proxy in &proxies {
            let gh = GitHub::new(UreqClient)
                .with_base_url(proxy)
                .with_token_from_env(&["GITHUB_TOKEN", "GH_TOKEN"]);
            match self.fetch_with_forge(gh) {
                Ok(url) => return Ok(url),
                Err(e) => last_error = Some(e),
            }
        }

        Err(last_error.unwrap_or_else(|| Error::ForgeApi("All proxies failed".into())))
    }

    fn fetch_with_forge(&self, forge: impl Forge) -> Result<String> {
        let project = format!("{}/{}", self.owner, self.repo);

        let (tag, find_prerelease) = match self.tag.as_str() {
            "latest" | "latest-pre" | "latest-all" => (None, self.tag == "latest-pre"),
            tag => (Some(tag), false),
        };

        let releases = forge
            .fetch_releases(&project, tag)
            .map_err(|e| Error::ForgeApi(e.to_string()))?;

        let release = self.select_release(releases, tag.is_some(), find_prerelease)?;

        self.find_matching_asset(&release)
    }

    fn select_release(
        &self,
        releases: Vec<Release>,
        is_specific_tag: bool,
        find_prerelease: bool,
    ) -> Result<Release> {
        if is_specific_tag {
            return releases
                .into_iter()
                .next()
                .ok_or_else(|| Error::ForgeApi("No release found for tag".into()));
        }

        if find_prerelease {
            return releases
                .into_iter()
                .find(|r| r.is_prerelease())
                .ok_or_else(|| Error::ForgeApi("No prerelease found".into()));
        }

        if self.tag == "latest" {
            return releases
                .into_iter()
                .find(|r| !r.is_prerelease())
                .ok_or_else(|| Error::ForgeApi("No stable release found".into()));
        }

        // latest-all: first release regardless
        releases
            .into_iter()
            .next()
            .ok_or_else(|| Error::ForgeApi("No release found".into()))
    }

    fn find_matching_asset(&self, release: &Release) -> Result<String> {
        let filter = Filter::new().glob(format!("*{}", self.filename));

        let mut matching: Vec<&str> = release
            .assets()
            .iter()
            .filter(|a| filter.matches(a.name()))
            .map(|a| a.url())
            .collect();

        if matching.is_empty() {
            return Err(Error::ForgeApi(format!(
                "No asset matched pattern: *{}",
                self.filename
            )));
        }

        matching.sort();
        matching.reverse();

        Ok(matching[0].to_string())
    }
}
