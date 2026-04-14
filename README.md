# AppImageUpdate
<img src="https://img.shields.io/github/downloads/pkgforge-dev/AppImageUpdate/total" alt="GitHub Downloads (all assets, all releases)"> <img src="https://img.shields.io/github/v/release/pkgforge-dev/AppImageUpdate" alt="GitHub Release"> <img src="https://img.shields.io/github/release-date/pkgforge-dev/AppImageUpdate" alt="GitHub Release Date">

A Rust implementation of AppImageUpdate - a tool for updating AppImages using efficient delta updates.

## Features

- **Delta Updates** - Only download the changed portions of the AppImage, saving bandwidth and time
- **Parallel Updates** - Update multiple AppImages concurrently with per-URL deduplication
- **Multi-Forge Support** - Fetch releases from GitHub, GitLab, Codeberg, and Gitea instances
- **Decentralized** - No central repository required; updates come directly from the source
- **Directory Scanning** - Pass a directory to update all AppImages inside
- **Progress Bars** - Real-time progress during downloads with per-file bars
- **Checksum Verification** - SHA1 verification ensures downloaded files are valid
- **Permission Preservation** - Maintains executable permissions from the original AppImage
- **Skip Unnecessary Updates** - Automatically skips update if the target file already exists with the correct checksum
- **In-Place Updates** - Updates to same filename preserve old version as `.old` backup

## Installation

### From Source

```bash
git clone https://github.com/pkgforge-dev/appimageupdate.git
cd appimageupdate
cargo install --path .
```

## Usage

### Update an AppImage

```bash
appimageupdate ./myapp.AppImage
```

### Update Multiple AppImages in Parallel

```bash
appimageupdate ./app1.AppImage ./app2.AppImage ./app3.AppImage
```

AppImages are updated concurrently. When multiple AppImages share the same update source, the download happens only once and the result is copied to the rest.

Control parallelism with `-J`:
```bash
appimageupdate -J 4 ~/Applications/
```

Use `0` (default) to auto-detect based on CPU count.

### Update All AppImages in a Directory

```bash
appimageupdate ~/Applications/
```

### Check for Updates

```bash
appimageupdate -j ./myapp.AppImage
```

Check multiple AppImages in parallel:
```bash
appimageupdate -j ~/Applications/
```

Exit code 1 if any update available, 0 if all up to date.

### Describe an AppImage

```bash
appimageupdate -d ./myapp.AppImage
```

### Options

```
appimageupdate [OPTIONS] [APPIMAGE]...

Arguments:
  [APPIMAGE]...           Path(s) to AppImage(s) or directories to update

Options:
  -O, --overwrite         Overwrite existing target file
  -r, --remove-old        Remove old AppImage after successful update
  -u, --update-info <INFO> Override update information in the AppImage
      --output-dir <DIR>  Output directory for updated AppImages
  -d, --describe          Parse and describe AppImage and its update information
  -j, --check-for-update  Check for update (exit 1 if any available, 0 if not)
  -l, --list-releases     List available releases from the update source
  -t, --target-tag <TAG>  Install a specific version (e.g., for downgrade)
  -J, --jobs <N>          Number of parallel jobs (default: 0 = auto-detect)
      --github-api-proxy <URL>    GitHub API proxy [env: GITHUB_API_PROXY]
      --gitlab-api-proxy <URL>    GitLab API proxy [env: GITLAB_API_PROXY]
      --codeberg-api-proxy <URL>  Codeberg API proxy [env: CODEBERG_API_PROXY]
  -h, --help              Print help
  -V, --version           Print version
```

## How It Works

1. **Reads Update Information** - Extracts embedded update info from the AppImage's `.upd-info` ELF section
2. **Fetches Zsync Metadata** - Downloads the `.zsync` control file which contains block checksums and file metadata
3. **Calculates Delta** - Compares local file blocks against remote blocks using rolling checksums
4. **Downloads Only Changes** - Fetches only the blocks that differ from the local copy
5. **Assembles & Verifies** - Reconstructs the file and verifies SHA1 checksum

