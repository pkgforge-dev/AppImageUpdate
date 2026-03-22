use std::cell::OnceCell;

use serde::Deserialize;

use crate::error::{Error, Result};

#[derive(Debug, Clone)]
pub struct GitHubUpdateInfo {
    pub username: String,
    pub repo: String,
    pub tag: String,
    pub filename: String,
    resolved_url: OnceCell<String>,
}

#[derive(Debug, Clone, Deserialize)]
struct GitHubRelease {
    assets: Vec<GitHubAsset>,
    #[serde(default)]
    prerelease: bool,
}

#[derive(Debug, Clone, Deserialize)]
struct GitHubAsset {
    name: String,
    browser_download_url: String,
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
        let api_url = match self.tag.as_str() {
            "latest" => format!(
                "https://api.github.com/repos/{}/{}/releases/latest",
                self.username, self.repo
            ),
            "latest-pre" | "latest-all" => format!(
                "https://api.github.com/repos/{}/{}/releases",
                self.username, self.repo
            ),
            tag => format!(
                "https://api.github.com/repos/{}/{}/releases/tags/{}",
                self.username, self.repo, tag
            ),
        };

        let response = ureq::get(&api_url)
            .header("User-Agent", "appimageupdate-rs")
            .call()
            .map_err(|e| Error::Http(format!("GitHub API request failed: {}", e)))?;

        if !response.status().is_success() {
            return Err(Error::GitHubApi(format!(
                "GitHub API returned status {}",
                response.status()
            )));
        }

        let release: GitHubRelease = serde_json::from_reader(response.into_body().into_reader())
            .map_err(|e| Error::GitHubApi(format!("Failed to parse GitHub response: {}", e)))?;

        let release = if self.tag == "latest-pre" || self.tag == "latest-all" {
            let releases: Vec<GitHubRelease> = vec![release];
            Self::find_suitable_release(&releases, self.tag == "latest-pre")?
        } else {
            release
        };

        let asset_url = Self::find_matching_asset(&release, &self.filename)?;

        Ok(format!("{}.zsync", asset_url))
    }

    fn find_suitable_release(
        releases: &[GitHubRelease],
        prereleases_only: bool,
    ) -> Result<GitHubRelease> {
        if prereleases_only {
            releases
                .iter()
                .find(|r| r.prerelease)
                .cloned()
                .ok_or_else(|| Error::GitHubApi("No prerelease found".into()))
        } else {
            releases
                .first()
                .cloned()
                .ok_or_else(|| Error::GitHubApi("No release found".into()))
        }
    }

    fn find_matching_asset(release: &GitHubRelease, filename_pattern: &str) -> Result<String> {
        let pattern = format!("*{}", filename_pattern);

        let mut matching_urls: Vec<String> = release
            .assets
            .iter()
            .filter(|asset| Self::glob_match(&pattern, &asset.name))
            .map(|asset| asset.browser_download_url.clone())
            .collect();

        if matching_urls.is_empty() {
            return Err(Error::GitHubApi(format!(
                "No asset matched pattern: {}",
                pattern
            )));
        }

        matching_urls.sort();
        matching_urls.reverse();

        Ok(matching_urls.remove(0))
    }

    fn glob_match(pattern: &str, text: &str) -> bool {
        let pattern_chars: Vec<char> = pattern.chars().collect();
        let text_chars: Vec<char> = text.chars().collect();
        Self::glob_match_recursive(&pattern_chars, &text_chars)
    }

    fn glob_match_recursive(pattern: &[char], text: &[char]) -> bool {
        match (pattern.first(), text.first()) {
            (None, None) => true,
            (Some('*'), _) => {
                Self::glob_match_recursive(pattern, &text[1..])
                    || Self::glob_match_recursive(&pattern[1..], text)
            }
            (Some(p), Some(t)) if *p == *t => Self::glob_match_recursive(&pattern[1..], &text[1..]),
            (Some('?'), Some(_)) => Self::glob_match_recursive(&pattern[1..], &text[1..]),
            _ => false,
        }
    }
}
