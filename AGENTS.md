# AGENTS.md

Guidelines for AI agents working on the appimageupdate project.

## Project Overview

appimageupdate is a Rust implementation of AppImageUpdate - a tool for updating AppImages in a decentral way using delta updates. This project is a rewrite of the upstream C++ implementation at https://github.com/AppImage/AppImageUpdate.

Key features:
- Decentralized updates (no central repository)
- Delta updates for efficient downloads
- GPG signature validation
- Works with update information embedded in AppImages

## Development Workflow

### Commits

- **Commit in small chunks** - One logical change per commit
- **Never commit broken state** - All code must compile and pass tests
- **Format before commit** - Run `cargo fmt` before every commit
- **Fix clippy issues** - Run `cargo clippy` and address all warnings before committing

### Commit Messages

Follow conventional commit format with imperative mood:

```
type: message
```

Types:
- `feat:` - New feature
- `fix:` - Bug fix
- `refactor:` - Code refactoring
- `test:` - Adding or updating tests
- `docs:` - Documentation changes
- `chore:` - Maintenance tasks
- `perf:` - Performance improvements
- `style:` - Code style changes (formatting, etc.)
- `ci:` - CI/CD configuration changes

Examples:
- `feat: add AppImage update information parser`
- `fix: handle HTTP range requests correctly`
- `refactor: extract delta update logic into separate module`

## Code Quality

### Formatting

```bash
cargo fmt
```

Always run before committing.

### Linting

```bash
cargo clippy --all-targets --all-features -- -D warnings
```

All clippy warnings must be addressed before committing.

### Testing

```bash
cargo test --all-features
```

All tests must pass before committing.

## Pre-commit Checklist

Before every commit, ensure:

1. [ ] `cargo fmt` - Code is formatted
2. [ ] `cargo clippy` - No warnings
3. [ ] `cargo test` - All tests pass
4. [ ] `cargo build` - Clean build with no errors

## Project Structure

```
appimageupdate/
├── src/
│   ├── lib.rs          # Library entry point
│   ├── bin/            # Binary (CLI) entry point
│   └── ...             # Library modules
├── tests/              # Integration tests
└── examples/           # Usage examples
```

## Library Design

- Library-first approach: core functionality in `src/lib.rs` and modules
- CLI tool uses the library (no duplicated logic)
- Public API should be well-documented with rustdoc
- Use `thiserror` for error types
- Prefer synchronous APIs for simplicity (no async runtime needed unless required)

## Dependencies

Keep dependencies minimal. Prefer lightweight libraries:
- `ureq` - HTTP client (synchronous, lightweight)
- `thiserror` - Error handling
- `clap` - CLI argument parsing

Avoid adding libraries for things that can be implemented in a few dozen lines.

## Upstream Reference

The upstream C++ implementation is at: https://github.com/AppImage/AppImageUpdate

When implementing features, refer to the upstream for:
- Update information format specification
- Delta update algorithm (zsync-based)
- GPG signature validation behavior
- CLI interface design

## Additional Notes

- Target MSRV (Minimum Supported Rust Version): Latest stable
- Use `#[deny(missing_docs)]` for public APIs
- Prefer `Result<T, E>` over `Option<T>` for fallible operations with context
- AppImages use ISO 9660 format with ELF wrapper - may need specialized parsing
