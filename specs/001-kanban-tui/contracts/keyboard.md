# Keyboard Contract: tudo Kanban TUI

**Feature**: 001-kanban-tui
**Date**: 2026-02-27

This document is the authoritative reference for all keyboard bindings.
Implementation MUST match this contract exactly.

---

## Normal Mode (board navigation)

| Key(s)              | Action                                          |
|---------------------|-------------------------------------------------|
| `h` or `←`          | Move focus to the adjacent left column          |
| `l` or `→`          | Move focus to the adjacent right column         |
| `j` or `↓`          | Move focus to the next card down in the column  |
| `k` or `↑`          | Move focus to the previous card up in the column|
| `a`                 | Open title input popup to create a new task     |
| `e`                 | Open title input popup to edit focused card title |
| `E`                 | Open detail input popup to edit focused card detail |
| `Enter`             | Advance focused card to next status             |
| `BackSpace`         | Move focused card to previous status            |
| `D`                 | Delete focused card permanently                 |
| `q`                 | Quit the application                            |

---

## Input Mode (text entry popup — title or detail)

| Key(s)         | Action                                                 |
|----------------|--------------------------------------------------------|
| Any printable  | Append character to the input buffer                   |
| `Backspace`    | Delete the last character in the buffer                |
| `Enter`        | Confirm: save input and return to Normal mode          |
| `Esc`          | Cancel: discard input and return to Normal mode        |

---

## Boundary Behaviours (Normal Mode)

| Situation                                     | Behaviour                          |
|-----------------------------------------------|------------------------------------|
| `h`/`←` when focused in Todo (col 0)          | No-op; optionally flash border     |
| `l`/`→` when focused in Done (col 3)          | No-op; optionally flash border     |
| `j`/`↓` on the last card in a column          | No-op                              |
| `k`/`↑` on the first card in a column         | No-op                              |
| `Enter` on a card in Done                     | No-op; Done is terminal status     |
| `BackSpace` on a card in Todo                 | No-op; Todo is initial status      |
| `e`, `E`, `D`, `Enter`, `BackSpace` on an empty column (no focused card) | No-op |

---

## Key Precedence

- In Input Mode, ALL keys listed in the Input Mode table take precedence over
  Normal Mode bindings. No Normal Mode action fires during text entry.
- `Esc` always cancels Input Mode regardless of buffer contents.

---

## Notes

- Key bindings are case-sensitive: `e` (lowercase) edits title; `E` (uppercase)
  edits detail; `D` (uppercase) deletes.
- `q` (lowercase) quits; `Q` has no binding.
- Mouse events are not handled; mouse input is ignored entirely.
