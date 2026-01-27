# Distribution Guide

This document describes how to distribute Holecard through various channels.

## Prerequisites

Before distributing, ensure:

1. All tests pass: `cargo test`
2. Code is formatted: `cargo fmt`
3. No clippy warnings: `cargo clippy`
4. README.md is up to date
5. CHANGELOG.md documents all changes
6. Version is bumped in Cargo.toml

## Distribution Channels

### 1. crates.io (Rust Package Registry)

**Setup (one-time):**
```bash
# Get your API token from https://crates.io/me
cargo login <your-api-token>
```

**Publishing:**
```bash
# Update version in Cargo.toml
vim Cargo.toml

# Dry run to check for issues
cargo publish --dry-run

# Publish
cargo publish
```

**Users install with:**
```bash
cargo install hc
```

### 2. GitHub Releases (Binary Distribution)

**Automated via GitHub Actions:**

The `.github/workflows/release.yml` workflow automatically builds and publishes binaries when you push a version tag.

**To create a release:**
```bash
# Update version in Cargo.toml
vim Cargo.toml

# Commit changes
git add Cargo.toml
git commit -m "chore: bump version to 0.2.0"

# Create and push tag
git tag v0.2.0
git push origin v0.2.0

# GitHub Actions will automatically:
# 1. Build for all platforms
# 2. Create GitHub Release
# 3. Upload binaries and checksums
```

**Supported platforms:**
- macOS Apple Silicon (aarch64-apple-darwin)
- macOS Intel (x86_64-apple-darwin) - if added to workflow
- Linux x86_64 (x86_64-unknown-linux-gnu)

**Users install with:**
```bash
# macOS Apple Silicon
curl -LO https://github.com/shabarba/holecard/releases/download/v0.1.0/hc-aarch64-apple-darwin.tar.gz
tar xzf hc-aarch64-apple-darwin.tar.gz
sudo mv hc /usr/local/bin/

# Linux
curl -LO https://github.com/shabarba/holecard/releases/download/v0.1.0/hc-x86_64-unknown-linux-gnu.tar.gz
tar xzf hc-x86_64-unknown-linux-gnu.tar.gz
sudo mv hc /usr/local/bin/
```

### 3. Homebrew (macOS/Linux Package Manager)

**Setup (one-time):**
```bash
# Use existing tap repository
git clone https://github.com/shabarba/homebrew-tap
cd homebrew-tap
```

**After each release:**
```bash
# 1. Download release archives
VERSION=0.2.0
curl -LO https://github.com/shabarba/holecard/releases/download/v${VERSION}/hc-aarch64-apple-darwin.tar.gz
curl -LO https://github.com/shabarba/holecard/releases/download/v${VERSION}/hc-x86_64-unknown-linux-gnu.tar.gz

# 2. Calculate SHA256 checksums
shasum -a 256 hc-aarch64-apple-darwin.tar.gz
shasum -a 256 hc-x86_64-unknown-linux-gnu.tar.gz

# 3. Update homebrew/hc.rb.template with:
#    - VERSION
#    - SHA256_ARM64
#    - SHA256_LINUX

# 4. Copy to tap repository
cp hc.rb homebrew-tap/Formula/

# 5. Commit and push
cd homebrew-tap
git add Formula/hc.rb
git commit -m "feat: update hc to v${VERSION}"
git push
```

**Users install with:**
```bash
brew tap shabarba/tap
brew install hc
```

### 4. cargo-binstall (Fast Binary Installation)

**No setup required!** The `Cargo.toml` already includes `[package.metadata.binstall]` configuration.

**Users install with:**
```bash
cargo binstall hc
```

This downloads pre-built binaries from GitHub Releases instead of compiling from source.

## Release Checklist

Use this checklist for each release:

- [ ] Update version in `Cargo.toml`
- [ ] Update `CHANGELOG.md` with changes
- [ ] Run `cargo test` (all tests pass)
- [ ] Run `cargo clippy` (no warnings)
- [ ] Run `cargo fmt` (code formatted)
- [ ] Update README.md if needed
- [ ] Commit changes: `git commit -m "chore: bump version to X.Y.Z"`
- [ ] Create git tag: `git tag vX.Y.Z`
- [ ] Push commit: `git push origin main`
- [ ] Push tag: `git push origin vX.Y.Z`
- [ ] Wait for GitHub Actions to complete
- [ ] Verify binaries in GitHub Releases
- [ ] Test installation from each channel
- [ ] Publish to crates.io: `cargo publish`
- [ ] Update Homebrew formula (if applicable)
- [ ] Announce release (Twitter, Reddit, etc.)

## Versioning

Follow [Semantic Versioning](https://semver.org/):

- **MAJOR** (X.0.0): Breaking changes
- **MINOR** (0.X.0): New features, backward compatible
- **PATCH** (0.0.X): Bug fixes, backward compatible

## Changelog

Keep `CHANGELOG.md` updated following [Keep a Changelog](https://keepachangelog.com/) format:

```markdown
## [Unreleased]

### Added
- New feature X

### Changed
- Improved Y

### Fixed
- Bug Z

## [0.2.0] - 2024-01-20

### Added
- Feature A
- Feature B
```

## Platform-Specific Notes

### macOS

- Universal binary not currently supported
- Separate builds for Intel and Apple Silicon
- May require users to allow the app in System Preferences on first run

### Linux

- Requires glibc (GNU libc)
- Musl builds not currently provided
- Users need D-Bus for keyring functionality

## Troubleshooting

### GitHub Actions fails to build

- Check Cargo.lock is committed
- Verify all dependencies support target platforms
- Check Actions logs for specific errors

### cargo publish fails

- Ensure package name is available on crates.io
- Check that all required fields are in Cargo.toml
- Verify README.md and LICENSE files exist

### Homebrew formula issues

- Verify SHA256 checksums match exactly
- Ensure URLs are correct
- Test formula locally: `brew install --build-from-source ./Formula/hc.rb`

## Support

For distribution issues, please open an issue on GitHub:
https://github.com/shabarba/holecard/issues
