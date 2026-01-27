# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Holecard (`hc`) is a secure CLI password manager written in Rust. It uses dual-key encryption (master password + secret key) with Argon2id key derivation and AES-256-GCM encryption, and stores the secret key in the system keyring.

## Build and Development Commands

```bash
# Build
cargo build --release

# Run
cargo run -- <command>
# or after build:
./target/release/hc <command>

# Check (fast compilation check)
cargo check

# Format
cargo fmt

# Lint
cargo clippy
```

## CLI Commands

```bash
hc init                  # Initialize vault (creates master password + secret key)
hc add [name]            # Add entry (interactive or with -f key=value flags)
hc get <name>            # Get entry (--clip to copy, --totp for TOTP code)
hc list                  # List all entries
hc edit <name>           # Edit entry
hc rm <name>             # Remove entry
hc lock                  # Clear session (require password again)
hc status                # Show session status
hc config                # View/set configuration
hc inject <entry> <tpl>  # Render template with entry fields
hc run <entry> -- <cmd>  # Run command with entry fields as env vars
hc export <file>         # Export vault to JSON
hc import <file>         # Import from JSON
```

## Architecture

The project follows a layered architecture with clear separation of concerns:

```
src/
├── main.rs              # CLI entry point (41 lines - routing only)
├── context.rs           # VaultContext (application state management)
├── config.rs            # Configuration management (~/.holecard/config.toml)
├── cli/
│   ├── commands.rs      # Clap command definitions
│   └── input.rs         # Interactive input prompts
├── handlers/            # Application layer - command handlers
│   ├── vault.rs         # Vault operations (init, add, get, list, edit, rm)
│   ├── session.rs       # Session management (lock, status)
│   ├── config.rs        # Configuration commands
│   ├── template.rs      # Template operations (inject, run)
│   ├── transfer.rs      # Import/export operations
│   └── totp.rs          # TOTP operations
├── domain/              # Core business logic (no I/O dependencies)
│   ├── crypto.rs        # CryptoService trait definition
│   ├── vault.rs         # Vault data structure (entries HashMap)
│   ├── entry.rs         # Entry data structure with custom fields
│   ├── template.rs      # Template engine for variable injection
│   ├── totp.rs          # TOTP code generation
│   └── error.rs         # Domain error types
└── infrastructure/      # I/O and external service implementations
    ├── crypto_impl.rs   # Argon2id + AES-256-GCM implementation
    ├── storage.rs       # Encrypted vault file operations
    ├── session.rs       # Session caching (derived key in keyring)
    └── keyring.rs       # System keyring management
```

### Key Design Patterns

- **Layered architecture**: main.rs → handlers → context → domain + infrastructure
- **Single responsibility**: Each handler module groups related commands (vault ops, session ops, etc.)
- **Trait-based crypto abstraction**: `CryptoService` trait in domain, `CryptoServiceImpl` in infrastructure
- **Session caching**: Derived key cached in system keyring with configurable timeout to avoid repeated password entry
- **Dual-key encryption**: Master password + secret key combined before key derivation for additional security
- **Atomic writes**: Vault saves use temp file + rename pattern

### Data Flow

1. `VaultContext::load()` checks for cached session or prompts for master password
2. Secret key is automatically retrieved from system keyring
3. Key derivation: `master_password | secret_key` → Argon2id → 32-byte derived key
4. Vault file format: `[16-byte salt][12-byte nonce][AES-256-GCM ciphertext]`
5. Session stores derived key in system keyring with metadata file for timeout tracking

### Backup and Recovery

- **Backup**: `hc export <file>` creates encrypted JSON export with separate password
- **Restore**: `hc import <file>` restores from encrypted export
- No plaintext secret key backups are created
- Export/import is the recommended backup strategy

### Configuration

- Config directory: `~/.holecard/`
- Config file: `~/.holecard/config.toml`
- Default vault path: `~/.holecard/vault.enc`
- Default session timeout: 60 minutes
