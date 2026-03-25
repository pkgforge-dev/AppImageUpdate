# AppImageUpdate
<img src="https://img.shields.io/github/downloads/pkgforge-dev/AppImageUpdate/total" alt="GitHub Downloads (all assets, all releases)"> <img src="https://img.shields.io/github/v/release/pkgforge-dev/AppImageUpdate" alt="GitHub Release"> <img src="https://img.shields.io/github/release-date/pkgforge-dev/AppImageUpdate" alt="GitHub Release Date">

A Rust implementation of AppImageUpdate - a tool for updating AppImages using efficient delta updates.

## Features

- **Delta Updates** - Only download the changed portions of the AppImage, saving bandwidth and time
- **Decentralized** - No central repository required; updates come directly from the source
- **Multiple AppImages** - Update or check multiple AppImages at once
- **Directory Scanning** - Pass a directory to update all AppImages inside
- **Smart Grouping** - AppImages with the same update source share downloads
- **Progress Display** - Real-time progress during updates
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

Output:
```
Source:   ./myapp.AppImage (85.3 MB)
Target:   ./myapp-2.0.AppImage (92.1 MB)

Performing delta update...
Progress: 100% (92.1/92.1 MB)

Reused:        42.1 MB  (1247 blocks)
Downloaded:    50.0 MB  (1482 blocks)
Saved:         42.1 MB  (45%)

Updated: ./myapp-2.0.AppImage
```

### Update Multiple AppImages

```bash
appimageupdate ./app1.AppImage ./app2.AppImage ./app3.AppImage
```

### Update All AppImages in a Directory

```bash
appimageupdate ~/Applications/
```

### Check for Updates

Check a single AppImage:
```bash
appimageupdate -j ./myapp.AppImage
```

Check multiple AppImages:
```bash
appimageupdate -j ~/Applications/
```

Output:
```
Checking: /home/user/Applications/app1.AppImage ... Update available
Checking: /home/user/Applications/app2.AppImage ... Up to date
Checking: /home/user/Applications/app3.AppImage ... Update available
```

Exit code 1 if any update available, 0 if all up to date.

### Describe an AppImage

```bash
appimageupdate -d ./myapp.AppImage
```

Output:
```
Path:         ./myapp.AppImage
Size:         85.3 MB
Target:       ./myapp-2.0.AppImage
Target Size:  92.1 MB
Update Info:  gh-releases-zsync|user|repo|latest|*.AppImage
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
      --github-api-proxy <URL>  GitHub API proxy URL [env: GITHUB_API_PROXY]
                           (supports comma-separated list for fallback)
  -h, --help              Print help
  -V, --version           Print version
```

## How It Works

1. **Reads Update Information** - Extracts embedded update info from the AppImage's `.upd-info` ELF section
2. **Fetches Zsync Metadata** - Downloads the `.zsync` control file which contains block checksums and file metadata
3. **Calculates Delta** - Compares local file blocks against remote blocks using rolling checksums
4. **Downloads Only Changes** - Fetches only the blocks that differ from the local copy
5. **Assembles & Verifies** - Reconstructs the file and verifies SHA1 checksum

## Configuration

Create a config file at `~/.config/appimageupdate/config.toml`:

```toml
# GitHub API proxy (supports single or multiple for fallback)
github_api_proxy = "https://ghproxy.net"
# Or multiple proxies:
# github_api_proxy = ["https://ghproxy.net", "https://mirror.example.com"]

# Remove old AppImage after successful update
remove_old = true

# Output directory for updated AppImages (supports shell expansion)
output_dir = "~/Applications"
```

### Environment Variables

| Variable | Description |
|----------|-------------|
| `GITHUB_API_PROXY` | GitHub API proxy URL (comma-separated for multiple) |
| `APPIMAGEUPDATE_REMOVE_OLD` | Set to `true` to remove old AppImage after update |
| `APPIMAGEUPDATE_OUTPUT_DIR` | Output directory for updated AppImages |

**Priority:** CLI args > Environment variables > Config file

## Supported Update Formats

- `zsync|<url>` - Direct zsync URL
- `gh-releases-zsync|<user>|<repo>|<tag>|<filename>` - GitHub releases with glob pattern matching

## Comparison with Upstream

This is a Rust rewrite of the upstream [AppImageUpdate](https://github.com/AppImage/AppImageUpdate) (C++/Qt).

Advantages:
- Single static binary with no runtime dependencies
- Over 10x smaller
- Cleaner, more maintainable codebase
- URL caching to avoid redundant API calls
- Proxy support
- Better error messages

Differences:
- No GPG signature verification (not implemented)
- No pling integration (not implemented)
- No GUI (not implemented)

## Library Usage

```rust
use appimageupdate::{Updater, Error};

fn main() -> Result<(), Error> {
    let updater = Updater::new("./myapp.AppImage")?;

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
