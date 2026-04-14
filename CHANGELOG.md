
## [0.3.1](https://github.com/pkgforge-dev/AppImageUpdate/compare/0.3.0...0.3.1) - 2026-04-14

### 🐛 Bug Fixes

- Improve error messages with project context - ([9bfdfd9](https://github.com/pkgforge-dev/AppImageUpdate/commit/9bfdfd9ce55caed710f18c7c8afc4ff5a4eb308d))
- Handle full URL in Gitea/Forgejo instance field - ([0683fbf](https://github.com/pkgforge-dev/AppImageUpdate/commit/0683fbf1deee5623c3f7cffef2dce20f6bc17430))

## [0.3.0](https://github.com/pkgforge-dev/AppImageUpdate/compare/0.2.1...0.3.0) - 2026-04-14

### ⛰️  Features

- Add Forgejo support as alias for Gitea - ([886efa2](https://github.com/pkgforge-dev/AppImageUpdate/commit/886efa245c22d61a668a748b8f8df0c8c3ad3e21))
- Add --list-releases to show available versions - ([78d3b89](https://github.com/pkgforge-dev/AppImageUpdate/commit/78d3b892740b38043f34dada05dd72dcd51713f3))
- Add --target-tag for version targeting and downgrade - ([513e522](https://github.com/pkgforge-dev/AppImageUpdate/commit/513e522aefabe2609442e2f0bb69c7b42f116d1f))
- Add API proxy support for GitLab and Codeberg - ([0e802af](https://github.com/pkgforge-dev/AppImageUpdate/commit/0e802afbcdf968fbde292b6dec41bcba5f2d11bd))
- Add Codeberg and Gitea update info support - ([5d85807](https://github.com/pkgforge-dev/AppImageUpdate/commit/5d85807198cd123461c33649a5dac50ddbda2783))
- Add GitLab update info support - ([c20dc30](https://github.com/pkgforge-dev/AppImageUpdate/commit/c20dc30fb618b0303cd3691bad3388d7de51087d))
- Parallel AppImage updates with progress bars - ([9d0da26](https://github.com/pkgforge-dev/AppImageUpdate/commit/9d0da26d808157a9630f50d8d47c6cb3e7127bc3))

### 🚜 Refactor

- Use releasekit for GitHub release fetching - ([ad1465c](https://github.com/pkgforge-dev/AppImageUpdate/commit/ad1465c4bc64e636f4372d20a38eff5a512ef472))

### 📚 Documentation

- Update README - ([e801527](https://github.com/pkgforge-dev/AppImageUpdate/commit/e801527310c2047e3f9283e64bf1485c36317495))
- Update README with multi-forge support - ([52c02f8](https://github.com/pkgforge-dev/AppImageUpdate/commit/52c02f8291eb7324a42c1b14f23bb9fe61c1dde3))
- Update README - ([029a364](https://github.com/pkgforge-dev/AppImageUpdate/commit/029a364a7489dc4b54dc4ea7f56797148e757cb7))

## [0.2.1](https://github.com/pkgforge-dev/AppImageUpdate/compare/0.2.0...0.2.1) - 2026-03-25

### ⛰️  Features

- Support checking updates for multiple AppImages - ([a50f175](https://github.com/pkgforge-dev/AppImageUpdate/commit/a50f175c8b87163372b2533e8df1d5d7e6affc31))
- Add `GITHUB_TOKEN` support ([#9](https://github.com/pkgforge-dev/AppImageUpdate/pull/9)) - ([e9247f6](https://github.com/pkgforge-dev/AppImageUpdate/commit/e9247f6d048f19604568768fd54448fa0dcc4673))

## [0.2.0](https://github.com/pkgforge-dev/AppImageUpdate/compare/0.1.1...0.2.0) - 2026-03-24

### ⛰️  Features

- Add progress display during updates - ([90233e1](https://github.com/pkgforge-dev/AppImageUpdate/commit/90233e1b05d2e68a998cec4a383f992b453496d1))
- Add support for updating multiple AppImages at once - ([c59368d](https://github.com/pkgforge-dev/AppImageUpdate/commit/c59368d4e81df1e31c0a3be57d42bcdb07baa1ae))
- Add global and portable config file support ([#5](https://github.com/pkgforge-dev/AppImageUpdate/pull/5)) - ([2db70c8](https://github.com/pkgforge-dev/AppImageUpdate/commit/2db70c8b35eeb950ee3aa87860d0eb67db0f8466))

### 🐛 Bug Fixes

- Handle backup file removal - ([73d4ba5](https://github.com/pkgforge-dev/AppImageUpdate/commit/73d4ba5cf6d711418cbafca744dcb002c01b6489))

### 📚 Documentation

- Update README ([#4](https://github.com/pkgforge-dev/AppImageUpdate/pull/4)) - ([0c72d84](https://github.com/pkgforge-dev/AppImageUpdate/commit/0c72d84d3b7a18c2f759204ae7a5241d2778f9f5))

## [0.1.1](https://github.com/pkgforge-dev/AppImageUpdate/compare/0.1.0...0.1.1) - 2026-03-23

### 🐛 Bug Fixes

- Compare SHA1 of source file in check_for_update - ([6f46a5b](https://github.com/pkgforge-dev/AppImageUpdate/commit/6f46a5bf4bbc68dd93a2d4a3c3d637231d1f8102))
- Make -O flag overwrite source file in-place - ([6644b33](https://github.com/pkgforge-dev/AppImageUpdate/commit/6644b3306c22a4ce40bc1096cf2a07c2906752f7))

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
