
## [0.1.0] - 2026-03-23

### ⛰️  Features

- Add TOML config file support - ([853712c](https://github.com/pkgforge-dev/AppImageUpdate/commit/853712c611e7527f22cb8aa65bf519cb6640d863))
- Make CLI flags compatible with upstream appimageupdate - ([9aa74fc](https://github.com/pkgforge-dev/AppImageUpdate/commit/9aa74fcd8feccc395f243ed0a9080219d43fa139))
- Add GitHub API proxy support with fallback - ([ac5911e](https://github.com/pkgforge-dev/AppImageUpdate/commit/ac5911ecaee53fa1fc6c847d88e0291f3d60f0d8))
- Add detailed update statistics and improve CLI output - ([b69cdee](https://github.com/pkgforge-dev/AppImageUpdate/commit/b69cdeea094a55fa84b4549098bdab023fb152c4))
- Skip update if target file already exists with correct checksum - ([4653f6f](https://github.com/pkgforge-dev/AppImageUpdate/commit/4653f6f11487a3b274f3298810ac971811198708))
- Preserve original AppImage file permissions on update - ([3a03297](https://github.com/pkgforge-dev/AppImageUpdate/commit/3a03297a8a88183aa6dc4ada2ba7973f1fb65c5a))
- Implement CLI with update and check commands - ([02c12c9](https://github.com/pkgforge-dev/AppImageUpdate/commit/02c12c9d9679fe5542d20235e3534a38830ec982))
- Implement update logic with zsync integration - ([432e324](https://github.com/pkgforge-dev/AppImageUpdate/commit/432e32442f2cb71e41cfc17ecf38b4ce09b4ef68))
- Add path() accessor to AppImage - ([765bed1](https://github.com/pkgforge-dev/AppImageUpdate/commit/765bed1f5f562297810c7fc8098b21b504f1e294))
- Add zsync_url method to UpdateInfo - ([4ae844f](https://github.com/pkgforge-dev/AppImageUpdate/commit/4ae844fba592aa9b9c3d114410b1abfa6c379d38))
- Add Zsync error variant - ([b069332](https://github.com/pkgforge-dev/AppImageUpdate/commit/b069332ed3f95c90a0dd90afe0c569764a56f29b))
- Implement GitHub releases URL resolution - ([1c75557](https://github.com/pkgforge-dev/AppImageUpdate/commit/1c75557cd99ad0dc57f5e48b5a0b79914315557f))
- Implement AppImage parsing with ELF section support - ([2ca4e75](https://github.com/pkgforge-dev/AppImageUpdate/commit/2ca4e75ea57759c88f469d29357df894104a717a))
- Add Updater struct with builder pattern - ([8675524](https://github.com/pkgforge-dev/AppImageUpdate/commit/86755249293135b582198989cfa256c9cb9c8f7d))
- Add AppImage file handling stub - ([51d52ce](https://github.com/pkgforge-dev/AppImageUpdate/commit/51d52ce2fb82c0e26a8feed43c758566a732e508))
- Add update information parser - ([a086bc5](https://github.com/pkgforge-dev/AppImageUpdate/commit/a086bc5aedbe62e740406ec0dd71892e53ce7b93))
- Add library structure with error types - ([08bc111](https://github.com/pkgforge-dev/AppImageUpdate/commit/08bc1115e01b3bfb4cf74de259da57121aceb408))

### 🐛 Bug Fixes

- Handle in-place updates by preserving old file as .old backup - ([a9d77bd](https://github.com/pkgforge-dev/AppImageUpdate/commit/a9d77bdc9e736e64e5b9696c10bfe6d2369ded31))
- Output updated AppImage in same directory as original - ([3a40f56](https://github.com/pkgforge-dev/AppImageUpdate/commit/3a40f5689241689cf9ca7c9965123a21ea57c5ca))
- Derive output path from zsync control file filename - ([24d4220](https://github.com/pkgforge-dev/AppImageUpdate/commit/24d42209f97d3560aae83a2c84fc9e88e49b7b20))
- Handle empty text in glob matching and remove .zsync suffix - ([1ce9d16](https://github.com/pkgforge-dev/AppImageUpdate/commit/1ce9d16f933d558c9707b18c0aa2777a10b8659b))

### 📚 Documentation

- Add config file documentation to README - ([3b61038](https://github.com/pkgforge-dev/AppImageUpdate/commit/3b61038efba0d2472bba71a77c41022889004866))
- Update README with new CLI interface - ([9a3e67f](https://github.com/pkgforge-dev/AppImageUpdate/commit/9a3e67f7d91c26b570d8bc9c3e3b5c0ea99da6ee))
- Add project README - ([a5fc0e1](https://github.com/pkgforge-dev/AppImageUpdate/commit/a5fc0e106c06a313e624c3f36f6022d99567bf45))
- Add AGENTS.md with project guidelines - ([40f9159](https://github.com/pkgforge-dev/AppImageUpdate/commit/40f915906a4fa1208092d0a207e5727aec6754ea))

### ⚙️ Miscellaneous Tasks

- Add release workflow - ([aed4971](https://github.com/pkgforge-dev/AppImageUpdate/commit/aed49718364585e8bf39f16e67e179c862539ec9))
- Update manifest - ([e016597](https://github.com/pkgforge-dev/AppImageUpdate/commit/e0165979763fa70b871b2143323405cca8ad29e3))
- Add license - ([841132e](https://github.com/pkgforge-dev/AppImageUpdate/commit/841132ec30244aa383ad72f87046619915da4c30))
- Add clap dependency with derive feature - ([b91360f](https://github.com/pkgforge-dev/AppImageUpdate/commit/b91360fcb1dd83648132e142befda9a7fb008cfa))
- Add zsync-rs as git dependency - ([9c6c5bd](https://github.com/pkgforge-dev/AppImageUpdate/commit/9c6c5bd12354b66b74f235d9dfcf0d03a02e86a5))
- Add ureq, serde, and serde_json dependencies - ([5ea196e](https://github.com/pkgforge-dev/AppImageUpdate/commit/5ea196ef18e0c68af9e42c986645c245bcda2c62))
- Add thiserror dependency for error handling - ([0c7d8aa](https://github.com/pkgforge-dev/AppImageUpdate/commit/0c7d8aab75577af1c55fa0b47bdf17a6b97a87a2))
- Initialize Rust project - ([7337f84](https://github.com/pkgforge-dev/AppImageUpdate/commit/7337f8474749e9ba83ad795b3c7e09fbeecd34fc))
