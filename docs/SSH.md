# SSH Key Management Guide

This guide covers Holecard's SSH key management features, including secure storage, ssh-agent integration, and connection workflows.

## Table of Contents

- [Overview](#overview)
- [Quick Start](#quick-start)
- [Adding SSH Keys](#adding-ssh-keys)
- [Connecting via SSH](#connecting-via-ssh)
- [Managing Keys](#managing-keys)
- [Advanced Usage](#advanced-usage)
- [Security Considerations](#security-considerations)

## Overview

Holecard provides secure SSH key management with:

- **Encrypted storage**: SSH keys stored in vault with Argon2id + AES-256-GCM
- **ssh-agent integration**: Automatic key loading and unloading
- **Alias support**: Connect using memorable aliases instead of full SSH strings
- **Passphrase management**: Securely store and use key passphrases
- **Auto-cleanup**: Keys automatically removed from ssh-agent on `hc lock`

### Why Use Holecard for SSH Keys?

Traditional SSH key management has several pain points:

❌ Keys stored in `~/.ssh/` in plaintext (encrypted only with passphrase)
❌ Must type passphrase repeatedly (or leave keys unprotected)
❌ Difficult to manage multiple keys across different servers
❌ Keys can be accidentally committed to git repositories

Holecard solves these problems:

✅ Keys encrypted in vault with dual-key encryption
✅ Automatic passphrase entry via vault
✅ Centralized key management with aliases
✅ Keys never written to disk in plaintext

## Quick Start

```bash
# Add SSH key from file
hc add github-key \
  --file private_key=~/.ssh/id_ed25519 \
  -f alias="git@github.com"

# Connect via SSH (auto-loads key)
hc ssh connect git@github.com

# Or use entry name
hc ssh connect github-key

# List loaded keys
hc ssh list

# Unload key
hc ssh unload github-key
```

## Adding SSH Keys

### From Existing Key File

**Recommended method** - preserves newlines and formatting:

```bash
hc add my-server \
  --file private_key=~/.ssh/id_rsa \
  -f alias="user@example.com" \
  -f passphrase="optional-passphrase"
```

**Fields:**
- `private_key`: Path to private key file (required)
- `alias`: SSH connection string or shorthand (optional, comma-separated for multiple)
- `passphrase`: Key passphrase if encrypted (optional)

### Interactive Entry

```bash
hc add my-server

# Prompted for:
# - Fields to add (select "private_key")
# - File path: ~/.ssh/id_rsa
# - Alias: user@example.com
# - Passphrase: (optional)
```

### Multiple Aliases

```bash
hc add prod-server \
  --file private_key=~/.ssh/prod_key \
  -f alias="prod,user@prod.example.com,10.0.1.100"

# Now can connect with any alias:
hc ssh connect prod
hc ssh connect user@prod.example.com
hc ssh connect 10.0.1.100
```

### Using Simplified `hc ssh add`

```bash
# Simplified command for SSH key creation
hc ssh add github-key ~/.ssh/id_ed25519 \
  --alias git@github.com \
  --passphrase "optional"

# Equivalent to:
hc add github-key \
  --file private_key=~/.ssh/id_ed25519 \
  -f alias="git@github.com" \
  -f passphrase="optional"
```

## Connecting via SSH

### Basic Connection

```bash
# Using alias
hc ssh connect git@github.com

# Using entry name
hc ssh connect github-key
```

**What happens:**
1. Holecard finds entry by alias or name
2. Retrieves private key and passphrase from vault
3. Loads key into ssh-agent (with passphrase if needed)
4. Executes `ssh` with the connection string
5. Key remains loaded for session duration

### Passing SSH Arguments

Use `--` to pass additional arguments to ssh:

```bash
# Specify port
hc ssh connect prod -- -p 2222

# Enable verbose mode
hc ssh connect prod -- -v

# Multiple arguments
hc ssh connect prod -- -p 2222 -v -o StrictHostKeyChecking=no
```

### Password Authentication

For servers that use password authentication instead of keys:

```bash
# Add entry with password field
hc add my-server \
  -f username=user \
  -f password="mypassword" \
  -f alias="user@server.com"

# Connect (will use sshpass for password entry)
hc ssh connect user@server.com
```

**Note**: `sshpass` must be installed for password authentication:
```bash
# macOS
brew install hudochenkov/sshpass/sshpass

# Ubuntu/Debian
sudo apt install sshpass
```

## Managing Keys

### List Loaded Keys

```bash
hc ssh list
```

Shows:
- SSH keys currently loaded in ssh-agent
- Key fingerprints (SHA256)
- Comment/identifier

### List Vault SSH Entries

```bash
hc list
```

Shows all entries in vault, including SSH keys (entries with `private_key` field).

### Load Key Manually

```bash
# Load key into ssh-agent
hc ssh load my-server

# Load with lifetime (auto-expires after 8 hours)
hc ssh load my-server --lifetime 28800
```

### Unload Key

```bash
# Remove specific key from ssh-agent
hc ssh unload my-server

# Lock vault (removes all loaded keys)
hc lock
```

### Update SSH Key

```bash
# Edit existing entry
hc edit my-server --file private_key=~/.ssh/new_key

# Update alias
hc edit my-server -f alias="newuser@example.com"

# Update passphrase
hc edit my-server -f passphrase="newpassphrase"
```

### Remove SSH Key

```bash
hc rm my-server
```

## Advanced Usage

### Key Forwarding

```bash
# Load key, then use SSH with agent forwarding
hc ssh load github-key
ssh -A user@jumpbox.example.com
```

### Multiple Keys per Entry

You can store multiple keys in separate entries with the same alias:

```bash
# Personal GitHub key
hc add github-personal \
  --file private_key=~/.ssh/id_ed25519_personal \
  -f alias="git@github.com"

# Work GitHub key
hc add github-work \
  --file private_key=~/.ssh/id_ed25519_work \
  -f alias="git@github.com-work"

# Connect with specific key
hc ssh connect git@github.com           # Uses github-personal
hc ssh connect github-work               # Uses github-work
```

### Session Management

Keys loaded via `hc ssh` remain in ssh-agent for the session:

```bash
# Load key
hc ssh connect prod

# Use git/rsync/etc. with loaded key
git push origin main
rsync -avz ./files/ user@prod:/data/

# Clean up when done
hc lock  # Removes all loaded keys
```

### Generating New SSH Keys

Holecard doesn't generate SSH keys, but you can use standard tools:

```bash
# Generate Ed25519 key (recommended)
ssh-keygen -t ed25519 -C "user@example.com" -f ~/.ssh/id_ed25519_new

# Generate RSA key (4096-bit)
ssh-keygen -t rsa -b 4096 -C "user@example.com" -f ~/.ssh/id_rsa_new

# Add to Holecard
hc add my-new-key \
  --file private_key=~/.ssh/id_ed25519_new \
  -f alias="user@server.com"

# Optional: Remove plaintext key
rm ~/.ssh/id_ed25519_new
```

## Security Considerations

### Key Storage

✅ **Encrypted in vault**: Keys stored with Argon2id + AES-256-GCM
✅ **No plaintext on disk**: Keys never written to `~/.ssh/` unless you explicitly do so
✅ **Passphrase protection**: Key passphrases stored encrypted in vault

### ssh-agent Security

⚠️ **Keys in memory**: Loaded keys reside in ssh-agent memory
⚠️ **Session duration**: Keys remain loaded until explicitly unloaded
⚠️ **Socket access**: ssh-agent socket accessible to user account

**Mitigations:**
- Use `hc lock` when finished to clear all keys
- Configure short session timeouts
- Use `--lifetime` parameter for time-limited key access

### Passphrase Best Practices

- Use strong passphrases for SSH keys (even when in vault)
- Different passphrase than vault master password
- Consider passphraseless keys only for automation use cases

### Audit Trail

```bash
# Check what keys are loaded
hc ssh list

# View SSH entry details (without exposing key)
hc get my-server

# Export for backup (encrypted)
hc export ~/backup.json
```

### Key Rotation

Regularly rotate SSH keys:

```bash
# 1. Generate new key
ssh-keygen -t ed25519 -f ~/.ssh/id_ed25519_new

# 2. Add public key to servers
ssh-copy-id -i ~/.ssh/id_ed25519_new.pub user@server.com

# 3. Update Holecard entry
hc edit my-server --file private_key=~/.ssh/id_ed25519_new

# 4. Remove old key from servers
# (manually remove from ~/.ssh/authorized_keys on each server)

# 5. Delete old key files
rm ~/.ssh/id_ed25519_old*
```

## Troubleshooting

### "Could not open a connection to your authentication agent"

ssh-agent is not running. Start it:

```bash
eval $(ssh-agent -s)
```

### "Permission denied (publickey)"

Possible causes:
1. Wrong key for server
2. Public key not on server
3. Key passphrase incorrect

**Debug:**
```bash
# Test SSH connection manually
hc ssh load my-server
ssh -v user@server.com
```

### "Bad passphrase"

Key passphrase in vault is incorrect. Update it:

```bash
hc edit my-server -f passphrase="correct-passphrase"
```

### Keys Not Unloading

```bash
# Force remove all keys
ssh-add -D

# Or restart ssh-agent
killall ssh-agent
eval $(ssh-agent -s)
```

## Examples

### GitHub Setup

```bash
# Add GitHub SSH key
hc add github \
  --file private_key=~/.ssh/id_ed25519 \
  -f alias="git@github.com"

# Test connection
hc ssh connect git@github.com

# Clone repository (key auto-loaded in session)
git clone git@github.com:user/repo.git
```

### Multiple Servers

```bash
# Add production server
hc add prod \
  --file private_key=~/.ssh/prod_key \
  -f alias="prod,user@prod.example.com"

# Add staging server
hc add staging \
  --file private_key=~/.ssh/staging_key \
  -f alias="staging,user@staging.example.com"

# Connect to production
hc ssh connect prod

# Connect to staging
hc ssh connect staging
```

### Bastion/Jump Host

```bash
# Load key for bastion
hc ssh load bastion-key

# SSH to internal server via bastion
ssh -J user@bastion.example.com user@internal.example.com
```

## Related Documentation

- [Security Guide](SECURITY.md) - Encryption and security model
- [Multi-Vault Support](MULTI_VAULT.md) - Managing multiple vaults

## License

This guide is part of the Holecard project and is licensed under MIT OR Apache-2.0.
