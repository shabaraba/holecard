# Security Guide

This document provides detailed information about Holecard's security model, encryption implementation, and best practices.

## Table of Contents

- [Security Architecture](#security-architecture)
- [Encryption Details](#encryption-details)
- [Threat Model](#threat-model)
- [Security Best Practices](#security-best-practices)
- [Reporting Vulnerabilities](#reporting-vulnerabilities)

## Security Architecture

### Dual-Key Encryption

Holecard uses a dual-key encryption system that combines:

1. **Master Password** - User-provided password (stored only in your memory)
2. **Secret Key** - 160-bit random key (stored in system keyring)

Both keys are required to derive the encryption key. This provides defense-in-depth:

- Compromising the system keyring alone doesn't expose your deck
- Keyloggers capturing your master password alone can't decrypt your deck
- Both keys must be compromised simultaneously to access your data

### Key Derivation

```
Combined Input = master_password | "|" | secret_key
Derived Key = Argon2id(Combined Input, salt, params)
```

**Argon2id Parameters:**
- Memory: 19 MB (19456 KB)
- Iterations: 2
- Parallelism: 1
- Output length: 32 bytes (256 bits)

Argon2id is the recommended algorithm from the Password Hashing Competition (PHC), providing resistance against:
- GPU attacks (memory-hard)
- Side-channel attacks (data-independent memory access)
- Time-memory trade-off attacks

### Encryption

**Algorithm**: AES-256-GCM (Galois/Counter Mode)

- **Cipher**: AES with 256-bit key
- **Mode**: GCM (provides both confidentiality and authenticity)
- **Nonce**: 12 bytes (96 bits), randomly generated per encryption
- **Authentication tag**: 16 bytes (128 bits)

**Deck File Format:**
```
[16-byte salt][12-byte nonce][ciphertext + auth tag]
```

**Export File Format:**
```
[16-byte salt][12-byte nonce][ciphertext + auth tag]
```

### Secret Key Generation

Secret keys are generated using `OsRng` (Operating System Random Number Generator), which provides cryptographically secure randomness:

- 160 bits (20 bytes) of random data
- Base32-encoded (Crockford alphabet)
- Formatted as: `A3-XXXXXX-XXXXXX-XXXXX-XXXXX-XXXXX-XXXXXX`

### Session Caching

To avoid repeated password prompts, the derived encryption key is cached in the system keyring:

**macOS**: Stored in macOS Keychain with access control
**Linux**: Stored in Secret Service (GNOME Keyring, KWallet, etc.)

**Session Metadata:**
- Session ID
- Creation timestamp
- Last access timestamp
- Expiry time

Sessions automatically expire after the configured timeout (default: 60 minutes).

### Biometric Authentication (macOS)

On macOS, biometric authentication is implemented using the `security-framework` crate:

1. **First unlock**:
   - User provides master password via biometric prompt
   - Master password is cached in macOS Keychain with device-level encryption
   - Keychain item protected by biometric authentication

2. **Subsequent unlocks**:
   - System prompts for Touch ID/Face ID/Apple Watch
   - On success, master password retrieved from Keychain
   - Combined with secret key for deck decryption

**Supported Methods:**
- Touch ID (MacBook Pro/Air, Magic Keyboard)
- Face ID (future Mac devices)
- Apple Watch unlock
- Passkey
- macOS login password (fallback)

## Encryption Details

### Memory Safety

- **Zeroization**: Sensitive data (keys, passwords) is zeroed after use using the `zeroize` crate
- **No swapping**: Sensitive data should not be written to swap (OS-dependent)
- **Stack allocation**: Sensitive buffers use stack allocation when possible

### Random Number Generation

All random values (nonces, salts, secret keys) are generated using:

- `aes_gcm::aead::OsRng` - OS-provided CSPRNG
- **Linux**: `/dev/urandom`
- **macOS**: `SecRandomCopyBytes`

### Authentication

AES-GCM provides authenticated encryption:

- Encryption and authentication in one operation
- Any tampering with ciphertext is detected during decryption
- Prevents chosen-ciphertext attacks

## Threat Model

### What Holecard Protects Against

✅ **Deck file theft**: Without master password + secret key, deck is encrypted with AES-256
✅ **Keyloggers**: Master password alone cannot decrypt deck
✅ **System compromise**: Secret key alone cannot decrypt deck
✅ **Tampering**: AES-GCM detects any modification to encrypted data
✅ **Rainbow tables**: Argon2id with unique salt per deck
✅ **Brute force**: Argon2id is computationally expensive (19MB memory, 2 iterations)

### What Holecard Does NOT Protect Against

❌ **Malware on your device**: Active malware can intercept passwords in memory
❌ **Shoulder surfing**: Attacker observing your screen/keyboard
❌ **Compromised system keyring**: If an attacker gains access to your user account
❌ **Physical device access**: Unlocked session allows deck access
❌ **Weak master password**: Short/common passwords reduce security
❌ **Rubber-hose cryptanalysis**: Physical coercion to reveal passwords

### Attack Scenarios

**Scenario 1: Deck file stolen**
- Attacker needs BOTH master password AND secret key
- Without both, must brute force AES-256 (computationally infeasible)

**Scenario 2: System keyring compromised**
- Attacker obtains secret key
- Still needs master password to derive encryption key
- Cannot decrypt deck with secret key alone

**Scenario 3: Master password leaked**
- Attacker obtains master password (keylogger, phishing, etc.)
- Still needs secret key from system keyring
- Cannot decrypt deck with master password alone

**Scenario 4: Active session**
- Attacker gains access to unlocked device
- Can access deck data through `hc` commands
- **Mitigation**: Use `hc lock` when leaving device, configure short session timeout

## Security Best Practices

### Master Password

✅ **Use a strong, unique password**
- Minimum 12 characters
- Mix of uppercase, lowercase, numbers, symbols
- Avoid dictionary words, personal information
- Use a passphrase (e.g., "correct horse battery staple")

✅ **Never reuse passwords**
- Master password should be unique to Holecard
- Don't use it for any other service

✅ **Don't share your master password**
- No one should ever ask for it
- Holecard never transmits it anywhere

### Session Management

✅ **Lock deck when away**: `hc lock`

✅ **Configure appropriate timeout**:
```bash
# 30 minutes for shared computers
hc config session-timeout 30

# 120 minutes for personal devices
hc config session-timeout 120
```

✅ **Enable biometric authentication (macOS)**:
```bash
hc config enable-biometric true
```

### Backup Strategy

✅ **Regular backups**:
```bash
# Export deck weekly/monthly
hc export ~/Backups/holecard-$(date +%Y%m%d).json
```

✅ **Secure storage**:
- Store export files on external drive or encrypted cloud storage
- Use a strong export password (different from master password)
- Test restore process periodically

✅ **Multiple backup locations**:
- Local: External drive, USB stick
- Cloud: Encrypted cloud storage (Dropbox, Google Drive with export encryption)
- Offline: Paper backup of critical hands (air-gapped)

### System Security

✅ **Keep system updated**:
- Install OS security updates
- Update Holecard: `cargo install holecard-cli`

✅ **Use full disk encryption**:
- macOS: FileVault
- Linux: LUKS

✅ **Secure your system keyring**:
- Use strong user account password
- Lock screen when away
- Enable firewall

### Operational Security

✅ **Verify downloads**:
```bash
# Check SHA256 checksums
sha256sum -c hc-*.sha256
```

✅ **Build from source** (maximum trust):
```bash
git clone https://github.com/shabarba/holecard
cd holecard
cargo build --release
```

✅ **Review code** (if capable):
- Holecard is open source
- Encryption code: `src/infrastructure/crypto_impl.rs`
- Storage code: `src/infrastructure/storage.rs`

## Reporting Vulnerabilities

If you discover a security vulnerability in Holecard, please report it responsibly:

### DO NOT

❌ Open a public GitHub issue for security vulnerabilities

### DO

✅ Email security concerns to: **[security@shabarba.com]** (or create a GitHub Security Advisory)

✅ Include:
- Description of the vulnerability
- Steps to reproduce
- Potential impact
- Suggested fix (if any)

### Response Timeline

- **24 hours**: Initial acknowledgment
- **7 days**: Severity assessment and response plan
- **30 days**: Fix developed and tested
- **Public disclosure**: After fix is released and users have time to update

### Hall of Fame

Security researchers who responsibly disclose vulnerabilities will be acknowledged here (with permission).

## Security Audit Status

**Last Security Review**: Not yet conducted

Holecard is open source and community-reviewed. A formal third-party security audit has not been performed. Contributions and reviews from security experts are welcome.

## References

- [Argon2id Specification](https://github.com/P-H-C/phc-winner-argon2)
- [AES-GCM NIST SP 800-38D](https://csrc.nist.gov/publications/detail/sp/800-38d/final)
- [OWASP Password Storage Cheat Sheet](https://cheatsheetseries.owasp.org/cheatsheets/Password_Storage_Cheat_Sheet.html)
- [OWASP Key Management Cheat Sheet](https://cheatsheetseries.owasp.org/cheatsheets/Key_Management_Cheat_Sheet.html)

## License

This security guide is part of the Holecard project and is licensed under MIT OR Apache-2.0.
