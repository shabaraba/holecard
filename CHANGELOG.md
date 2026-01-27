# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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
