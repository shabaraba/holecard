# Multi-Vault Support Guide

This guide covers how to use Holecard's multi-vault feature to manage multiple encrypted vaults (e.g., personal, work, family).

## Table of Contents

- [Overview](#overview)
- [Quick Start](#quick-start)
- [Vault Management](#vault-management)
- [Working with Multiple Vaults](#working-with-multiple-vaults)
- [Use Cases](#use-cases)
- [Best Practices](#best-practices)

## Overview

Multi-vault support allows you to:

- **Separate concerns**: Personal vs. work vs. family passwords
- **Different security levels**: High-security vs. convenience vaults
- **Team collaboration**: Shared vaults with team members
- **Compliance**: Isolate sensitive data per regulatory requirements

Each vault has:
- Independent master password
- Separate secret key (in system keyring)
- Isolated entries and configuration
- Independent session management

## Quick Start

```bash
# Create personal vault (default)
hc init

# Create work vault
hc vault add work ~/.holecard/work.enc
hc vault init work

# Switch to work vault
hc vault switch work

# Add entry to work vault
hc add jira -f username=user -f password=pass

# Switch back to personal vault
hc vault switch default

# List all vaults
hc vault list
```

## Vault Management

### Creating Vaults

```bash
# Add new vault
hc vault add <name> <path>

# Examples:
hc vault add work ~/.holecard/work.enc
hc vault add family ~/Dropbox/family-vault.enc
hc vault add shared /mnt/shared/team-vault.enc
```

### Initializing Vaults

```bash
# Initialize vault (set master password)
hc vault init <name>

# Example:
hc vault init work
# Prompts for master password for work vault
```

### Switching Vaults

```bash
# Switch active vault
hc vault switch <name>

# Examples:
hc vault switch work
hc vault switch default

# Verify current vault
hc status
```

### Listing Vaults

```bash
# List all configured vaults
hc vault list

# Example output:
# Vaults:
# * default - ~/.holecard/vault.enc (active)
#   work - ~/.holecard/work.enc
#   family - ~/Dropbox/family-vault.enc
```

### Removing Vaults

```bash
# Remove vault from registry (doesn't delete file)
hc vault rm <name>

# Example:
hc vault rm old-work
```

**Note**: This only removes the vault from Holecard's registry. The encrypted vault file remains on disk.

### Changing Active Vault Password

```bash
# Switch to target vault
hc vault switch work

# Change password
hc change-password
```

## Working with Multiple Vaults

### Vault Context

All commands operate on the **active vault**:

```bash
# Current vault context
hc status

# Add to current vault
hc add entry1 -f key=value

# Switch vault
hc vault switch work

# Add to work vault
hc add entry2 -f key=value
```

### Cross-Vault Operations

Currently, Holecard doesn't support cross-vault operations. To move entries between vaults:

```bash
# Export from source vault
hc vault switch personal
hc export ~/temp-export.json

# Import to destination vault
hc vault switch work
hc import ~/temp-export.json

# Clean up
rm ~/temp-export.json
```

### Session Management

Each vault has an independent session:

```bash
# Lock current vault
hc lock

# Lock all vaults
hc vault switch default && hc lock
hc vault switch work && hc lock
```

## Use Cases

### Personal vs. Work Separation

```bash
# Setup
hc vault add work ~/.holecard/work.enc
hc vault init work

# Personal passwords
hc vault switch default
hc add gmail -f username=personal@gmail.com -f password=...
hc add bank -f username=... -f password=...

# Work passwords
hc vault switch work
hc add jira -f username=work@company.com -f password=...
hc add aws -f access_key=... -f secret_key=...
```

**Benefits:**
- Clear separation of personal and professional secrets
- Different master passwords for different threat models
- Selective backup/sharing strategies

### Family Vault

```bash
# Create shared family vault
hc vault add family ~/Dropbox/family-vault.enc
hc vault init family

# Add family credentials
hc vault switch family
hc add netflix -f email=family@example.com -f password=...
hc add wifi -f ssid=HomeWiFi -f password=...
hc add router -f admin_user=admin -f admin_pass=...
```

**Sharing:**
- Export vault: `hc export ~/family-backup.json`
- Share encrypted export file with family members
- Each member imports: `hc import ~/family-backup.json`

### Security Levels

```bash
# High-security vault (sensitive data)
hc vault add high-sec ~/.holecard/high-security.enc
hc vault init high-sec
hc config session-timeout 5  # Short timeout

# Convenience vault (low-risk data)
hc vault add convenience ~/.holecard/convenience.enc
hc vault init convenience
hc config session-timeout 120  # Long timeout
```

### Team/Project Vaults

```bash
# Project-specific vault
hc vault add project-alpha ~/work/alpha-secrets.enc
hc vault init project-alpha

hc vault switch project-alpha
hc add staging -f db_host=... -f db_pass=...
hc add prod -f db_host=... -f db_pass=...
```

## Best Practices

### Naming Conventions

Use descriptive vault names:

✅ Good:
- `personal`
- `work-acme-corp`
- `family-shared`
- `project-alpha`
- `high-security`

❌ Bad:
- `vault1`
- `temp`
- `asdf`

### File Organization

Organize vault files logically:

```
~/.holecard/
├── vault.enc           # Default personal vault
├── work.enc           # Work vault
└── config.toml        # Holecard config

~/Dropbox/
└── family-vault.enc   # Shared family vault

~/work/
└── project-secrets.enc  # Work project vault
```

### Backup Strategy

Different backup strategies per vault:

```bash
# Personal vault - local backup
hc vault switch personal
hc export ~/Backups/personal-$(date +%Y%m%d).json

# Work vault - no backup (company policy)
# (or encrypted cloud backup)

# Family vault - cloud backup
hc vault switch family
hc export ~/Dropbox/Backups/family-$(date +%Y%m%d).json
```

### Security Configuration

Adjust security per vault:

```bash
# High-security vault
hc vault switch high-sec
hc config session-timeout 5
hc config enable-biometric false  # Require password

# Convenience vault
hc vault switch convenience
hc config session-timeout 120
hc config enable-biometric true   # Enable Touch ID
```

### Vault Registry

The vault registry is stored in:
```
~/.holecard/vault_registry.json
```

This file maps vault names to file paths. Back it up:

```bash
cp ~/.holecard/vault_registry.json ~/Backups/
```

### Master Password Management

**Different passwords per vault:**
- Use unique master passwords for each vault
- Higher security for sensitive vaults
- Consider password manager for master passwords (ironic but practical)

**Same password considerations:**
- Easier to remember
- Single point of failure
- Not recommended for high-security use cases

## Advanced Usage

### Default Vault

The `default` vault is created during initial `hc init`:

```bash
# List vaults (shows default)
hc vault list

# Default vault location
~/.holecard/vault.enc
```

To change default vault location:

```bash
hc config vault-path ~/new-location.enc
```

### Switching Vaults in Scripts

```bash
#!/bin/bash
# Deploy script using work vault

# Switch to work vault
hc vault switch work

# Use secrets
AWS_KEY=$(hc read hc://work/aws/access_key)
AWS_SECRET=$(hc read hc://work/aws/secret_key)

# Deploy
aws s3 sync ./dist s3://bucket --region us-east-1
```

### Temporary Vaults

For one-off projects:

```bash
# Create temporary vault
hc vault add temp-project /tmp/temp-vault.enc
hc vault init temp-project

# Use it
hc vault switch temp-project
# ... add entries ...

# When done
hc vault switch default
hc vault rm temp-project
rm /tmp/temp-vault.enc
```

## Troubleshooting

### "Vault not found"

Vault name not in registry:

```bash
# Check registered vaults
hc vault list

# Add vault if missing
hc vault add work ~/.holecard/work.enc
```

### "Failed to initialize vault"

Vault file already exists:

```bash
# Use existing vault (don't reinitialize)
hc vault switch work

# Or delete and recreate
rm ~/.holecard/work.enc
hc vault init work
```

### Wrong Vault Active

```bash
# Check current vault
hc status

# Switch to correct vault
hc vault switch <correct-name>
```

### Session Confusion

Each vault has independent session:

```bash
# Lock all vaults
for vault in $(hc vault list | awk '{print $2}'); do
  hc vault switch $vault
  hc lock
done
```

## Related Documentation

- [Security Guide](SECURITY.md) - Encryption and security model
- [SSH Key Management](SSH.md) - Managing SSH keys across vaults

## License

This guide is part of the Holecard project and is licensed under MIT OR Apache-2.0.
