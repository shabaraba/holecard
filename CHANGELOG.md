# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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
