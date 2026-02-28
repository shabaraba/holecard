# Multi-Hand Support Guide

This guide covers how to use Holecard's multi-hand feature to manage multiple encrypted hands (e.g., personal, work, family).

## Table of Contents

- [Overview](#overview)
- [Quick Start](#quick-start)
- [Hand Management](#hand-management)
- [Working with Multiple Hands](#working-with-multiple-vaults)
- [Use Cases](#use-cases)
- [Best Practices](#best-practices)

## Overview

Multi-hand support allows you to:

- **Separate concerns**: Personal vs. work vs. family passwords
- **Different security levels**: High-security vs. convenience vaults
- **Team collaboration**: Shared vaults with team members
- **Compliance**: Isolate sensitive data per regulatory requirements

Each hand has:
- Independent master password
- Separate secret key (in system keyring)
- Isolated cards and configuration
- Independent session management

## Quick Start

```bash
# Create personal hand (default)
hc init

# Create work hand
hc hand add work ~/.holecard/work.enc
hc hand init work

# Switch to work hand
hc hand switch work

# Add card to work hand
hc add jira -f username=user -f password=pass

# Switch back to personal hand
hc hand switch default

# List all hands
hc hand list
```

## Hand Management

### Creating Hands

```bash
# Add new hand
hc hand add <name> <path>

# Examples:
hc hand add work ~/.holecard/work.enc
hc hand add family ~/Dropbox/family-hand.enc
hc hand add shared /mnt/shared/team-hand.enc
```

### Initializing Hands

```bash
# Initialize hand (set master password)
hc hand init <name>

# Example:
hc hand init work
# Prompts for master password for work hand
```

### Switching Hands

```bash
# Switch active hand
hc hand switch <name>

# Examples:
hc hand switch work
hc hand switch default

# Verify current hand
hc status
```

### Listing Hands

```bash
# List all configured hands
hc hand list

# Example output:
# Vaults:
# * default - ~/.holecard/hand.enc (active)
#   work - ~/.holecard/work.enc
#   family - ~/Dropbox/family-hand.enc
```

### Removing Hands

```bash
# Remove hand from registry (doesn't delete file)
hc hand rm <name>

# Example:
hc hand rm old-work
```

**Note**: This only removes the hand from Holecard's registry. The encrypted hand file remains on disk.

### Changing Active Hand Password

```bash
# Switch to target hand
hc hand switch work

# Change password
hc change-password
```

## Working with Multiple Hands

### Hand Context

All commands operate on the **active hand**:

```bash
# Current hand context
hc status

# Add to current hand
hc add card1 -f key=value

# Switch hand
hc hand switch work

# Add to work hand
hc add card2 -f key=value
```

### Cross-Hand Operations

Currently, Holecard doesn't support cross-hand operations. To move cards between hands:

```bash
# Export from source hand
hc hand switch personal
hc export ~/temp-export.json

# Import to destination hand
hc hand switch work
hc import ~/temp-export.json

# Clean up
rm ~/temp-export.json
```

### Session Management

Each hand has an independent session:

```bash
# Lock current hand
hc lock

# Lock all hands
hc hand switch default && hc lock
hc hand switch work && hc lock
```

## Use Cases

### Personal vs. Work Separation

```bash
# Setup
hc hand add work ~/.holecard/work.enc
hc hand init work

# Personal passwords
hc hand switch default
hc add gmail -f username=personal@gmail.com -f password=...
hc add bank -f username=... -f password=...

# Work passwords
hc hand switch work
hc add jira -f username=work@company.com -f password=...
hc add aws -f access_key=... -f secret_key=...
```

**Benefits:**
- Clear separation of personal and professional secrets
- Different master passwords for different threat models
- Selective backup/sharing strategies

### Family Hand

```bash
# Create shared family hand
hc hand add family ~/Dropbox/family-hand.enc
hc hand init family

# Add family credentials
hc hand switch family
hc add netflix -f email=family@example.com -f password=...
hc add wifi -f ssid=HomeWiFi -f password=...
hc add router -f admin_user=admin -f admin_pass=...
```

**Sharing:**
- Export hand: `hc export ~/family-backup.json`
- Share encrypted export file with family members
- Each member imports: `hc import ~/family-backup.json`

### Security Levels

```bash
# High-security hand (sensitive data)
hc hand add high-sec ~/.holecard/high-security.enc
hc hand init high-sec
hc config session-timeout 5  # Short timeout

# Convenience hand (low-risk data)
hc hand add convenience ~/.holecard/convenience.enc
hc hand init convenience
hc config session-timeout 120  # Long timeout
```

### Team/Project Hands

```bash
# Project-specific hand
hc hand add project-alpha ~/work/alpha-secrets.enc
hc hand init project-alpha

hc hand switch project-alpha
hc add staging -f db_host=... -f db_pass=...
hc add prod -f db_host=... -f db_pass=...
```

## Best Practices

### Naming Conventions

Use descriptive hand names:

✅ Good:
- `personal`
- `work-acme-corp`
- `family-shared`
- `project-alpha`
- `high-security`

❌ Bad:
- `hand1`
- `temp`
- `asdf`

### File Organization

Organize hand files logically:

```
~/.holecard/
├── hand.enc           # Default personal hand
├── work.enc           # Work hand
└── config.toml        # Holecard config

~/Dropbox/
└── family-hand.enc   # Shared family hand

~/work/
└── project-secrets.enc  # Work project hand
```

### Backup Strategy

Different backup strategies per hand:

```bash
# Personal hand - local backup
hc hand switch personal
hc export ~/Backups/personal-$(date +%Y%m%d).json

# Work hand - no backup (company policy)
# (or encrypted cloud backup)

# Family hand - cloud backup
hc hand switch family
hc export ~/Dropbox/Backups/family-$(date +%Y%m%d).json
```

### Security Configuration

Adjust security per hand:

```bash
# High-security hand
hc hand switch high-sec
hc config session-timeout 5
hc config enable-biometric false  # Require password

# Convenience hand
hc hand switch convenience
hc config session-timeout 120
hc config enable-biometric true   # Enable Touch ID
```

### Hand Registry

The hand registry is stored in:
```
~/.holecard/vault_registry.json
```

This file maps hand names to file paths. Back it up:

```bash
cp ~/.holecard/vault_registry.json ~/Backups/
```

### Master Password Management

**Different passwords per hand:**
- Use unique master passwords for each hand
- Higher security for sensitive vaults
- Consider password manager for master passwords (ironic but practical)

**Same password considerations:**
- Easier to remember
- Single point of failure
- Not recommended for high-security use cases

## Advanced Usage

### Default Hand

The `default` hand is created during initial `hc init`:

```bash
# List vaults (shows default)
hc hand list

# Default hand location
~/.holecard/hand.enc
```

To change default hand location:

```bash
hc config deck-path ~/new-location.enc
```

### Switching Hands in Scripts

```bash
#!/bin/bash
# Deploy script using work hand

# Switch to work hand
hc hand switch work

# Use secrets
AWS_KEY=$(hc read hc://work/aws/access_key)
AWS_SECRET=$(hc read hc://work/aws/secret_key)

# Deploy
aws s3 sync ./dist s3://bucket --region us-east-1
```

### Temporary Hands

For one-off projects:

```bash
# Create temporary hand
hc hand add temp-project /tmp/temp-hand.enc
hc hand init temp-project

# Use it
hc hand switch temp-project
# ... add cards ...

# When done
hc hand switch default
hc hand rm temp-project
rm /tmp/temp-hand.enc
```

## Troubleshooting

### "Hand not found"

Hand name not in registry:

```bash
# Check registered hands
hc hand list

# Add hand if missing
hc hand add work ~/.holecard/work.enc
```

### "Failed to initialize hand"

Hand file already exists:

```bash
# Use existing hand (don't reinitialize)
hc hand switch work

# Or delete and recreate
rm ~/.holecard/work.enc
hc hand init work
```

### Wrong Hand Active

```bash
# Check current hand
hc status

# Switch to correct hand
hc hand switch <correct-name>
```

### Session Confusion

Each hand has independent session:

```bash
# Lock all hands
for hand_item in $(hc hand list | awk '{print $2}'); do
  hc hand switch $hand_item
  hc lock
done
```

## Related Documentation

- [Security Guide](SECURITY.md) - Encryption and security model
- [SSH Key Management](SSH.md) - Managing SSH keys across hands

## License

This guide is part of the Holecard project and is licensed under MIT OR Apache-2.0.
