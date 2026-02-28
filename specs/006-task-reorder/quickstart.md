# Developer Quickstart: Task Reordering

**Branch**: `006-task-reorder` | **Date**: 2026-02-28

## What changes

| File | Change |
|------|--------|
| `src/app.rs` | Add `reorder_task_down()` and `reorder_task_up()` methods to `AppState` + inline tests |
| `src/main.rs` | Add `J`/`K` key dispatch in `handle_normal_key`; add debounce state and logic in `run_app` |

No new files, no new crates, no schema changes.

---

## Step 1 — Write tests first (in `src/app.rs`)

Add to the existing `#[cfg(test)] mod tests` block:

```rust
// Reorder down: normal swap
#[test]
fn test_reorder_task_down_swaps_and_follows_focus() {
    let mut app = make_app_with_tasks(&[
        (1, "first",  Status::Todo),
        (2, "second", Status::Todo),
    ]);
    app.focused_col = Status::Todo.col_index();
    app.focused_card[Status::Todo.col_index()] = 0;  // focused on "first"

    let moved = app.reorder_task_down();

    assert!(moved);
    // Focus cursor now points to position 1 (where "first" landed)
    assert_eq!(app.focused_card[Status::Todo.col_index()], 1);
    // "second" is now first in the column
    let col = app.tasks_for_column(Status::Todo);
    assert_eq!(col[0].id, 2);
    assert_eq!(col[1].id, 1);
}

// Reorder down: boundary no-op
#[test]
fn test_reorder_task_down_at_last_is_noop() {
    let mut app = make_app_with_tasks(&[
        (1, "only", Status::Todo),
    ]);
    app.focused_col = Status::Todo.col_index();
    app.focused_card[Status::Todo.col_index()] = 0;

    let moved = app.reorder_task_down();

    assert!(!moved);
    assert_eq!(app.focused_card[Status::Todo.col_index()], 0);
}

// Reorder up: normal swap
#[test]
fn test_reorder_task_up_swaps_and_follows_focus() {
    let mut app = make_app_with_tasks(&[
        (1, "first",  Status::Todo),
        (2, "second", Status::Todo),
    ]);
    app.focused_col = Status::Todo.col_index();
    app.focused_card[Status::Todo.col_index()] = 1;  // focused on "second"

    let moved = app.reorder_task_up();

    assert!(moved);
    assert_eq!(app.focused_card[Status::Todo.col_index()], 0);
    let col = app.tasks_for_column(Status::Todo);
    assert_eq!(col[0].id, 2);
    assert_eq!(col[1].id, 1);
}

// Reorder up: boundary no-op
#[test]
fn test_reorder_task_up_at_first_is_noop() {
    let mut app = make_app_with_tasks(&[
        (1, "only", Status::Todo),
    ]);
    app.focused_col = Status::Todo.col_index();
    app.focused_card[Status::Todo.col_index()] = 0;

    let moved = app.reorder_task_up();

    assert!(!moved);
    assert_eq!(app.focused_card[Status::Todo.col_index()], 0);
}

// Reorder only affects the focused column, not others
#[test]
fn test_reorder_does_not_affect_other_columns() {
    let mut app = make_app_with_tasks(&[
        (1, "todo-a",  Status::Todo),
        (2, "todo-b",  Status::Todo),
        (3, "doing-a", Status::Doing),
    ]);
    app.focused_col = Status::Todo.col_index();
    app.focused_card[Status::Todo.col_index()] = 0;
    app.focused_card[Status::Doing.col_index()] = 0;

    app.reorder_task_down();

    // Doing column is unaffected
    let doing = app.tasks_for_column(Status::Doing);
    assert_eq!(doing.len(), 1);
    assert_eq!(doing[0].id, 3);
}

// Reorder does not change task status
#[test]
fn test_reorder_preserves_task_status() {
    let mut app = make_app_with_tasks(&[
        (1, "first",  Status::Doing),
        (2, "second", Status::Doing),
    ]);
    app.focused_col = Status::Doing.col_index();
    app.focused_card[Status::Doing.col_index()] = 0;

    app.reorder_task_down();

    for task in &app.board.tasks {
        assert_eq!(task.status, Status::Doing);
    }
}
```

