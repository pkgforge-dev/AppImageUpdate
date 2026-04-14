use crate::error::{Error, Result};

use super::forge::{ForgeKind, ForgeUpdateInfo};
use super::{GenericUpdateInfo, UpdateInfoInner};

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
        "gh-releases-zsync" => parse_forge(ForgeKind::GitHub, &parts, "gh-releases-zsync"),
        "gl-releases-zsync" => parse_forge(ForgeKind::GitLab, &parts, "gl-releases-zsync"),
        _ => Err(Error::InvalidUpdateInfo(format!(
            "Unknown update information type: {}",
            parts[0]
        ))),
    }
}

fn parse_forge(kind: ForgeKind, parts: &[&str], prefix: &str) -> Result<UpdateInfoInner> {
    if parts.len() != 5 {
        return Err(Error::InvalidUpdateInfo(format!(
            "{prefix} format requires 4 parameters: {prefix}|<owner>|<repo>|<tag>|<filename>"
        )));
    }
    Ok(UpdateInfoInner::Forge(ForgeUpdateInfo::new(
        kind,
        parts[1].into(),
        parts[2].into(),
        parts[3].into(),
        parts[4].into(),
    )))
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
            UpdateInfoInner::Forge(f) => {
                assert!(matches!(f.kind, ForgeKind::GitHub));
                assert_eq!(f.owner, "user");
                assert_eq!(f.repo, "repo");
                assert_eq!(f.tag, "latest");
                assert_eq!(f.filename, "app-*.AppImage");
            }
            _ => panic!("Expected Forge variant"),
        }
    }

    #[test]
    fn parse_gitlab_releases() {
        let info = parse("gl-releases-zsync|owner|repo|latest|app-*.AppImage").unwrap();
        match info {
            UpdateInfoInner::Forge(f) => {
                assert!(matches!(f.kind, ForgeKind::GitLab));
                assert_eq!(f.owner, "owner");
                assert_eq!(f.repo, "repo");
                assert_eq!(f.tag, "latest");
                assert_eq!(f.filename, "app-*.AppImage");
            }
            _ => panic!("Expected Forge variant"),
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
        assert!(parse("gl-releases-zsync|user|repo").is_err());
    }
}
