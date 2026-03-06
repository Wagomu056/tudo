# Quickstart: Detail Field Cursor Movement and Unicode-Aware Text Wrapping

## Prerequisites

- Rust stable >= 1.75 (`rustup update stable`)
- Existing tudo project builds (`cargo build`)

## Build & Test

```bash
# Build
cargo build

# Run all tests
cargo test

# Run specific tests for this feature
cargo test cursor       # Cursor movement tests
cargo test wrap_str     # Wrapping tests (existing + new)
cargo test detail       # Detail rendering tests

# Lint
cargo clippy -- -D warnings
```

## Manual Testing

```bash
# Run the application
cargo run

# Test cursor movement:
# 1. Select a task, press 'e' to edit detail
# 2. Type some text (mix of ASCII and Japanese)
# 3. Press Left/Right arrows to move cursor
# 4. Press Home/End to jump to beginning/end
# 5. Type characters mid-text to verify insertion at cursor
# 6. Press Backspace mid-text to verify deletion at cursor

# Test wrapping:
# 1. Enter a long detail text (>40 chars or >20 CJK chars)
# 2. Verify text wraps at panel boundary
# 3. Try mixed ASCII + Japanese text
# 4. Verify no character is split at wrap boundary

# Test cursor on wrapped lines:
# 1. Enter detail text long enough to wrap
# 2. Move cursor past the wrap point
# 3. Verify cursor appears on the correct visual line
```

## Key Files

| File | What changes |
|------|-------------|
| `src/model.rs` | `InputState` gains `cursor: usize` field and navigation methods |
| `src/app.rs` | Key event handlers call new cursor methods instead of push/pop |
| `src/ui.rs` | `render_detail_panel` uses `wrap_str` for detail text + cursor rendering |
| `src/input.rs` | May gain cursor utility functions if needed |
