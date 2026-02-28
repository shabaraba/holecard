# Multi-Deck Support Guide

This guide covers how to use Holecard's multi-deck feature to manage multiple encrypted decks (e.g., personal, work, family).

## Table of Contents

- [Overview](#overview)
- [Quick Start](#quick-start)
- [Deck Management](#deck-management)
- [Working with Multiple Decks](#working-with-multiple-decks)
- [Use Cases](#use-cases)
- [Best Practices](#best-practices)

## Overview

Multi-deck support allows you to:

- **Separate concerns**: Personal vs. work vs. family passwords
- **Different security levels**: High-security vs. convenience decks
- **Team collaboration**: Shared decks with team members
- **Compliance**: Isolate sensitive data per regulatory requirements

Each deck has:
- Independent master password
- Shared secret key (in system keyring)
- Isolated hands and configuration
- Independent session management

## Quick Start

```bash
# Create default deck
hc init

# Create work deck
hc deck create work

# Switch to work deck
hc deck use work

# Add hand to work deck
hc hand add jira -f username=user -f password=pass

# Switch back to default deck
hc deck use default

# List all decks
hc deck list
```

## Deck Management

### Creating Decks

```bash
# Create new deck
hc deck create <name>

# Examples:
hc deck create work
hc deck create family
hc deck create project-alpha
```

### Switching Decks

```bash
# Switch active deck
hc deck use <name>

# Examples:
hc deck use work
hc deck use default

# Verify current deck
hc status
```

### Listing Decks

```bash
# List all configured decks
hc deck list

# Example output:
# Decks:
#   • default (active)
#     Path: ~/.holecard/default.enc
#     Last accessed: 2024-01-15 10:30:00
#   • work
#     Path: ~/.holecard/work.enc
#     Last accessed: 2024-01-14 18:45:00
```

### Deleting Decks

```bash
# Delete deck (with confirmation)
hc deck delete <name>

# Force delete (skip confirmation)
hc deck delete <name> --force
```

### Moving Hands Between Decks

```bash
# Move a hand from active deck to another deck
hc deck move <hand-name> <target-deck>

# Example:
hc deck move github work

# Copy a hand (keep in both decks)
hc deck copy <hand-name> <target-deck>
```

### Changing Master Password

```bash
# Change password for active deck
hc deck passwd

# Or switch to target deck first
hc deck use work
hc deck passwd
```

## Working with Multiple Decks

### Deck Context

All hand commands operate on the **active deck**:

```bash
# Check active deck
hc status

# Add hand to current deck
hc hand add card1 -f key=value

# Switch deck
hc deck use work

# Add hand to work deck
hc hand add card2 -f key=value
```

### Using --deck Flag

You can specify a deck for a single command without switching:

```bash
# Read from specific deck
hc --deck work hand get github

# Export specific deck
hc --deck work export backup.json
```

### Session Management

Each deck has an independent session:

```bash
# Lock current deck
hc lock

# Check status
hc status
```

## Use Cases

### Personal vs. Work Separation

```bash
# Setup
hc deck create work

# Personal passwords (default deck)
hc deck use default
hc hand add gmail -f username=personal@gmail.com -f password=...
hc hand add bank -f username=... -f password=...

# Work passwords
hc deck use work
hc hand add jira -f username=work@company.com -f password=...
hc hand add aws -f access_key=... -f secret_key=...
```

**Benefits:**
- Clear separation of personal and professional secrets
- Different master passwords for different threat models
- Selective backup/sharing strategies

### Family Deck

```bash
# Create family deck
hc deck create family

# Add family credentials
hc deck use family
hc hand add netflix -f email=family@example.com -f password=...
hc hand add wifi -f ssid=HomeWiFi -f password=...
hc hand add router -f admin_user=admin -f admin_pass=...
```

**Sharing:**
- Export deck: `hc export ~/family-backup.json`
- Share encrypted export file with family members
- Each member imports: `hc import ~/family-backup.json`

### Security Levels

```bash
# High-security deck (sensitive data)
hc deck create high-sec
hc deck use high-sec
hc config session-timeout 5  # Short timeout

# Convenience deck (low-risk data)
hc deck create convenience
hc deck use convenience
hc config session-timeout 120  # Long timeout
```

## Best Practices

### Naming Conventions

Use descriptive deck names:

Good:
- `personal`
- `work-acme-corp`
- `family-shared`
- `project-alpha`
- `high-security`

Bad:
- `deck1`
- `temp`
- `asdf`

### Backup Strategy

Different backup strategies per deck:

```bash
# Personal deck - local backup
hc deck use default
hc export ~/Backups/personal-$(date +%Y%m%d).json

# Work deck - per company policy
hc deck use work
hc export ~/Backups/work-$(date +%Y%m%d).json

# Family deck - cloud backup
hc deck use family
hc export ~/Dropbox/Backups/family-$(date +%Y%m%d).json
```

### Master Password Management

**Different passwords per deck:**
- Use unique master passwords for each deck
- Higher security for sensitive decks
- Consider password manager for master passwords

**Same password considerations:**
- Easier to remember
- Single point of failure
- Not recommended for high-security use cases

## Troubleshooting

### "Deck not found"

Deck name not in registry:

```bash
# Check registered decks
hc deck list
```

### Wrong Deck Active

```bash
# Check current deck
hc status

# Switch to correct deck
hc deck use <correct-name>
```

## Related Documentation

- [Security Guide](SECURITY.md) - Encryption and security model
- [SSH Key Management](SSH.md) - Managing SSH keys across decks

## License

This guide is part of the Holecard project and is licensed under MIT OR Apache-2.0.
