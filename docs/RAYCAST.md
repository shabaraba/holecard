# Raycast Extension

Holecard provides a [Raycast](https://www.raycast.com/) extension for quick access to your passwords and TOTP codes without leaving your keyboard.

## Installation

1. Clone this repository
2. Navigate to the `raycast/` directory
3. Install dependencies and start development mode:

```bash
cd raycast
pnpm install
pnpm dev
```

Raycast will automatically detect and load the extension. To use it permanently, import the extension via **Raycast Preferences → Extensions → + → Add Script Directory**.

## Commands

### Clip Card

Search your hands and copy a card value to clipboard.

**Open with:** `Clip Card` in Raycast

| Key | Action |
|-----|--------|
| `Enter` | Copy default card (password, or first card) |
| `⌘C` | Copy default card |
| `⌘K` | Open action panel to copy a specific card |
| `⌘N` | Open Add Hand form |

**Flow:**
```
Clip Card → type to search → Enter to copy password
                           → ⌘K → select card → Enter to copy
```

### Clip TOTP

List your TOTP services with a live countdown timer. Copies the current code to clipboard.

**Open with:** `Clip TOTP` in Raycast

| Key | Action |
|-----|--------|
| `Enter` | Copy TOTP code to clipboard |

The remaining seconds are shown as a colored badge:
- 🟢 Green — more than 10 seconds remaining
- 🟠 Orange — 10 seconds or less
- 🔴 Red — 5 seconds or less

TOTP codes are fetched from the CLI only when they expire (every 30 seconds), not on every tick.

### Manage Hand

Add, edit, and delete hands and cards.

**Open with:** `Manage Hand` in Raycast

| Key | Action |
|-----|--------|
| `Enter` | Open card management for selected hand |
| `⌘N` | Add a new hand |
| `⌘K` | Open action panel (Delete Hand, etc.) |

**Within card management:**

| Key | Action |
|-----|--------|
| `Enter` | Copy selected card |
| `⌘K` | Edit value / Add card / Remove card |

#### Adding a Hand

Fill in the Hand Name and up to 4 initial key/value card pairs. Empty pairs are ignored. An optional Notes field is also available.

#### Editing a Card Value

Select the card → `⌘K` → **Edit Value** → enter the new value → `Enter`.

The current value is not displayed (it requires biometric re-authentication), but the new value you enter will replace it.

#### Removing a Card or Hand

Both operations show a confirmation dialog before proceeding.

## Session Requirements

The extension uses the same session as the CLI. If your session has expired, commands will show a **"Holecard Locked"** error with instructions to unlock:

```bash
hc status
```

Running `hc status` in your terminal will prompt for your master password and restore the session. The extension will then work without further prompts until the session times out (default: 60 minutes).

## Security Notes

- Card values are never displayed in the Raycast UI — only keys are shown
- Clipboard is automatically cleared 30 seconds after copying (handled by the CLI)
- Sensitive operations (copy, edit) require Touch ID / biometric authentication via the CLI
