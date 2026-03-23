# appimageupdate

A Rust implementation of AppImageUpdate - a tool for updating AppImages using efficient delta updates.

## Features

- **Delta Updates** - Only download the changed portions of the AppImage, saving bandwidth and time
- **Decentralized** - No central repository required; updates come directly from the source
- **Checksum Verification** - SHA1 verification ensures downloaded files are valid
- **Permission Preservation** - Maintains executable permissions from the original AppImage
- **Skip Unnecessary Updates** - Automatically skips update if the target file already exists with the correct checksum
- **In-Place Updates** - Updates to same filename preserve old version as `.old` backup

## Installation

### From Source

```bash
git clone https://github.com/qaidvoid/appimageupdate.git
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

Reused:        42.1 MB  (1247 blocks)
Downloaded:    50.0 MB  (1482 blocks)
Saved:         42.1 MB  (45%)

Updated: ./myapp-2.0.AppImage
```

### Check for Updates

```bash
appimageupdate -j ./myapp.AppImage
```

Exit code 1 if update available, 0 if up to date.

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
appimageupdate [OPTIONS] [APPIMAGE]

Arguments:
  [APPIMAGE]              Path to the AppImage to update

Options:
  -O, --overwrite         Overwrite existing target file
  -r, --remove-old        Remove old AppImage after successful update
  -u, --update-info <INFO> Override update information in the AppImage
  -d, --describe          Parse and describe AppImage and its update information
  -j, --check-for-update  Check for update (exit 1 if available, 0 if not)
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

## Supported Update Formats

- `zsync|<url>` - Direct zsync URL
- `gh-releases-zsync|<user>|<repo>|<tag>|<filename>` - GitHub releases with glob pattern matching

## Comparison with Upstream

This is a Rust rewrite of the upstream [AppImageUpdate](https://github.com/AppImage/AppImageUpdate) (C++/Qt).

Advantages:
- Single binary with no runtime dependencies
- Cleaner, more maintainable codebase
- URL caching to avoid redundant API calls
- Better error messages

Differences:
- No GPG signature verification (not implemented)
- No pling integration (not implemented)

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
