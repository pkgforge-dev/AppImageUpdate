use crate::error::{Error, Result};

use super::{GenericUpdateInfo, GitHubUpdateInfo, UpdateInfoInner};

pub fn parse(s: &str) -> Result<UpdateInfoInner> {
    let parts: Vec<&str> = s.split('|').collect();

    if parts.is_empty() {
        return Err(Error::InvalidUpdateInfo("Empty update information".into()));
    }

    match parts[0] {
        "zsync" => {
            if parts.len() != 2 {
                return Err(Error::InvalidUpdateInfo(
                    "zsync format requires exactly 1 parameter: zsync|<url>".into(),
                ));
            }
            Ok(UpdateInfoInner::Generic(GenericUpdateInfo {
                url: parts[1].into(),
            }))
        }
        "gh-releases-zsync" => {
            if parts.len() != 5 {
                return Err(Error::InvalidUpdateInfo(
                    "gh-releases-zsync format requires 4 parameters: gh-releases-zsync|<username>|<repo>|<tag>|<filename>".into(),
                ));
            }
            Ok(UpdateInfoInner::GitHub(GitHubUpdateInfo::new(
                parts[1].into(),
                parts[2].into(),
                parts[3].into(),
                parts[4].into(),
            )))
        }
        _ => Err(Error::InvalidUpdateInfo(format!(
            "Unknown update information type: {}",
            parts[0]
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_generic_zsync() {
        let info = parse("zsync|https://example.com/app.AppImage.zsync").unwrap();
        match info {
            UpdateInfoInner::Generic(g) => {
                assert_eq!(g.url, "https://example.com/app.AppImage.zsync");
            }
            _ => panic!("Expected Generic variant"),
        }
    }

    #[test]
    fn parse_github_releases() {
        let info = parse("gh-releases-zsync|user|repo|latest|app-*.AppImage").unwrap();
        match info {
            UpdateInfoInner::GitHub(g) => {
                assert_eq!(g.username, "user");
                assert_eq!(g.repo, "repo");
                assert_eq!(g.tag, "latest");
                assert_eq!(g.filename, "app-*.AppImage");
            }
            _ => panic!("Expected GitHub variant"),
        }
    }

    #[test]
    fn parse_invalid_type() {
        assert!(parse("invalid|params").is_err());
    }

    #[test]
    fn parse_missing_params() {
        assert!(parse("zsync").is_err());
        assert!(parse("gh-releases-zsync|user|repo").is_err());
    }
}
