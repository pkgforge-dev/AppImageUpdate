# appimageupdate

A Rust implementation of AppImageUpdate - a tool for updating AppImages using efficient delta updates.

## Features

- **Delta Updates** - Only download the changed portions of the AppImage, saving bandwidth and time
- **Decentralized** - No central repository required; updates come directly from the source
- **Checksum Verification** - SHA1 verification ensures downloaded files are valid
- **Permission Preservation** - Maintains executable permissions from the original AppImage
- **Skip Unnecessary Updates** - Automatically skips update if the target file already exists with the correct checksum

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
appimageupdate update ./myapp.AppImage
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
appimageupdate check ./myapp.AppImage
```

Output:
```
Source:   ./myapp.AppImage (85.3 MB)
Target:   ./myapp-2.0.AppImage (92.1 MB)

Status: Update available
```

### Options

```
appimageupdate update [OPTIONS] <APPIMAGE>

Arguments:
  <APPIMAGE>  Path to the AppImage to update

Options:
  -o, --output <DIR>   Output directory for the updated AppImage
  -w, --overwrite      Overwrite existing target file
  -h, --help           Print help
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
