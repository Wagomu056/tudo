# Quickstart: Click-to-Focus Items

## Prerequisites

- Rust stable ≥ 1.75 (via rustup)
- Existing tudo build working (`cargo build`)

## Build & Test

```bash
cargo test
cargo clippy -- -D warnings
cargo build
```

## Key Files to Modify

1. **`src/model.rs`** — Add `TaskHitRegion` and `MemoHitRegion` structs
2. **`src/app.rs`** — Add `clickable_tasks`/`clickable_memos` fields to `AppState`; extend `handle_left_click` to check these regions and update focus
3. **`src/ui.rs`** — Populate `clickable_tasks` in `render_column` and `clickable_memos` in `render_memo_panel`
4. **`src/main.rs`** — Clear `clickable_tasks`/`clickable_memos` at frame start; save board after focus-changing clicks

## Verification

1. Run `cargo test` — all existing + new tests pass
2. Run `cargo clippy -- -D warnings` — no warnings
3. Manual test: launch app, click on tasks in different columns, verify focus moves
4. Manual test: click on memos, verify focus switches to memo panel
5. Manual test: enter input mode (press `a`), click a task — verify focus does NOT change
6. Manual test: click empty area — verify no change
