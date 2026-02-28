# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Holecard (`hc`) is a secure CLI password manager written in Rust. It uses dual-key encryption (master password + secret key) with Argon2id key derivation and AES-256-GCM encryption, and stores the secret key in the system keyring.

## Domain Terminology

The project uses poker-themed terminology consistently across domain, CLI, and user-facing messages:

- **Deck** - Top-level encrypted container (like a vault). Managed via `hc deck`.
- **Hand** - An individual entry within a deck (like a password entry). Managed via `hc hand`.
- **Card** - A key-value pair within a hand (like a field). e.g., `username=myuser`, `password=secret`.

URI format: `hc://[deck/]hand/card` or `op://[deck/]hand/card`

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
hc init                  # Initialize deck (creates master password + secret key)
hc hand add [name]       # Add hand (interactive or with -f key=value flags)
hc hand get <name>       # Get hand (--clip to copy, --totp for TOTP code)
hc hand list             # List all hands
hc hand edit <name>      # Edit hand
hc hand rm <name>        # Remove hand
hc lock                  # Clear session (require password again)
hc status                # Show session status
hc config                # View/set configuration
hc inject                # Inject secrets from URI-based template
hc run                   # Run command with URI-based env vars
hc export <file>         # Export deck to JSON
hc import <file>         # Import from JSON
hc deck list             # List all decks
hc deck create <name>    # Create a new deck
hc deck use <name>       # Set active deck
hc deck delete <name>    # Delete a deck
hc deck move <hand> <deck>  # Move hand to another deck
hc deck copy <hand> <deck>  # Copy hand to another deck
hc deck passwd           # Change master password
```

## Architecture

The project follows a layered architecture with clear separation of concerns:

```
src/
├── main.rs              # CLI entry point (routing only)
├── deck_context.rs      # DeckContext (application state management)
├── multi_deck_context.rs # MultiDeckContext (multi-deck support)
├── config.rs            # Configuration management (~/.holecard/config.toml)
├── cli/
│   ├── commands.rs      # Clap command definitions
│   └── input.rs         # Interactive input prompts
├── handlers/            # Application layer - command handlers
│   ├── deck.rs          # Hand operations (add, get, list, edit, rm)
│   ├── deck_management.rs # Deck operations (create, delete, use, move, copy, passwd)
│   ├── session.rs       # Session management (lock, status)
│   ├── config.rs        # Configuration commands
│   ├── inject.rs        # URI-based template injection
│   ├── run.rs           # URI-based env var injection
│   ├── read.rs          # URI-based secret reading
│   ├── transfer.rs      # Import/export operations
│   ├── totp.rs          # TOTP operations
│   ├── ssh.rs           # SSH key management
│   ├── provider.rs      # Secret provider management
│   ├── password.rs      # Password generation
│   └── completion.rs    # Shell completion
├── domain/              # Core business logic (no I/O dependencies)
│   ├── crypto.rs        # CryptoService trait definition
│   ├── deck.rs          # Deck data structure (hands HashMap)
│   ├── hand.rs          # Hand data structure with cards HashMap
│   ├── template.rs      # Template engine for variable injection
│   ├── secret_resolver.rs # URI-based secret resolution
│   ├── uri.rs           # URI parsing (hc:// and op://)
│   ├── totp.rs          # TOTP code generation
│   ├── ssh_key.rs       # SSH key validation
│   ├── provider.rs      # Provider trait and config
│   ├── password_gen.rs  # Password generation logic
│   └── error.rs         # Domain error types
└── infrastructure/      # I/O and external service implementations
    ├── crypto_impl.rs   # Argon2id + AES-256-GCM implementation
    ├── storage.rs       # Encrypted deck file operations
    ├── session.rs       # Session caching (derived key in keyring)
    ├── keyring.rs       # System keyring management
    └── deck_registry.rs # Multi-deck registry (vaults.toml)
```

### Key Design Patterns

- **Layered architecture**: main.rs → handlers → context → domain + infrastructure
- **Single responsibility**: Each handler module groups related commands (deck ops, hand ops, session ops, etc.)
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
