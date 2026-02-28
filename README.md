# Holecard

[![Crates.io](https://img.shields.io/crates/v/holecard-cli.svg)](https://crates.io/crates/holecard-cli)
[![License](https://img.shields.io/crates/l/holecard-cli.svg)](https://github.com/shabarba/holecard#license)
[![CI](https://github.com/shabarba/holecard/actions/workflows/ci.yml/badge.svg)](https://github.com/shabarba/holecard/actions)

A secure CLI password manager with dual-key encryption, TOTP support, and SSH key management.

## Features

- **Dual-key encryption** - Master password + secret key for enhanced security
- **Strong cryptography** - Argon2id key derivation + AES-256-GCM encryption
- **System keyring integration** - Secure storage with macOS/Linux keychain
- **TOTP support** - Generate 2FA codes with auto-clipboard copy
- **SSH key management** - Store and manage SSH keys with ssh-agent integration
- **Biometric authentication** - Touch ID, Face ID, Apple Watch support on macOS
- **Session caching** - Avoid repeated password prompts with configurable timeout

## Quick Start

```bash
# Install from crates.io
cargo install holecard-cli

# Initialize deck
hc init

# Add a hand
hc hand add github -f username=myuser -f password=mypass

# Get a hand
hc hand get github

# Copy password to clipboard (auto-clears after 30s)
hc hand get github -c
```

## Installation

### From crates.io (Recommended)

```bash
cargo install holecard-cli
```

### From source

```bash
git clone https://github.com/shabarba/holecard
cd holecard
cargo install --path .
```

### From binary releases

Download pre-built binaries from [GitHub Releases](https://github.com/shabarba/holecard/releases).

**macOS (Apple Silicon):**
```bash
curl -LO https://github.com/shabarba/holecard/releases/latest/download/hc-aarch64-apple-darwin.tar.gz
tar xzf hc-aarch64-apple-darwin.tar.gz
sudo mv hc /usr/local/bin/
```

**Linux (x86_64):**
```bash
curl -LO https://github.com/shabarba/holecard/releases/latest/download/hc-x86_64-unknown-linux-gnu.tar.gz
tar xzf hc-x86_64-unknown-linux-gnu.tar.gz
sudo mv hc /usr/local/bin/
```

### With cargo-binstall

```bash
cargo binstall holecard-cli
```

## Basic Usage

### Managing Hands (Password Entries)

```bash
# Add hand with cards (key-value fields)
hc hand add aws -f access_key=AKIA... -f secret_key=...

# Add hand interactively
hc hand add

# List all hands
hc hand list

# Get hand details
hc hand get github

# Copy specific card to clipboard
hc hand get github -c password
hc hand get github -c username

# Edit hand
hc hand edit github -f password=newpass

# Remove hand
hc hand rm github
```

### TOTP (Two-Factor Authentication)

```bash
# Add TOTP secret
hc totp add github JBSWY3DPEHPK3PXP

# Get TOTP code (displays + copies to clipboard)
hc totp get github
# Output: TOTP Code: 123456 (valid for 28 seconds)

# Remove TOTP secret
hc totp rm github
```

### SSH Key Management

```bash
# Add SSH key from file
hc ssh add my-server \
  --private-key ~/.ssh/id_rsa \
  --username user \
  --hostname server.com \
  --passphrase "optional"

# Connect via SSH (auto-loads key)
hc ssh connect user@server.com
hc ssh connect my-server

# Pass additional SSH arguments
hc ssh connect prod -- -p 2222 -v

# List loaded keys
hc ssh list

# Unload key from ssh-agent
hc ssh unload my-server
```

### Session Management

```bash
# Check session status
hc status

# Lock deck (clear cached session)
hc lock

# Configure session timeout (minutes)
hc config session-timeout 30
```

## Advanced Features

### URI-Based Secret Injection (1Password Compatible)

Inject secrets into config files using `hc://` or `op://` URI references:

```bash
# Create a template config file
cat > config.yaml << EOF
database:
  host: localhost
  username: hc://prod/db/username
  password: hc://prod/db/password
api:
  key: op://prod/api/secret_key
EOF

# Inject secrets (supports both hc:// and op:// URIs)
hc inject -i config.yaml -o config.prod.yaml

# Or from stdin
cat config.yaml | hc inject -i - > config.prod.yaml
```

**URI Format**: `hc://[deck/]hand/card` or `op://[deck/]hand/card`

### Environment Variables with URIs

Run commands with secrets from URI references:

```bash
# Use hc:// or op:// URIs
hc run \
  --env DB_URL=hc://prod/db/connection_string \
  --env API_KEY=op://prod/api/key \
  -- python app.py

# Supports environment variable substitution
export DECK=production
hc run --env DB_PASS=hc://${DECK}/db/password -- ./deploy.sh
```

### Import/Export

```bash
# Export deck to encrypted JSON
hc export backup.json

# Import from encrypted JSON
hc import backup.json
hc import backup.json --overwrite  # Replace existing hands
```

### Biometric Authentication (macOS)

Touch ID, Face ID, and Apple Watch authentication:

```bash
# Enable/disable biometric auth (enabled by default on macOS)
hc config enable-biometric true

# After initial setup:
# - First unlock: Biometric + master password
# - Subsequent unlocks: Biometric only
```

## Documentation

- [Security Guide](docs/SECURITY.md) - Encryption details and security model
- [SSH Key Management](docs/SSH.md) - Comprehensive SSH integration guide
- [Multi-Deck Support](docs/MULTI_VAULT.md) - Managing multiple decks
- [Distribution Guide](DISTRIBUTION.md) - Release and distribution process

## Security

### Encryption

- **Key derivation**: Argon2id (19MB memory, 2 iterations) with master password + secret key
- **Encryption**: AES-256-GCM with random nonce per save
- **Secret key**: 160-bit random key stored in system keyring

### Session Caching

The derived encryption key is cached in the system keyring to avoid repeated password prompts. Sessions automatically expire after the configured timeout (default: 60 minutes).

### Backup and Recovery

Use `hc export` to backup your deck:

```bash
hc export backup.json  # Encrypted with a password you choose
```

**Important**: Store export files securely. You need BOTH the export file and its password to restore your deck.

## Building from Source

```bash
git clone https://github.com/shabarba/holecard
cd holecard

# Set up Git hooks (recommended for contributors)
cp .githooks/pre-push .git/hooks/pre-push
chmod +x .git/hooks/pre-push

# Build
cargo build --release

# Run tests
cargo test

# Lint
cargo clippy

# Format
cargo fmt
```

### Git Hooks

The project includes a pre-push hook that automatically runs:
- `cargo fmt --check` - Ensures code is properly formatted
- `cargo clippy -- -D warnings` - Catches common mistakes and enforces best practices

This prevents CI failures by catching issues before pushing to remote.

## Platform Support

- **macOS**: Apple Silicon (aarch64) and Intel (x86_64)
- **Linux**: x86_64 GNU

## Contributing

Contributions are welcome! Please feel free to submit issues or pull requests.

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Acknowledgments

Built with:
- [clap](https://github.com/clap-rs/clap) - Command line argument parsing
- [argon2](https://github.com/RustCrypto/password-hashes) - Key derivation
- [aes-gcm](https://github.com/RustCrypto/AEADs) - Authenticated encryption
- [keyring](https://github.com/hwchen/keyring-rs) - System keyring access
- [totp-lite](https://github.com/fosskers/totp-lite) - TOTP implementation
