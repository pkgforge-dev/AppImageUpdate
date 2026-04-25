use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;

use releasekit::Forge;
use releasekit::client::UreqClient;
use releasekit::platform::GitHub;

use crate::error::{Error, Result};

const REPO: &str = "pkgforge-dev/appimageupdate";

pub fn run() -> Result<()> {
    let arch = detect_arch()?;
    let asset_name = format!("appimageupdate-{}-linux", arch);

    eprintln!("Checking latest release of {}...", REPO);
    let github = GitHub::new(UreqClient).with_token_from_env(&["GITHUB_TOKEN", "GH_TOKEN"]);
    let releases = github
        .fetch_releases(REPO, None)
        .map_err(|e| Error::ForgeApi(format!("Failed to fetch releases: {}", e)))?;

    let release = releases
        .into_iter()
        .find(|r| !r.is_prerelease())
        .ok_or_else(|| Error::ForgeApi("No stable release found".into()))?;

    let latest = release.tag().trim_start_matches('v').to_string();
    let current = env!("CARGO_PKG_VERSION")
        .trim_start_matches('v')
        .to_string();

    if latest == current {
        eprintln!("Already up to date (v{}).", current);
        return Ok(());
    }

    eprintln!("Updating: v{} -> v{}", current, latest);

    let asset = release
        .assets()
        .iter()
        .find(|a| a.name() == asset_name)
        .ok_or_else(|| {
            Error::ForgeApi(format!(
                "No asset '{}' in release {}",
                asset_name,
                release.tag()
            ))
        })?;
    let b3sum_asset = release
        .assets()
        .iter()
        .find(|a| a.name() == format!("{}.b3sum", asset_name));

    let current_exe = std::env::current_exe()
        .map_err(|e| Error::AppImage(format!("Failed to determine current executable: {}", e)))?;

    eprintln!(
        "Downloading {} ({})...",
        asset.name(),
        format_size(asset.size())
    );
    let bin_data = download(asset.url())?;

    if let Some(b3) = b3sum_asset {
        let b3_data = download(b3.url())?;
        let expected = String::from_utf8_lossy(&b3_data)
            .split_whitespace()
            .next()
            .unwrap_or("")
            .to_ascii_lowercase();
        if expected.is_empty() {
            return Err(Error::AppImage("Empty .b3sum file".into()));
        }
        let actual = blake3::hash(&bin_data).to_hex().to_string();
        if actual != expected {
            return Err(Error::AppImage(format!(
                "blake3 mismatch: expected {}, got {}",
                expected, actual
            )));
        }
        eprintln!("Checksum verified.");
    } else {
        eprintln!("Warning: no .b3sum asset; skipping checksum verification.");
    }

    install(&current_exe, &bin_data)?;
    eprintln!("Updated to v{} at {}", latest, current_exe.display());
    Ok(())
}

fn install(current_exe: &Path, bin_data: &[u8]) -> Result<()> {
    let parent = current_exe.parent().ok_or_else(|| {
        Error::AppImage(format!(
            "Current executable has no parent dir: {}",
            current_exe.display()
        ))
    })?;
    let filename = current_exe
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("appimageupdate");
    let tmp = parent.join(format!(".{}.new", filename));

    let _ = fs::remove_file(&tmp);
    fs::write(&tmp, bin_data)?;

    let mode = fs::metadata(current_exe)
        .map(|m| m.permissions().mode())
        .unwrap_or(0o755)
        | 0o111;
    fs::set_permissions(&tmp, fs::Permissions::from_mode(mode))?;

    if let Err(e) = fs::rename(&tmp, current_exe) {
        let _ = fs::remove_file(&tmp);
        return Err(Error::Io(e));
    }
    Ok(())
}

fn detect_arch() -> Result<&'static str> {
    if cfg!(not(target_os = "linux")) {
        return Err(Error::AppImage(
            "--self-update only supports Linux binary releases".into(),
        ));
    }
    if cfg!(target_arch = "x86_64") {
        Ok("x86_64")
    } else if cfg!(target_arch = "aarch64") {
        Ok("aarch64")
    } else {
        Err(Error::AppImage(
            "--self-update only supports x86_64 and aarch64".into(),
        ))
    }
}

fn download(url: &str) -> Result<Vec<u8>> {
    let resp = ureq::get(url)
        .header("User-Agent", "appimageupdate")
        .call()
        .map_err(|e| Error::Http(format!("GET {} failed: {}", url, e)))?;
    resp.into_body()
        .read_to_vec()
        .map_err(|e| Error::Http(format!("read {}: {}", url, e)))
}

fn format_size(size: Option<u64>) -> String {
    match size {
        Some(s) => crate::util::format_size(s),
        None => "unknown size".into(),
    }
}
