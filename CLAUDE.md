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
├── main.rs              # CLI entry point and command handlers
├── config.rs            # Configuration management (~/.holecard/config.toml)
├── cli/
│   ├── commands.rs      # Clap command definitions
│   └── input.rs         # Interactive input prompts
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

- **Trait-based crypto abstraction**: `CryptoService` trait in domain, `CryptoServiceImpl` in infrastructure
- **Session caching**: Derived key cached in system keyring with configurable timeout to avoid repeated password entry
- **Dual-key encryption**: Master password + secret key combined before key derivation for additional security
- **Atomic writes**: Vault saves use temp file + rename pattern

### Data Flow

1. `VaultContext::load()` checks for cached session or prompts for master password
2. Key derivation: `master_password | secret_key` → Argon2id → 32-byte derived key
3. Vault file format: `[16-byte salt][12-byte nonce][AES-256-GCM ciphertext]`
4. Session stores derived key in system keyring with metadata file for timeout tracking

### Configuration

- Config directory: `~/.holecard/`
- Config file: `~/.holecard/config.toml`
- Default vault path: `~/.holecard/vault.enc`
- Default session timeout: 60 minutes
