# Contributing to Holecard

Thank you for your interest in contributing to Holecard! This document provides guidelines and instructions for contributing.

## Table of Contents

- [Code of Conduct](#code-of-conduct)
- [Getting Started](#getting-started)
- [Development Workflow](#development-workflow)
- [Coding Standards](#coding-standards)
- [Testing](#testing)
- [Submitting Changes](#submitting-changes)
- [Reporting Issues](#reporting-issues)

## Code of Conduct

This project adheres to a simple code of conduct:

- Be respectful and considerate
- Welcome newcomers and help them get started
- Focus on what is best for the project and community
- Accept constructive criticism gracefully

## Getting Started

### Prerequisites

- **Rust**: Install via [rustup](https://rustup.rs/)
- **Git**: Version control
- **macOS/Linux**: Development primarily targets Unix-like systems

### Fork and Clone

```bash
# Fork the repository on GitHub, then:
git clone https://github.com/YOUR-USERNAME/holecard
cd holecard

# Add upstream remote
git remote add upstream https://github.com/shabarba/holecard
```

### Build and Test

```bash
# Build
cargo build

# Run tests
cargo test

# Run clippy (linter)
cargo clippy

# Format code
cargo fmt
```

### Development Build

```bash
# Build and install locally for testing
cargo install --path .

# Test your changes
hc --version
```

## Development Workflow

### Branching Strategy

```bash
# Create feature branch
git checkout -b feat/my-feature

# Create fix branch
git checkout -b fix/bug-description

# Keep your branch updated
git fetch upstream
git rebase upstream/main
```

### Commit Messages

Follow [Conventional Commits](https://www.conventionalcommits.org/):

```
<type>(<scope>): <description>

[optional body]

[optional footer]
```

**Types:**
- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation changes
- `style`: Code style changes (formatting, missing semicolons, etc.)
- `refactor`: Code refactoring
- `test`: Adding or updating tests
- `chore`: Maintenance tasks

**Examples:**
```
feat(ssh): add SSH key fingerprint display

fix(crypto): correct nonce generation for export

docs(readme): update installation instructions

test(vault): add test cases for entry CRUD operations
```

### Development Tips

**Run tests on file change:**
```bash
cargo watch -x test
```

**Run specific test:**
```bash
cargo test test_encryption
```

**Debug logging:**
```bash
RUST_LOG=debug cargo run -- <command>
```

## Coding Standards

### Rust Style

- Follow [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- Use `rustfmt` for formatting (enforced in CI)
- Address all `clippy` warnings
- Prefer idiomatic Rust patterns

### Code Organization

Follow the project architecture:

```
src/
├── main.rs              # CLI entry point
├── cli/                 # Command definitions and input
├── handlers/            # Command handlers (application layer)
├── domain/              # Business logic (no I/O)
└── infrastructure/      # I/O implementations (crypto, storage, keyring)
```

**Principles:**
- **Single Responsibility**: Each module/function has one clear purpose
- **Dependency Injection**: Pass dependencies explicitly (especially `CryptoService`)
- **Error Handling**: Use `Result` and custom error types
- **No Panics**: Avoid `.unwrap()` in production code (use `?` operator)

### File Size

Keep files under 200 lines when possible. Split large files into logical modules.

### Documentation

**Public APIs:**
```rust
/// Encrypts data using AES-256-GCM with the provided key.
///
/// # Arguments
/// * `data` - The plaintext data to encrypt
/// * `derived_key` - The 32-byte encryption key
///
/// # Returns
/// Encrypted data with format: [nonce][ciphertext+tag]
///
/// # Errors
/// Returns `CryptoError` if encryption fails
pub fn encrypt_with_key(
    &self,
    data: &[u8],
    derived_key: &[u8; 32],
) -> Result<Vec<u8>, CryptoError>
```

**Complex logic:**
```rust
// Combine master password and secret key before derivation
// This provides defense-in-depth: both keys required for decryption
let mut combined = Vec::new();
combined.extend_from_slice(master_password.as_bytes());
combined.extend_from_slice(b"|");
combined.extend_from_slice(secret_key.as_bytes());
```

### Security Considerations

When contributing security-related code:

✅ **DO:**
- Use established cryptographic libraries (don't roll your own)
- Zero sensitive data after use (`zeroize` crate)
- Use constant-time comparison for secrets
- Add tests for security properties
- Document security assumptions

❌ **DON'T:**
- Log sensitive data (passwords, keys, decrypted content)
- Write sensitive data to disk unencrypted
- Use deprecated/weak cryptographic primitives
- Ignore security warnings from dependencies

## Testing

### Writing Tests

**Unit tests** - test individual functions:
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encrypt_decrypt_roundtrip() {
        let crypto = CryptoServiceImpl::new();
        let key = [0u8; 32];
        let data = b"secret data";

        let encrypted = crypto.encrypt_with_key(data, &key).unwrap();
        let decrypted = crypto.decrypt_with_key(&encrypted, &key).unwrap();

        assert_eq!(data, decrypted.as_slice());
    }
}
```

**Integration tests** - test full workflows:
```rust
// tests/integration_test.rs
#[test]
fn test_vault_lifecycle() {
    // Setup
    let temp_dir = tempfile::tempdir().unwrap();
    let vault_path = temp_dir.path().join("vault.enc");

    // Initialize vault
    // Add entry
    // Get entry
    // Verify
}
```

### Running Tests

```bash
# All tests
cargo test

# Specific test
cargo test test_encryption

# With output
cargo test -- --nocapture

# Integration tests only
cargo test --test '*'
```

### Test Coverage

Aim for:
- **80%+ coverage** for cryptographic code
- **60%+ coverage** for business logic
- **100% coverage** for critical security paths

## Submitting Changes

### Before Submitting

1. **Ensure tests pass:**
   ```bash
   cargo test
   ```

2. **Run clippy:**
   ```bash
   cargo clippy -- -D warnings
   ```

3. **Format code:**
   ```bash
   cargo fmt
   ```

4. **Update documentation:**
   - Update README.md if adding user-facing features
   - Add/update doc comments for public APIs
   - Update CHANGELOG.md (for maintainers)

### Pull Request Process

1. **Push your branch:**
   ```bash
   git push origin feat/my-feature
   ```

2. **Create Pull Request** on GitHub with:
   - Clear title (following conventional commit format)
   - Description of changes and motivation
   - Link to related issues (`Fixes #123`)
   - Screenshots/examples if UI/CLI changes

3. **PR Template:**
   ```markdown
   ## Summary
   Brief description of changes

   ## Changes
   - Added feature X
   - Fixed bug Y
   - Refactored Z

   ## Testing
   - [ ] Unit tests added/updated
   - [ ] Integration tests pass
   - [ ] Manual testing completed

   ## Related Issues
   Fixes #123
   ```

4. **Review Process:**
   - Maintainer reviews code
   - CI runs tests and linters
   - Address feedback with additional commits
   - Once approved, maintainer merges

### Merge Requirements

- ✅ All tests pass
- ✅ No clippy warnings
- ✅ Code formatted with `rustfmt`
- ✅ At least one maintainer approval
- ✅ Commit messages follow conventional format

## Reporting Issues

### Bug Reports

Use the [Bug Report template](.github/ISSUE_TEMPLATE/bug_report.md):

**Include:**
- Holecard version: `hc --version`
- Operating system and version
- Steps to reproduce
- Expected vs. actual behavior
- Relevant logs (remove sensitive data!)

**Example:**
```markdown
## Bug Description
`hc get` fails with "decryption error" after system reboot

## Environment
- Holecard: 0.3.0
- OS: macOS 14.2 (23C64)
- Shell: zsh 5.9

## Steps to Reproduce
1. `hc init`
2. `hc add test -f password=test`
3. Reboot system
4. `hc get test` → Error

## Expected Behavior
Entry should be retrieved successfully

## Actual Behavior
Error: Failed to decrypt vault: decryption failed
```

### Feature Requests

Use the [Feature Request template](.github/ISSUE_TEMPLATE/feature_request.md):

**Include:**
- Problem you're trying to solve
- Proposed solution
- Alternative solutions considered
- Additional context

### Security Issues

**DO NOT open public issues for security vulnerabilities.**

Email security concerns to: **[security@shabarba.com]**

See [SECURITY.md](docs/SECURITY.md) for details.

## Development Resources

### Project Structure

```
holecard/
├── src/
│   ├── main.rs              # CLI entry point
│   ├── cli/                 # CLI definitions
│   ├── handlers/            # Command handlers
│   ├── domain/              # Business logic
│   └── infrastructure/      # I/O implementations
├── tests/                   # Integration tests
├── docs/                    # Documentation
├── .github/                 # CI/CD workflows
└── homebrew/               # Homebrew formula
```

### Key Files

- `src/infrastructure/crypto_impl.rs` - Encryption implementation
- `src/infrastructure/storage.rs` - Vault file I/O
- `src/domain/vault.rs` - Vault data structure
- `src/handlers/vault.rs` - Vault command handlers

### Useful Commands

```bash
# Generate documentation
cargo doc --open

# Check for outdated dependencies
cargo outdated

# Security audit
cargo audit

# Benchmark (if benches exist)
cargo bench
```

## Getting Help

- **Questions**: Open a [Discussion](https://github.com/shabarba/holecard/discussions)
- **Chat**: (If Discord/Slack exists)
- **Email**: (If public email exists)

## Recognition

Contributors will be recognized in:
- GitHub contributors list
- Release notes
- CHANGELOG.md (for significant contributions)

Thank you for contributing to Holecard! 🎉
