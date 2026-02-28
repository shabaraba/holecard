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
hc init                  # Initialize hand (creates master password + secret key)
hc add [name]            # Add card (interactive or with -f key=value flags)
hc get <name>            # Get card (--clip to copy, --totp for TOTP code)
hc list                  # List all cards
hc edit <name>           # Edit card
hc rm <name>             # Remove card
hc lock                  # Clear session (require password again)
hc status                # Show session status
hc config                # View/set configuration
hc inject <card> <tpl>   # Render template with card fields
hc run <card> -- <cmd>   # Run command with card fields as env vars
hc export <file>         # Export hand to JSON
hc import <file>         # Import from JSON
```

## Architecture

The project follows a layered architecture with clear separation of concerns:

```
src/
├── main.rs              # CLI entry point (41 lines - routing only)
├── context.rs           # HandContext (application state management)
├── config.rs            # Configuration management (~/.holecard/config.toml)
├── cli/
│   ├── commands.rs      # Clap command definitions
│   └── input.rs         # Interactive input prompts
├── handlers/            # Application layer - command handlers
│   ├── hand.rs          # Hand operations (init, add, get, list, edit, rm)
│   ├── session.rs       # Session management (lock, status)
│   ├── config.rs        # Configuration commands
│   ├── template.rs      # Template operations (inject, run)
│   ├── transfer.rs      # Import/export operations
│   └── totp.rs          # TOTP operations
├── domain/              # Core business logic (no I/O dependencies)
│   ├── crypto.rs        # CryptoService trait definition
│   ├── hand.rs          # Hand data structure (cards HashMap)
│   ├── card.rs          # Card data structure with custom fields
│   ├── template.rs      # Template engine for variable injection
│   ├── totp.rs          # TOTP code generation
│   └── error.rs         # Domain error types
└── infrastructure/      # I/O and external service implementations
    ├── crypto_impl.rs   # Argon2id + AES-256-GCM implementation
    ├── storage.rs       # Encrypted hand file operations
    ├── session.rs       # Session caching (derived key in keyring)
    └── keyring.rs       # System keyring management
```

### Key Design Patterns

- **Layered architecture**: main.rs → handlers → context → domain + infrastructure
- **Single responsibility**: Each handler module groups related commands (deck/hand ops, session ops, etc.)
- **Trait-based crypto abstraction**: `CryptoService` trait in domain, `CryptoServiceImpl` in infrastructure
- **Session caching**: Derived key cached in system keyring with configurable timeout to avoid repeated password prompts
- **Dual-key encryption**: Master password + secret key combined before key derivation for additional security
- **Atomic writes**: Deck saves use temp file + rename pattern

### Data Flow

1. `DeckContext::load()` checks for cached session or prompts for master password
2. Secret key is automatically retrieved from system keyring
3. Key derivation: `master_password | secret_key` → Argon2id → 32-byte derived key
4. Deck file format: `[16-byte salt][12-byte nonce][AES-256-GCM ciphertext]`
5. Session stores derived key in system keyring with metadata file for timeout tracking

### Backup and Recovery

- **Backup**: `hc export <file>` creates encrypted JSON export with separate password
- **Restore**: `hc import <file>` restores from encrypted export
- No plaintext secret key backups are created
- Export/import is the recommended backup strategy

### Configuration

- Config directory: `~/.holecard/`
- Config file: `~/.holecard/config.toml`
- Default deck path: `~/.holecard/vault.enc`
- Default session timeout: 60 minutes