Run `cargo test` — all new tests should **fail** (methods don't exist yet).

---

## Step 2 — Implement `reorder_task_down` and `reorder_task_up` in `src/app.rs`

Add to the `impl AppState` block (e.g., after `move_down`):

```rust
/// Move the focused task one position down within its column (J key).
/// Returns true if a swap occurred, false if already last (no-op).
pub fn reorder_task_down(&mut self) -> bool {
    let col = self.focused_col;
    let status = crate::model::ALL_STATUSES[col];
    let cursor = self.focused_card[col];

    let col_indices: Vec<usize> = self.board.tasks
        .iter()
        .enumerate()
        .filter(|(_, t)| t.status == status)
        .map(|(i, _)| i)
        .collect();

    if cursor + 1 >= col_indices.len() {
        return false;
    }

    self.board.tasks.swap(col_indices[cursor], col_indices[cursor + 1]);
    self.focused_card[col] = cursor + 1;
    true
}

/// Move the focused task one position up within its column (K key).
/// Returns true if a swap occurred, false if already first (no-op).
pub fn reorder_task_up(&mut self) -> bool {
    let col = self.focused_col;
    let status = crate::model::ALL_STATUSES[col];
    let cursor = self.focused_card[col];

    if cursor == 0 {
        return false;
    }

    let col_indices: Vec<usize> = self.board.tasks
        .iter()
        .enumerate()
        .filter(|(_, t)| t.status == status)
        .map(|(i, _)| i)
        .collect();

    self.board.tasks.swap(col_indices[cursor - 1], col_indices[cursor]);
    self.focused_card[col] = cursor - 1;
    true
}
```

Run `cargo test` — all tests should now **pass** (green).

---

## Step 3 — Key dispatch in `src/main.rs`

### 3a. Add J/K to `handle_normal_key`

```rust
KeyCode::Char('J') => { app.reorder_task_down(); }
KeyCode::Char('K') => { app.reorder_task_up(); }
```

### 3b. Refactor `run_app` to add debounce

Introduce two local variables at the top of `run_app`:

```rust
let mut reorder_save_pending = false;
let mut last_reorder_at: Option<std::time::Instant> = None;
```

In the poll-timeout branch (currently just `continue`):

```rust
if !event::poll(std::time::Duration::from_millis(200))? {
    if reorder_save_pending {
        if last_reorder_at
            .map(|t| t.elapsed() >= std::time::Duration::from_secs(1))
            .unwrap_or(false)
        {
            if let Err(e) = storage::save_board(&mut app.board) {
                app.status_msg = Some(e.to_string());
            }
            reorder_save_pending = false;
        }
    }
    continue;
}
```

In the key-event branch, split the existing `save_board` call into reorder vs
other:

```rust
Event::Key(key) => {
    // Quit (flush pending reorder save first)
    if (key.code == KeyCode::Char('q') && app.mode == AppMode::Normal)
        || (key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL))
    {
        if reorder_save_pending {
            let _ = storage::save_board(&mut app.board);
        }
        break;
    }

    let is_reorder = app.mode == AppMode::Normal
        && matches!(key.code, KeyCode::Char('J') | KeyCode::Char('K'));

    match app.mode {
        AppMode::Normal => handle_normal_key(app, key.code),
        AppMode::InputTitle | AppMode::InputDetail => {
            handle_input_key(app, key.code, key.modifiers)
        }
    }

    if is_reorder {
        reorder_save_pending = true;
        last_reorder_at = Some(std::time::Instant::now());
    } else {
        // Cancel debounce; the upcoming save captures all reorder changes too
        reorder_save_pending = false;
        if let Err(e) = storage::save_board(&mut app.board) {
            app.status_msg = Some(e.to_string());
        }
    }
}
```

---

## Step 4 — Verify

```bash
cargo test
cargo clippy -- -D warnings
cargo fmt --check
```

All should pass with no warnings.

---

## Acceptance smoke test

1. Launch: `cargo run`
2. Create three tasks in Todo (`a` key × 3).
3. Focus the first task, press `J` — it should move to position 2.
4. Press `J` again — it should move to position 3 (last).
5. Press `J` again — no movement (boundary no-op).
6. Press `K` — it moves back to position 2.
7. Quit (`q`), relaunch — order should be preserved.
