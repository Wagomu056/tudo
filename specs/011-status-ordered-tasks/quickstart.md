# Quickstart: Status-Ordered Task Lists

**Feature**: 011-status-ordered-tasks
**Branch**: `011-status-ordered-tasks`

## Prerequisites

- Rust stable >= 1.75 (`rustup update stable`)
- All existing tests pass: `cargo test`

## Development Workflow (TDD)

1. **Write a failing test** for the behavior you're implementing
2. **Run `cargo test`** — confirm the test fails (red)
3. **Implement** the minimum code to make the test pass (green)
4. **Refactor** while keeping all tests green
5. Repeat

## Key Files to Modify

| File | What Changes |
|------|-------------|
| `src/model.rs` | Add `StatusTaskMap` type; change `BoardState.tasks` type; custom Serialize/Deserialize |
| `src/app.rs` | Update `tasks_for_column()`, `advance_status()`, `retreat_status()`, `confirm_input()`, `delete_focused_card()`, `reorder_task_up/down()`, `apply_daily_filter()`, `focus_task_by_id()` |
| `src/ui.rs` | No logic changes — `tasks_for_column()` API stays the same |
| `tests/storage_tests.rs` | Add legacy format backward-compatibility test |

## Build & Test

```bash
# Run all tests
cargo test

# Run only tests matching a pattern
cargo test status_ordered

# Lint
cargo clippy -- -D warnings

# Format check
cargo fmt --check
```

## Implementation Order

1. `StatusTaskMap` struct + unit tests (construction, get, insert, remove)
2. `BoardState` migration (change `tasks` field type)
3. Custom Serialize/Deserialize for backward-compatible JSON
4. Update `AppState::tasks_for_column()` to delegate to `StatusTaskMap`
5. Update `advance_status()` / `retreat_status()` (remove + insert at top)
6. Update `confirm_input()` for task creation (insert at top of Todo)
7. Update `delete_focused_card()`, `reorder_task_up/down()`
8. Update `apply_daily_filter()`
9. Verify all existing tests pass