## Supported Update Formats

| Format | Description |
|--------|-------------|
| `zsync\|<url>` | Direct zsync URL |
| `gh-releases-zsync\|<owner>\|<repo>\|<tag>\|<filename>` | GitHub releases |
| `gl-releases-zsync\|<owner>\|<repo>\|<tag>\|<filename>` | GitLab releases |
| `cb-releases-zsync\|<owner>\|<repo>\|<tag>\|<filename>` | Codeberg releases |
| `gitea-releases-zsync\|<instance>\|<owner>\|<repo>\|<tag>\|<filename>` | Gitea releases |
| `forgejo-releases-zsync\|<instance>\|<owner>\|<repo>\|<tag>\|<filename>` | Forgejo releases (alias for Gitea) |

The `<tag>` field supports special values:
- `latest` - Latest stable (non-prerelease) release
- `latest-pre` - Latest prerelease
- `latest-all` - Latest release regardless of prerelease status
- Any other value is treated as a specific tag name

The `<filename>` field supports glob patterns (e.g., `*x86_64.AppImage`).

## Configuration

Create a config file at `~/.config/appimageupdate/config.toml`:

```toml
# API proxies (supports single string or array for fallback)
github_api_proxy = "https://ghproxy.net"
# gitlab_api_proxy = "https://glproxy.example.com"
# codeberg_api_proxy = "https://cbproxy.example.com"

# Or multiple proxies for fallback:
# github_api_proxy = ["https://ghproxy.net", "https://mirror.example.com"]

# Remove old AppImage after successful update
remove_old = true

# Output directory for updated AppImages (supports shell expansion)
output_dir = "~/Applications"
```

### Environment Variables

| Variable | Description |
|----------|-------------|
| `GITHUB_TOKEN` / `GH_TOKEN` | GitHub authentication token |
| `GITLAB_TOKEN` / `GL_TOKEN` | GitLab authentication token |
| `CODEBERG_TOKEN` | Codeberg authentication token |
| `GITEA_TOKEN` / `FORGEJO_TOKEN` | Gitea/Forgejo authentication token |
| `GITHUB_API_PROXY` | GitHub API proxy URL (comma-separated for multiple) |
| `GITLAB_API_PROXY` | GitLab API proxy URL (comma-separated for multiple) |
| `CODEBERG_API_PROXY` | Codeberg API proxy URL (comma-separated for multiple) |
| `APPIMAGEUPDATE_REMOVE_OLD` | Set to `true` to remove old AppImage after update |
| `APPIMAGEUPDATE_OUTPUT_DIR` | Output directory for updated AppImages |

**Priority:** CLI args > Environment variables > Config file

## Comparison with Upstream

This is a Rust rewrite of the upstream [AppImageUpdate](https://github.com/AppImage/AppImageUpdate) (C++/Qt).

Advantages:
- Single static binary with no runtime dependencies
- Over 10x smaller
- Multi-forge support (GitHub, GitLab, Codeberg, Gitea/Forgejo)
- Cleaner, more maintainable codebase
- URL caching to avoid redundant API calls
- Proxy support for all forges
- Better error messages

Differences:
- No GPG signature verification (not implemented)
- No pling integration (not implemented)
- No GUI (not implemented)

## Library Usage

```rust
use appimageupdate::{Updater, Error};

fn main() -> Result<(), Error> {
    let updater = Updater::new("./myapp.AppImage")?
        .progress_callback(|done, total| {
            if total > 0 {
                eprintln!("Progress: {}/{}", done, total);
            }
        });

    if updater.check_for_update()? {
        let (path, stats) = updater.perform_update()?;
        println!("Updated to: {}", path.display());
        println!("Saved {}% bandwidth", stats.saved_percentage());
    } else {
        println!("Already up to date!");
    }

    Ok(())
}
```
