# Holecard Documentation

Comprehensive guides for using and contributing to Holecard.

## User Documentation

### [Security Guide](SECURITY.md)
Detailed security architecture, encryption implementation, threat model, and best practices.

**Topics:**
- Dual-key encryption system
- Argon2id key derivation
- AES-256-GCM encryption
- Session caching security
- Biometric authentication (macOS)
- Threat model and attack scenarios
- Security best practices
- Vulnerability reporting

### [SSH Key Management](SSH.md)
Complete guide to SSH key storage, management, and ssh-agent integration.

**Topics:**
- Adding SSH keys from files
- Connecting via SSH with auto-loaded keys
- Alias management
- ssh-agent integration
- Key rotation and security
- Troubleshooting

### [Multi-Vault Support](MULTI_VAULT.md)
Managing multiple encrypted vaults for different contexts (personal, work, family).

**Topics:**
- Creating and managing vaults
- Switching between vaults
- Vault use cases (personal/work separation)
- Backup strategies per vault
- Best practices

## Developer Documentation

### [Contributing Guide](../CONTRIBUTING.md)
Guidelines for contributing code, reporting issues, and development workflow.

**Topics:**
- Getting started with development
- Coding standards and style
- Testing requirements
- Pull request process
- Reporting bugs and feature requests

### [Distribution Guide](../DISTRIBUTION.md)
Release process and distribution through various channels.

**Topics:**
- crates.io publishing
- GitHub Releases automation
- Homebrew formula updates
- cargo-binstall support
- Release checklist

## Quick Links

- **Main README**: [../README.md](../README.md) - Project overview and quick start
- **Changelog**: [../CHANGELOG.md](../CHANGELOG.md) - Version history and changes
- **License**: [MIT](../LICENSE-MIT) / [Apache-2.0](../LICENSE-APACHE)

## Getting Help

- **Issues**: [GitHub Issues](https://github.com/shabarba/holecard/issues)
- **Discussions**: [GitHub Discussions](https://github.com/shabarba/holecard/discussions)
- **Security**: Email security concerns (see [SECURITY.md](SECURITY.md))

## Documentation Structure

```
holecard/
├── README.md                   # Project overview, quick start
├── CONTRIBUTING.md             # Contributing guidelines
├── DISTRIBUTION.md             # Release and distribution process
├── CHANGELOG.md                # Version history
├── CLAUDE.md                   # Project development guidelines
├── LICENSE-MIT                 # MIT license
├── LICENSE-APACHE              # Apache 2.0 license
└── docs/
    ├── README.md               # This file
    ├── SECURITY.md             # Security guide
    ├── SSH.md                  # SSH key management
    └── MULTI_VAULT.md          # Multi-vault support
```

## Contributing to Documentation

Documentation improvements are welcome! See [CONTRIBUTING.md](../CONTRIBUTING.md) for guidelines.

When updating documentation:
- Use clear, concise language
- Include code examples
- Add table of contents for long documents
- Cross-reference related documents
- Test all commands before documenting
