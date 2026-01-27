# Holecard

Secure CLI password manager with dual-key encryption.

## Features

- **Dual-key encryption**: Master password + secret key for enhanced security
- **Strong cryptography**: Argon2id key derivation + AES-256-GCM encryption
- **System keyring integration**: Secret key stored securely in OS keyring
- **Session caching**: Avoid repeated password entry with configurable timeout
- **Flexible entries**: Custom key-value fields per entry
- **TOTP support**: Dedicated TOTP entry for 2FA code generation with auto-clipboard copy
- **Smart clipboard**: Copy specific fields with auto-clear after 30 seconds
- **Template injection**: Render templates with entry fields
- **Environment variables**: Run commands with secrets as env vars
- **Vault reinitialization**: Safe vault reset with confirmation prompt

## Installation

### From source (recommended)

```bash
git clone https://github.com/shabarba/holecard
cd holecard
cargo build --release
cp target/release/hc /usr/local/bin/  # or ~/.local/bin/
```

### From crates.io (coming soon)

```bash
cargo install hc
```

### From binary releases (coming soon)

Download pre-built binaries from [GitHub Releases](https://github.com/shabarba/holecard/releases).

**Supported platforms:**
- macOS Apple Silicon (aarch64)
- macOS Intel (x86_64)
- Linux x86_64

### With Homebrew (coming soon)

```bash
brew tap shabarba/tap
brew install hc
```

### With cargo-binstall (coming soon)

```bash
cargo binstall hc
```

## Quick Start

```bash
# Initialize vault (creates master password + secret key + totp entry)
hc init

# Add an entry
hc add github -f username=myuser -f password=mypass

# Or add interactively
hc add

# List entries
hc list

# Get an entry
hc get github

# Copy password to clipboard (auto-clears after 30s)
hc get github -c

# Copy specific field
hc get github -c username

# Add TOTP secret
hc totp add github JBSWY3DPEHPK3PXP

# Get TOTP code (displays + copies to clipboard)
hc totp get github
```

## Usage

### Managing Entries

```bash
# Add entry with custom fields
hc add aws -f access_key=AKIA... -f secret_key=...

# Edit existing entry
hc edit github

# Remove entry
hc rm github

# Copy specific field to clipboard
hc get github -c password    # Copy password field
hc get github -c username    # Copy username field
hc get github -c             # Copy password field (or first field if no password)

# Export vault to JSON (plaintext - handle with care)
hc export backup.json

# Import from JSON
hc import backup.json
hc import backup.json --overwrite  # Replace existing entries
```

### TOTP Support

All TOTP secrets are stored in a dedicated `totp` entry that is automatically created during initialization.

```bash
# Add TOTP secret for a service
hc totp add github JBSWY3DPEHPK3PXP
hc totp add aws KBSWY3DPEHPK3PXQ

# Get TOTP code (displays + copies to clipboard)
hc totp get github
# Output:
# TOTP Code: 123456 (valid for 28 seconds)
# ✓ Copied to clipboard (will clear in 30 seconds)

# Remove TOTP secret
hc totp rm github

# View all TOTP services
hc get totp
```

### Template Injection

```bash
# Render template with entry fields
hc inject github "https://{{username}}:{{password}}@github.com"

# Access specific fields
hc inject aws "AWS_ACCESS_KEY={{access_key}}"
```

### Running Commands with Secrets

```bash
# Entry fields become uppercase environment variables
hc run aws -- env | grep -E "^(ACCESS_KEY|SECRET_KEY)"

# Use with any command
hc run database -- psql -h localhost -U $USERNAME -d mydb
```

### Session Management

```bash
# Check session status
hc status

# Lock vault (clear cached session)
hc lock

# Configure session timeout (minutes)
hc config session-timeout 30
```

### Configuration

```bash
# View current config
hc config

# Set vault file path
hc config vault-path ~/Dropbox/vault.enc

# Set session timeout
hc config session-timeout 120

# Reinitialize vault (WARNING: deletes all data)
hc init
# Output:
# ⚠ Vault already exists!
# ⚠ Vault already exists. Reinitialize? This will DELETE ALL existing data! (y/N):
```

## Security

### Encryption

- **Key derivation**: Argon2id with master password + secret key
- **Encryption**: AES-256-GCM with random nonce per save
- **Secret key**: 160-bit random key stored in system keyring

### Backup and Recovery

Use `hc export` to backup your entire vault:

```bash
hc export backup.json
```

The export file is encrypted with a password you choose. To restore:

```bash
hc import backup.json
```

**Important**:
- Store export files in a secure location (external drive, encrypted cloud storage, etc.)
- Use a strong password for export encryption
- You need BOTH the export file and its password to restore your vault
- Regular backups protect against data loss

### Session Caching

The derived encryption key is cached in the system keyring to avoid repeated password entry. Sessions automatically expire after the configured timeout (default: 60 minutes).

## File Locations

| File | Description |
|------|-------------|
| `~/.holecard/config.toml` | Configuration file |
| `~/.holecard/vault.enc` | Encrypted vault (default) |
| `~/.holecard/session.json` | Session metadata |

## Platform Support

- **macOS**: Apple Silicon (aarch64) and Intel (x86_64)
- **Linux**: x86_64 GNU

## Building from Source

```bash
# Clone repository
git clone https://github.com/shabarba/holecard
cd holecard

# Build
cargo build --release

# Run tests
cargo test

# Check
cargo check

# Lint
cargo clippy

# Format
cargo fmt

# Install locally
cargo install --path .
```

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Contributing

Contributions are welcome! Please feel free to submit issues or pull requests.

## Acknowledgments

Built with:
- [clap](https://github.com/clap-rs/clap) - Command line argument parsing
- [argon2](https://github.com/RustCrypto/password-hashes) - Key derivation
- [aes-gcm](https://github.com/RustCrypto/AEADs) - Authenticated encryption
- [keyring](https://github.com/hwchen/keyring-rs) - System keyring access
- [totp-lite](https://github.com/fosskers/totp-lite) - TOTP implementation
