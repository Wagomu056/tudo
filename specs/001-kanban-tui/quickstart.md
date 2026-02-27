# Quickstart: tudo Kanban TUI

**Feature**: 001-kanban-tui
**Date**: 2026-02-27

---

## Prerequisites

- Rust stable toolchain 1.75 or later:
  ```bash
  rustup update stable
  ```

---

## Build & Run

```bash
# Development
cargo run

# Release build
cargo build --release
./target/release/tudo
```

Run `tudo` from the directory where you want your data files stored. The app
creates `current.log` (and `YYYYMMDD.log` files) in the **current working
directory**.

---

## First Launch

On first launch with no `current.log`, all columns are empty.

Press `a` to create your first task.

---

## Key Bindings Reference

### Navigation

| Key          | Action                          |
|--------------|---------------------------------|
| `h` or `←`   | Move focus to the left column   |
| `l` or `→`   | Move focus to the right column  |
| `j` or `↓`   | Move focus to the card below    |
| `k` or `↑`   | Move focus to the card above    |

### Task Actions

| Key         | Action                                              |
|-------------|-----------------------------------------------------|
| `a`         | Add a new task (appears in Todo)                    |
| `e`         | Edit the focused card's title                       |
| `E`         | Edit the focused card's detail                      |
| `Enter`     | Advance card: Todo → Doing → Checking → Done        |
| `BackSpace` | Move card back: Done → Checking → Doing → Todo      |
| `D`         | Delete the focused card permanently                 |
| `q`         | Quit                                                |

### In the Input Popup

| Key         | Action            |
|-------------|-------------------|
| Any key     | Type text         |
| `Backspace` | Delete last char  |
| `Enter`     | Save and close    |
| `Esc`       | Cancel and close  |

---

## Screen Layout

```
┌─ Todo ──────┐┌─ Doing ──────┐┌─ Checking ───┐┌─ Done ───────┐┌─ Detail ────────────┐
│ [card 1]    ││ [card 3]     ││              ││              ││ Title               │
│ [card 2]    ││              ││              ││              ││ card 1              │
│             ││              ││              ││              ││                     │
│             ││              ││              ││              ││ Detail              │
│             ││              ││              ││              ││ (task detail here)  │
└─────────────┘└──────────────┘└──────────────┘└──────────────┘└─────────────────────┘
```

- The **focused card** is highlighted in blue.
- The **detail panel** (right) shows the full text of the focused card.
- The Done column shows **only today's completed cards**. It is empty on the
  next calendar day.

---

## Data Files

All data files live in the current working directory:

| File            | Description                                          |
|-----------------|------------------------------------------------------|
| `current.log`   | Full board state; JSON; human-readable               |
| `YYYYMMDD.log`  | Daily completed task log; JSON Lines; append-only    |

You can open and edit `current.log` in any text editor. Changes take effect
on the next launch.

### Example `current.log`

```json
{
  "version": 1,
  "next_id": 3,
  "saved_at": "2026-02-27T10:05:00+09:00",
  "tasks": [
    {
      "id": 1,
      "title": "My first task",
      "detail": "Some detail here",
      "status": "Todo",
      "created_at": "2026-02-27T09:00:00+09:00",
      "done_at": null
    }
  ]
}
```

### Example `20260227.log`

```
{"title":"Review PR #42","detail":"","completed_at":"2026-02-27T17:00:00+09:00"}
```

---

## Testing

```bash
cargo test
```

Tests are in `tests/` (integration) and inline `#[cfg(test)]` modules (unit).

---

## Known Limitations (v1)

- **Detail editing**: the popup is a single-line input. To write multi-line
  detail text, edit `current.log` directly (add `\n` in the JSON string) and
  relaunch.
- **No undo/redo**: destructive actions (delete, Done transition) are
  permanent. Completed tasks are preserved in `YYYYMMDD.log`.
- **No search or filter**: all tasks are always visible.
- **Single user, local only**: no sync, no conflict resolution.
