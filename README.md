# tudo

A terminal-based kanban board built with Rust and [ratatui](https://github.com/ratatui/ratatui).

<img width="1247" height="883" alt="スクリーンショット 2026-02-28 23 47 48" src="https://github.com/user-attachments/assets/5d694ec4-dc1e-4f76-82ea-a4aa240702ce" />

- 4-column kanban workflow: **Todo** → **In Progress** → **Blocked** → **Done**
- Clickable URLs in task details
- Automatic daily logs of completed tasks
- Lightweight JSON-based storage

## Installation

Requires **Rust 1.75+** (install via [rustup](https://rustup.rs/)).

```sh
# Build from source
cargo build --release

# The binary will be at target/release/tudo
```

## Keybindings

### Normal Mode

| Key | Action |
|-----|--------|
| `h` / `←` | Move to previous column |
| `l` / `→` | Move to next column |
| `k` / `↑` | Move to previous task |
| `j` / `↓` | Move to next task |
| `a` | Add new task |
| `e` | Edit task title |
| `E` | Edit task detail |
| `Enter` | Advance task to next status |
| `Backspace` | Move task to previous status |
| `D` | Delete task |
| `J` | Move task down within column |
| `K` | Move task up within column |
| `q` | Quit |
| `Ctrl+C` | Quit |

### Input Mode

| Key | Action |
|-----|--------|
| `Enter` | Confirm input |
| `Esc` | Cancel input |
| `Backspace` | Delete character |
| `Ctrl+J` | Insert new line (detail only) |

### Mouse

| Action | Effect |
|--------|--------|
| Left-click on URL | Open URL in default browser |

## Data Storage

tudo stores data in the platform-appropriate local data directory:

| Platform | Path |
|----------|------|
| macOS | `~/Library/Application Support/tudo/` |
| Linux | `~/.local/share/tudo/` (or `$XDG_DATA_HOME/tudo/`) |
| Windows | `%LOCALAPPDATA%\tudo\` |
| Fallback | `~/.tudo/` |

### Files

- **`current.log`** — JSON snapshot of the current board state. Auto-saved on every change.
- **`YYYYMMDD.log`** — JSON Lines daily log. A new entry is appended each time a task reaches the Done column.
