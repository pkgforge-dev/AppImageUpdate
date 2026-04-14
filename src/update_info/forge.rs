use std::cell::OnceCell;

pub use releasekit::Release as ReleaseInfo;
use releasekit::client::UreqClient;
use releasekit::platform::{GitHub, GitLab, Gitea};
use releasekit::{Filter, Forge, Release};

use crate::config;
use crate::error::{Error, Result};

fn forge_error(e: releasekit::Error, project: &str) -> Error {
    let msg = match &e {
        releasekit::Error::Http { status, url } => {
            format!("HTTP {status} from {url}")
        }
        releasekit::Error::Network(detail) => {
            let detail = detail.strip_prefix("io: ").unwrap_or(detail);
            format!("failed to connect to release server for {project}: {detail}")
        }
        releasekit::Error::NoReleases => {
            format!("no releases found for {project}")
        }
        releasekit::Error::NoMatchingAsset => {
            format!("no matching asset found for {project}")
        }
        _ => format!("{e}"),
    };
    Error::ForgeApi(msg)
}

#[derive(Debug, Clone)]
pub enum ForgeKind {
    GitHub,
    GitLab,
    Codeberg,
    Gitea { instance: String },
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

    pub fn set_tag(&mut self, tag: String) {
        self.tag = tag;
        self.resolved_url = OnceCell::new();
    }

    pub fn zsync_url(&self) -> Result<&str> {
        if self.resolved_url.get().is_none() {
            let url = self.resolve_url()?;
            let _ = self.resolved_url.set(url);
        }
        Ok(self.resolved_url.get().unwrap())
    }

    pub fn list_releases(&self) -> Result<Vec<Release>> {
        let project = format!("{}/{}", self.owner, self.repo);
        self.with_forge(|forge| {
            forge
                .fetch_releases(&project, None)
                .map_err(|e| forge_error(e, &project))
        })
    }

    fn resolve_url(&self) -> Result<String> {
        self.with_forge(|forge| self.fetch_with_forge(forge))
    }

    fn with_forge<T>(&self, action: impl Fn(&dyn Forge) -> Result<T>) -> Result<T> {
        match &self.kind {
            ForgeKind::GitHub => self.with_forge_proxied(
                config::get_github_proxies(),
                |base| {
                    GitHub::new(UreqClient)
                        .with_base_url(base)
                        .with_token_from_env(&["GITHUB_TOKEN", "GH_TOKEN"])
                },
                || GitHub::new(UreqClient).with_token_from_env(&["GITHUB_TOKEN", "GH_TOKEN"]),
                &action,
            ),
            ForgeKind::GitLab => self.with_forge_proxied(
                config::get_gitlab_proxies(),
                |base| {
                    GitLab::new(UreqClient)
                        .with_base_url(base)
                        .with_token_from_env(&["GITLAB_TOKEN", "GL_TOKEN"])
                },
                || GitLab::new(UreqClient).with_token_from_env(&["GITLAB_TOKEN", "GL_TOKEN"]),
                &action,
            ),
            ForgeKind::Codeberg => self.with_forge_proxied(
                config::get_codeberg_proxies(),
                |base| Gitea::new(UreqClient, base).with_token_from_env(&["CODEBERG_TOKEN"]),
                || {
                    Gitea::new(UreqClient, "https://codeberg.org")
                        .with_token_from_env(&["CODEBERG_TOKEN"])
                },
                &action,
            ),
            ForgeKind::Gitea { instance } => {
                let base_url =
                    if instance.starts_with("http://") || instance.starts_with("https://") {
                        instance.to_string()
                    } else {
                        format!("https://{instance}")
                    };
                let gt = Gitea::new(UreqClient, base_url)
                    .with_token_from_env(&["GITEA_TOKEN", "FORGEJO_TOKEN"]);
                action(&gt)
            }
        }
    }

    fn with_forge_proxied<F: Forge, T>(
        &self,
        proxies: Vec<String>,
        make_proxy: impl Fn(&str) -> F,
        make_default: impl FnOnce() -> F,
        action: &impl Fn(&dyn Forge) -> Result<T>,
    ) -> Result<T> {
        if proxies.is_empty() {
            let forge = make_default();
            return action(&forge);
        }

        let mut last_error = None;
        for proxy in &proxies {
            let forge = make_proxy(proxy);
            match action(&forge) {
                Ok(v) => return Ok(v),
                Err(e) => last_error = Some(e),
            }
        }

        Err(last_error.unwrap_or_else(|| Error::ForgeApi("All proxies failed".into())))
    }

    fn fetch_with_forge(&self, forge: &dyn Forge) -> Result<String> {
        let project = format!("{}/{}", self.owner, self.repo);

        let (tag, find_prerelease) = match self.tag.as_str() {
            "latest" | "latest-pre" | "latest-all" => (None, self.tag == "latest-pre"),
            tag => (Some(tag), false),
        };

        let releases = forge
            .fetch_releases(&project, tag)
            .map_err(|e| forge_error(e, &project))?;

        let release = self.select_release(releases, tag.is_some(), find_prerelease)?;

        self.find_matching_asset(&release)
    }

    fn select_release(
        &self,
        releases: Vec<Release>,
        is_specific_tag: bool,
        find_prerelease: bool,
    ) -> Result<Release> {
        let project = format!("{}/{}", self.owner, self.repo);

        if is_specific_tag {
            return releases.into_iter().next().ok_or_else(|| {
                Error::ForgeApi(format!(
                    "no release found for tag '{}' in {project}",
                    self.tag
                ))
            });
        }

        if find_prerelease {
            return releases
                .into_iter()
                .find(|r| r.is_prerelease())
                .ok_or_else(|| Error::ForgeApi(format!("no prerelease found for {project}")));
        }

        if self.tag == "latest" {
            return releases
                .into_iter()
                .find(|r| !r.is_prerelease())
                .ok_or_else(|| Error::ForgeApi(format!("no stable release found for {project}")));
        }

        // latest-all: first release regardless
        releases
            .into_iter()
            .next()
            .ok_or_else(|| Error::ForgeApi(format!("no releases found for {project}")))
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
                "no asset matching '*{}' in release {} of {}/{}",
                self.filename,
                release.tag(),
                self.owner,
                self.repo
            )));
        }

        matching.sort();
        matching.reverse();

        Ok(matching[0].to_string())
    }
}
