# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.3.3](https://github.com/shabaraba/holecard/compare/holecard-cli-v0.3.2...holecard-cli-v0.3.3) (2026-02-26)


### Documentation

* restructure and improve documentation ([#60](https://github.com/shabaraba/holecard/issues/60)) ([b137f6f](https://github.com/shabaraba/holecard/commit/b137f6fd3af05eef4539b07f72f9392c6e010240))

## [0.3.2](https://github.com/shabaraba/holecard/compare/holecard-cli-v0.3.1...holecard-cli-v0.3.2) (2026-02-26)


### Bug Fixes

* correct TOTP code generation by passing timestamp directly to totp_custom ([#58](https://github.com/shabaraba/holecard/issues/58)) ([c7c5eff](https://github.com/shabaraba/holecard/commit/c7c5eff5824f9a6a9f9fd8bf28eeb65bedfbfcb8))

## [0.3.1](https://github.com/shabaraba/holecard/compare/holecard-cli-v0.3.0...holecard-cli-v0.3.1) (2026-02-06)


### Bug Fixes

* pull latest changes before cargo publish in gh-hooks ([#56](https://github.com/shabaraba/holecard/issues/56)) ([e40743d](https://github.com/shabaraba/holecard/commit/e40743d8ec194ad76d6c15ef0839333dcc687d39))

## [0.3.0](https://github.com/shabaraba/holecard/compare/holecard-cli-v0.2.6...holecard-cli-v0.3.0) (2026-02-06)


### ⚠ BREAKING CHANGES

* Command structure has been reorganized for better UX and consistency across the CLI.

### Features

* add comprehensive distribution setup ([#4](https://github.com/shabaraba/holecard/issues/4)) ([cd4e214](https://github.com/shabaraba/holecard/commit/cd4e2140464c7ca3856a8183da9e5bb2c117a8df))
* add master password change functionality ([#36](https://github.com/shabaraba/holecard/issues/36)) ([d40d40e](https://github.com/shabaraba/holecard/commit/d40d40eb91c5385fcc71d529c4b425d5c48717a0))
* add multi-vault support with vault management commands ([#33](https://github.com/shabaraba/holecard/issues/33)) ([fcc7b3d](https://github.com/shabaraba/holecard/commit/fcc7b3d1498290884f6d1fde1adbaa4d32f5e3f2))
* add password authentication support for ssh connect ([#46](https://github.com/shabaraba/holecard/issues/46)) ([d9a8179](https://github.com/shabaraba/holecard/commit/d9a81794cbf4c661e4db2a2ca4f62ac62bf214ca))
* add password encryption for export/import ([#1](https://github.com/shabaraba/holecard/issues/1)) ([104115c](https://github.com/shabaraba/holecard/commit/104115c13922e81864db7c09a2bf673566a37e63))
* add password generation functionality ([#32](https://github.com/shabaraba/holecard/issues/32)) ([343fd1e](https://github.com/shabaraba/holecard/commit/343fd1e3978ff6cb39f6b92bed6a809bc1236954))
* add provider integration for GitHub and Cloudflare ([#6](https://github.com/shabaraba/holecard/issues/6)) ([0a350d6](https://github.com/shabaraba/holecard/commit/0a350d6098c21ff1cc6a73282143beb4e237c4ea))
* add shell completion support (bash/zsh/fish) ([#38](https://github.com/shabaraba/holecard/issues/38)) ([e61917e](https://github.com/shabaraba/holecard/commit/e61917e136100edf56fe11c47b265c90bfe5d731))
* add ssh add command for simplified SSH entry creation ([#49](https://github.com/shabaraba/holecard/issues/49)) ([61728f0](https://github.com/shabaraba/holecard/commit/61728f0f310ab1423098d254d534379e6a65d44b))
* add SSH key management with ssh-agent integration ([#34](https://github.com/shabaraba/holecard/issues/34)) ([61fc88e](https://github.com/shabaraba/holecard/commit/61fc88e3991c5102da2e3e10820a5e9eff27467d))
* add TOTP support with dedicated entry and smart clipboard ([91f7c16](https://github.com/shabaraba/holecard/commit/91f7c1673f2d185be67480c0725ed3f409f50d77))
* add Touch ID authentication for macOS ([#37](https://github.com/shabaraba/holecard/issues/37)) ([6adf4b0](https://github.com/shabaraba/holecard/commit/6adf4b0b6d9fb1230086dcafc45c141309d162e6))
* initial implementation of holecard password manager ([c233caf](https://github.com/shabaraba/holecard/commit/c233caf58159c54a7f0c1bdfab360215b7ccf42e))
* replace ssh list to show vault SSH entries ([#54](https://github.com/shabaraba/holecard/issues/54)) ([6aac596](https://github.com/shabaraba/holecard/commit/6aac596f1c540e940d65a174f3f8d3aea87d7558))
* シークレット参照形式（hc://vault/item/field） ([#35](https://github.com/shabaraba/holecard/issues/35)) ([6c81a79](https://github.com/shabaraba/holecard/commit/6c81a798b6338098c5febc076c95e27b752dfab0))


### Bug Fixes

* add debug logging and checkout ref fix for publish workflow ([#51](https://github.com/shabaraba/holecard/issues/51)) ([4db0f0a](https://github.com/shabaraba/holecard/commit/4db0f0a1a1ce18931f4f1b1000f5a03f81c741d1))
* add workflow_dispatch and created trigger to publish-crates workflow ([#50](https://github.com/shabaraba/holecard/issues/50)) ([1a57fe4](https://github.com/shabaraba/holecard/commit/1a57fe492db447e827d9dd94361a75654a6e70fb))


### Code Refactoring

* improve codebase architecture and security ([#2](https://github.com/shabaraba/holecard/issues/2)) ([5a5080e](https://github.com/shabaraba/holecard/commit/5a5080e62c55d4a1884f3a1bb62abfda76ad494d))
* rename project from pm to hc (holecard) ([d476d5d](https://github.com/shabaraba/holecard/commit/d476d5d7aba8e8cde5c1afd472f67bedac9792aa))

## [0.2.6](https://github.com/shabaraba/holecard/compare/v0.2.5...v0.2.6) (2026-02-04)


### Bug Fixes

* add debug logging and checkout ref fix for publish workflow ([#51](https://github.com/shabaraba/holecard/issues/51)) ([4db0f0a](https://github.com/shabaraba/holecard/commit/4db0f0a37e5e4a80d28f9b857d27ed44ce6ab551))
* add workflow_dispatch and created trigger to publish-crates workflow ([#50](https://github.com/shabaraba/holecard/issues/50)) ([1a57fe4](https://github.com/shabaraba/holecard/commit/1a57fe4c2ffa64ec48a7b0d9c89fff13a3a51e55))

## [0.2.5](https://github.com/shabaraba/holecard/compare/v0.2.4...v0.2.5) (2026-02-04)


### Features

* add ssh add command for simplified SSH entry creation ([#49](https://github.com/shabaraba/holecard/issues/49)) ([61728f0](https://github.com/shabaraba/holecard/commit/61728f0f310ab1423098d254d534379e6a65d44b))

## [0.2.4](https://github.com/shabaraba/holecard/compare/v0.2.3...v0.2.4) (2026-02-04)


### Features

* add password authentication support for ssh connect ([#46](https://github.com/shabaraba/holecard/issues/46)) ([d9a8179](https://github.com/shabaraba/holecard/commit/d9a81794cbf4c661e4db2a2ca4f62ac62bf214ca))

## [0.2.3](https://github.com/shabaraba/holecard/compare/v0.2.2...v0.2.3) (2026-02-04)


### Miscellaneous

* fix binstall URLs and README after crate rename ([#45](https://github.com/shabaraba/holecard/issues/45)) ([727cf29](https://github.com/shabaraba/holecard/commit/727cf29b11ad96cf2f920db50637f65a3da702ff))
* rename crate from hc to holecard-cli ([#43](https://github.com/shabaraba/holecard/issues/43)) ([3c623ab](https://github.com/shabaraba/holecard/commit/3c623ab375e5c7e00b5168b14b038e423b5bb543))

## [0.2.2](https://github.com/shabaraba/holecard/compare/v0.2.1...v0.2.2) (2026-02-04)


### Miscellaneous

* update Cargo.toml version to 0.2.1 and fix release-please config ([#41](https://github.com/shabaraba/holecard/issues/41)) ([e11602c](https://github.com/shabaraba/holecard/commit/e11602cfc52fcf2e2d46285b3dae59e34edae742))

## [0.2.1](https://github.com/shabaraba/holecard/compare/v0.2.0...v0.2.1) (2026-02-03)


### Miscellaneous

* add automatic binary upload to release-please workflow ([#39](https://github.com/shabaraba/holecard/issues/39)) ([8f72600](https://github.com/shabaraba/holecard/commit/8f72600a1982dc4a21d529be588194ebee79e2e6))

## [0.2.0](https://github.com/shabaraba/holecard/compare/v0.1.1...v0.2.0) (2026-02-02)


### ⚠ BREAKING CHANGES

* Command structure has been reorganized for better UX and consistency across the CLI.

### Features

* add master password change functionality ([#36](https://github.com/shabaraba/holecard/issues/36)) ([d40d40e](https://github.com/shabaraba/holecard/commit/d40d40eb91c5385fcc71d529c4b425d5c48717a0))
* add multi-vault support with vault management commands ([#33](https://github.com/shabaraba/holecard/issues/33)) ([fcc7b3d](https://github.com/shabaraba/holecard/commit/fcc7b3d1498290884f6d1fde1adbaa4d32f5e3f2))
* add password generation functionality ([#32](https://github.com/shabaraba/holecard/issues/32)) ([343fd1e](https://github.com/shabaraba/holecard/commit/343fd1e3978ff6cb39f6b92bed6a809bc1236954))
* add provider integration for GitHub and Cloudflare ([#6](https://github.com/shabaraba/holecard/issues/6)) ([0a350d6](https://github.com/shabaraba/holecard/commit/0a350d6098c21ff1cc6a73282143beb4e237c4ea))
* add shell completion support (bash/zsh/fish) ([#38](https://github.com/shabaraba/holecard/issues/38)) ([e61917e](https://github.com/shabaraba/holecard/commit/e61917e136100edf56fe11c47b265c90bfe5d731))
* add SSH key management with ssh-agent integration ([#34](https://github.com/shabaraba/holecard/issues/34)) ([61fc88e](https://github.com/shabaraba/holecard/commit/61fc88e3991c5102da2e3e10820a5e9eff27467d))
* add Touch ID authentication for macOS ([#37](https://github.com/shabaraba/holecard/issues/37)) ([6adf4b0](https://github.com/shabaraba/holecard/commit/6adf4b0b6d9fb1230086dcafc45c141309d162e6))
* シークレット参照形式（hc://vault/item/field） ([#35](https://github.com/shabaraba/holecard/issues/35)) ([6c81a79](https://github.com/shabaraba/holecard/commit/6c81a798b6338098c5febc076c95e27b752dfab0))

## [0.1.1](https://github.com/shabaraba/holecard/compare/v0.1.0...v0.1.1) (2026-01-27)


### Features

* add comprehensive distribution setup ([#4](https://github.com/shabaraba/holecard/issues/4)) ([cd4e214](https://github.com/shabaraba/holecard/commit/cd4e2140464c7ca3856a8183da9e5bb2c117a8df))
* add password encryption for export/import ([#1](https://github.com/shabaraba/holecard/issues/1)) ([104115c](https://github.com/shabaraba/holecard/commit/104115c13922e81864db7c09a2bf673566a37e63))
* add TOTP support with dedicated entry and smart clipboard ([91f7c16](https://github.com/shabaraba/holecard/commit/91f7c1673f2d185be67480c0725ed3f409f50d77))
* initial implementation of holecard password manager ([c233caf](https://github.com/shabaraba/holecard/commit/c233caf58159c54a7f0c1bdfab360215b7ccf42e))


### Code Refactoring

* improve codebase architecture and security ([#2](https://github.com/shabaraba/holecard/issues/2)) ([5a5080e](https://github.com/shabaraba/holecard/commit/5a5080e62c55d4a1884f3a1bb62abfda76ad494d))
* rename project from pm to hc (holecard) ([d476d5d](https://github.com/shabaraba/holecard/commit/d476d5d7aba8e8cde5c1afd472f67bedac9792aa))


### Miscellaneous

* remove .vibing directory from git tracking ([3132b25](https://github.com/shabaraba/holecard/commit/3132b25fc1d63796c0f814ff6222603cb51b6461))

## [Unreleased]

## [0.1.0] - 2024-01-27

### Added
- Initial release
- Dual-key encryption (master password + secret key)
- Argon2id key derivation
- AES-256-GCM authenticated encryption
- System keyring integration for secret key storage
- Session caching with configurable timeout
- TOTP support for 2FA codes
- Smart clipboard with auto-clear
- Custom key-value fields per entry
- Template injection for configuration files
- Command execution with secrets as environment variables
- Encrypted export/import functionality
- CLI commands: init, add, get, list, edit, rm, lock, status, config, inject, run, export, import
- Configuration management (~/.holecard/config.toml)
- Automatic vault reinitialization with confirmation

### Security
- Zeroize sensitive data in memory
- Atomic vault writes (temp file + rename)
- Session timeout enforcement
- No plaintext credentials on disk

[Unreleased]: https://github.com/shabarba/holecard/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/shabarba/holecard/releases/tag/v0.1.0
