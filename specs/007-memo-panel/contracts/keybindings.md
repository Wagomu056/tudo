# Key Binding Contract: Memo Panel

**Feature**: 007-memo-panel
**Date**: 2026-03-01

This document defines the complete key binding contract for the memo panel feature. It extends the existing kanban key bindings with memo-specific behaviour.

---

## Focus Area Model

At any time in `Normal` mode, the application is in one of two focus areas:

| Focus Area | Description |
|------------|-------------|
| `Kanban`   | One of the 4 kanban columns (Todo/Doing/Checking/Done) is active |
| `Memo`     | The memo panel at the bottom of the kanban columns is active |

---

## Key Bindings: Normal Mode — Kanban Focus (existing, modified)

| Key | Previous Behaviour | New Behaviour |
|-----|--------------------|---------------|
| `h` / `←` | Move focus left between columns | Unchanged |
| `l` / `→` | Move focus right between columns | Unchanged |
| `k` / `↑` | Move focus up within column | Unchanged |
| `j` / `↓` | Move focus down within column; no-op at bottom | Move down within column; **if already at bottom (or column empty): enter Memo focus** |
| `a` | Open "Add Task" input popup | Unchanged (adds a task in the focused column) |
| `e` | Open "Edit Title" for focused task | Unchanged |
| `E` | Open "Edit Detail" for focused task | Unchanged |
| `J` | Reorder focused task down | Unchanged |
| `K` | Reorder focused task up | Unchanged |
| `Enter` | Advance focused task status | Unchanged |
| `Backspace` | Retreat focused task status | Unchanged |
| `D` | Delete focused task | Unchanged |
| `q` / `Ctrl+C` | Quit | Unchanged |

---

## Key Bindings: Normal Mode — Memo Focus (new)

| Key | Behaviour |
|-----|-----------|
| `h` / `←` | Move focus to previous memo item (left); no-op at index 0 |
| `l` / `→` | Move focus to next memo item (right); no-op at last item |
| `k` / `↑` | Move focus up one row within memo panel; **if already on first row: return to Kanban focus** |
| `j` / `↓` | Move focus down one row within memo panel; no-op if already on last row |
| `a` | Open "Add Memo" input popup (title entry); **only active in Memo focus** |
| `e` | Open "Edit Memo Title" for focused memo item |
| `E` | Open "Edit Memo Detail" for focused memo item |
| `D` | Delete focused memo item |
| `q` / `Ctrl+C` | Quit (unchanged) |

**Keys with NO effect in Memo focus**:

| Key | Reason |
|-----|--------|
| `J` / `K` | Memo items have no reorder operation |
| `Enter` | Memos have no status to advance |
| `Backspace` | Memos have no status to retreat |

---

## Key Bindings: Input Mode (no change)

Input mode (`InputTitle` / `InputDetail`) behaviour is unchanged for both tasks and memos:

| Key | Behaviour |
|-----|-----------|
| `Enter` | Confirm input |
| `Esc` | Cancel input |
| `Backspace` | Delete last character |
| `Ctrl+J` | Insert newline (Detail mode only) |
| Any printable char | Append to buffer |

---

## Input Popup Titles

| Context | Is Create | Popup Title |
|---------|-----------|-------------|
| Task | true | `" Add Task "` |
| Task | false (title edit) | `" Edit Title "` |
| Task | false (detail edit) | `" Edit Detail "` |
| Memo | true | `" Add Memo "` |
| Memo | false (title edit) | `" Edit Memo Title "` |
| Memo | false (detail edit) | `" Edit Memo Detail "` |

---

## Navigation Boundary Behaviour Summary

| Situation | Key | Result |
|-----------|-----|--------|
| Kanban, column not empty, not at bottom | `j` | Move down within column |
| Kanban, column not empty, at bottom | `j` | Enter Memo focus (focused_memo = 0) |
| Kanban, column empty | `j` | Enter Memo focus (focused_memo = 0) |
| Memo, not on first row | `k` | Move up one row |
| Memo, on first row | `k` | Return to Kanban focus |
| Memo, not at first item | `h` | Move left |
| Memo, at first item (index 0) | `h` | No-op |
| Memo, not at last item | `l` | Move right |
| Memo, at last item | `l` | No-op |
| Memo, not on last row | `j` | Move down one row |
| Memo, on last row | `j` | No-op |

---

## Status Bar Hint Text (updated)

**Kanban focus**:
```
a:add  e:title  E:detail  Enter:→  BS:←  D:del  J/K:move  j:memo  q:quit
```

**Memo focus**:
```
a:add  e:title  E:detail  D:del  hjkl:nav  k:back  q:quit
```
