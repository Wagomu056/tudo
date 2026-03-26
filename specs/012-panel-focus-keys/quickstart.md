# Quickstart: Panel Focus Keyboard Shortcuts

**Feature**: 012-panel-focus-keys
**Date**: 2026-03-26

---

## What This Feature Does

Adds two keyboard shortcuts in Normal mode:

| Key | Action                          |
|-----|---------------------------------|
| `t` | Move focus to the todo (kanban) panel |
| `m` | Move focus to the memo panel    |

---

## Files to Change

Only two files require edits:

1. **`src/main.rs`** — Add two match arms in `handle_normal_key()`
2. **`src/app.rs`** — Add unit tests in the `#[cfg(test)]` module

---

## Implementation Steps (Test-First)

### Step 1: Write failing tests (`src/app.rs`)

Add to the `#[cfg(test)]` module:

```rust
#[test]
fn test_t_key_focuses_kanban() {
    // Construct AppState with focus_area = FocusArea::Memo
    // Simulate pressing 't' (call the handler or set focus_area directly)
    // Assert focus_area == FocusArea::Kanban
}

#[test]
fn test_m_key_focuses_memo() {
    // Construct AppState with focus_area = FocusArea::Kanban
    // Simulate pressing 'm'
    // Assert focus_area == FocusArea::Memo
}

#[test]
fn test_t_key_idempotent_when_kanban_focused() {
    // Start with focus_area = FocusArea::Kanban
    // Press 't'
    // Assert focus_area == FocusArea::Kanban (unchanged)
}

#[test]
fn test_m_key_idempotent_when_memo_focused() {
    // Start with focus_area = FocusArea::Memo
    // Press 'm'
    // Assert focus_area == FocusArea::Memo (unchanged)
}
```

Confirm tests fail with `cargo test`.

### Step 2: Add key handlers (`src/main.rs`)

Inside `handle_normal_key()`, add two arms to the existing `match key_code` block:

```rust
KeyCode::Char('t') => {
    app.focus_area = FocusArea::Kanban;
}
KeyCode::Char('m') => {
    app.focus_area = FocusArea::Memo;
}
```

### Step 3: Confirm tests pass

```bash
cargo test
cargo clippy -- -D warnings
cargo fmt --check
```

---

## Verification

1. Run `cargo run`
2. Press `m` → memo panel highlights / focus moves to memo panel
3. Press `t` → kanban panel highlights / focus moves to task panel
4. While editing a task title (Input mode), press `m` or `t` → characters are entered as text, no panel switch
