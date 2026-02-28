# Keybinding Contract: Task Reordering

**Branch**: `006-task-reorder` | **Date**: 2026-02-28

## New Bindings (this feature)

| Key | Mode | Action | Notes |
|-----|------|--------|-------|
| `J` (Shift+J) | Normal | Move focused task **down** one position within its column | No-op if task is already last in column |
| `K` (Shift+K) | Normal | Move focused task **up** one position within its column | No-op if task is already first in column |

## Existing Bindings (unchanged, for context)

| Key | Mode | Action |
|-----|------|--------|
| `j` / `↓` | Normal | Move focus **cursor** down to the next card |
| `k` / `↑` | Normal | Move focus **cursor** up to the previous card |
| `h` / `←` | Normal | Move focus to the left column |
| `l` / `→` | Normal | Move focus to the right column |
| `a` | Normal | Create a new task |
| `e` | Normal | Edit focused task title |
| `E` | Normal | Edit focused task detail |
| `Enter` | Normal | Advance focused task to next status |
| `Backspace` | Normal | Retreat focused task to previous status |
| `D` | Normal | Delete focused task |
| `q` | Normal | Quit |
| `Ctrl+C` | Any | Quit |
| `Enter` | Input | Confirm input |
| `Esc` | Input | Cancel input |
| `Ctrl+J` | InputDetail | Insert newline in detail field |

## Conflict Check

- `J` (uppercase) was previously unbound — no conflict.
- `K` (uppercase) was previously unbound — no conflict.
- Lowercase `j` / `k` (navigation) are unchanged.

## Behaviour in Input Mode

`J` and `K` in `InputTitle` or `InputDetail` modes are handled by the existing
`handle_input_key` path as regular character input (`KeyCode::Char(c)`).
The reorder dispatch is guarded by `app.mode == AppMode::Normal`, so no
interference occurs.
