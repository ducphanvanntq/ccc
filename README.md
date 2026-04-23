# ccc - Claude Code Config CLI

Quick setup tool for Claude Code configuration, with a built-in interactive TUI key manager.

## Install

### Windows (PowerShell)

```powershell
irm https://raw.githubusercontent.com/ducphanvanntq/ccc/main/install.ps1 | iex
```

### macOS / Linux

```bash
curl -fsSL https://raw.githubusercontent.com/ducphanvanntq/ccc/main/install.sh | bash
```

### Manual install

Download binary from [Releases](https://github.com/ducphanvanntq/ccc/releases), place it in a folder and add to PATH.

## Usage

```bash
# Init .claude config in current project (auto-applies default key)
ccc init

# Show current local config
ccc show config

# Show global default config
ccc show global

# Check API connection with current key
ccc check

# Check environment and config status
ccc doctor

# Check for updates
ccc update

# Show version
ccc version
```

### Key Management

#### Interactive TUI (recommended)

```bash
ccc key
```

Opens a full-screen terminal UI with:

- **Key table** — navigate with `↑↓` or `j/k`, default key marked with `★`
- **Modal dialogs** — inline input for add/rename, confirmation for remove
- **Status dashboard** — full-screen view with progress bar, API info, and live results
- **Toast notifications** — instant feedback for all operations

**Keyboard shortcuts:**

| Key | Action |
|-----|--------|
| `a` | Add a new key (modal input) |
| `d` | Set highlighted key as default |
| `u` | Use highlighted key for current folder |
| `r` | Remove highlighted key (with confirmation) |
| `n` | Rename highlighted key (modal input) |
| `s` | Check all keys status (full-screen dashboard) |
| `q` / `Esc` | Quit |

#### CLI commands

All key operations are also available as direct CLI commands:

```bash
# Add a new key
ccc key add <name> <value>

# List all saved keys
ccc key list

# Set default key (saved in keys.json, used by ccc init)
ccc key default [name]

# Use a key for current folder (.claude/settings.local.json)
ccc key use [name]

# Remove a key
ccc key remove [name]

# Rename a key
ccc key rename

# Check all keys status (test API connection)
ccc key status
```

**default** vs **use**:
- `ccc key default` — sets which key is the global default (stored in `~/.ccc/keys.json`). Used automatically when running `ccc init`.
- `ccc key use` — applies a key to the current project folder (writes to `.claude/settings.local.json`).

