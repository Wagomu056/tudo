# Research: Task Reordering Within Columns

**Branch**: `006-task-reorder` | **Date**: 2026-02-28

## Finding 1 — Uppercase Key Detection in crossterm

**Decision**: Use `KeyCode::Char('J')` and `KeyCode::Char('K')` (uppercase char literals).

**Rationale**: crossterm delivers uppercase letters as `KeyCode::Char('J')` when
Shift is held. The existing codebase already uses this pattern for `'D'` (delete)
and `'E'` (edit detail) in `handle_normal_key`. No need to check
`KeyModifiers::SHIFT` separately — the char value alone uniquely identifies the
keypress when `modifiers` are ignored for regular letter keys.

**Verification**: See `src/main.rs` lines 123 and 138:
```rust
KeyCode::Char('E') => app.open_edit_detail(),
KeyCode::Char('D') => { ... }
```
Both use uppercase char literals without checking modifiers. Same pattern applies.

**Alternatives considered**:
- Checking `key.modifiers.contains(KeyModifiers::SHIFT)` — redundant given
  crossterm already normalises case into the char value.
- Using raw scan codes — platform-dependent, not portable.

---

## Finding 2 — Vec Swap for In-Place Column Reorder

**Decision**: Use `Vec::swap(idx_a, idx_b)` on `BoardState.tasks` to exchange
the positions of two tasks.

**Rationale**: `BoardState.tasks: Vec<Task>` already stores all tasks in a flat
list. The order of tasks belonging to the same column is their relative insertion
order in the Vec. `tasks_for_column` collects them via `iter().filter()`, which
preserves Vec order. Swapping two elements in the Vec directly reorders them
within the column. This requires:
1. Collecting the Vec indices of tasks in the focused column.
2. Swapping `tasks[col_indices[cursor]]` and `tasks[col_indices[cursor+1]]`.

`Vec::swap` is `O(1)`, in-place, and stable in Rust's standard library. No
additional data structure or `position` field is needed.

**No schema change required**: JSON serialization of `Vec<Task>` preserves
element order. The new order is naturally persisted on the next save.

**Alternatives considered**:
- Adding an explicit `position: u32` field to `Task` — unnecessary complexity;
  requires maintaining uniqueness and renumbering. Rejected (YAGNI).
- Using a separate `Vec<u64>` per column for ordered IDs — indirection without
  benefit for a single-user local app. Rejected.

---

## Finding 3 — Debounce Without Threads in a 200 ms Poll Loop

**Decision**: Track debounce state with two local variables in `run_app`:
`reorder_save_pending: bool` and `last_reorder_at: Option<std::time::Instant>`.
Check elapsed time in the existing poll-timeout branch.

**Rationale**: The event loop already polls with `event::poll(Duration::from_millis(200))`.
On each timeout (every 200 ms), check `last_reorder_at.elapsed() >= 1 second`.
If true and pending, call `save_board`. This gives a save latency of 1–1.2 seconds
after the last reorder (1 s debounce + up to 200 ms poll granularity), well
within the spirit of "1 second debounce". No threads, no channels, no async
runtime, no new crates.

**Interaction with other saves**: When a non-reorder key event triggers a save,
set `reorder_save_pending = false` before calling `save_board`. Because
`save_board` writes the full current `BoardState` (including any reorder changes
made since the last debounce-reset), the reorder state is automatically captured.

**Quit flush**: Before `break`ing the event loop (on `q` or `Ctrl+C`), if
`reorder_save_pending` is true, call `save_board` once, then break.

**Pseudocode**:
```
let mut reorder_save_pending = false;
let mut last_reorder_at: Option<Instant> = None;

loop {
    // ... draw frame ...

    if !event::poll(200ms)? {
        // poll timeout — check debounce
        if reorder_save_pending {
            if last_reorder_at.unwrap().elapsed() >= 1s {
                save_board(&mut app.board);
                reorder_save_pending = false;
            }
        }
        continue;
    }

    match event::read()? {
        Key('q') | Key(Ctrl+C) => {
            if reorder_save_pending { save_board(...); }
            break;
        }
        Key('J') | Key('K') in Normal mode => {
            handle_reorder(app, key);
            reorder_save_pending = true;
            last_reorder_at = Some(Instant::now());
            // Do NOT call save_board here
        }
        Key(_) other => {
            handle_key(app, key);
            reorder_save_pending = false;  // cancel debounce
            save_board(...);              // immediate save (includes reorder state)
        }
    }
}
```

**Alternatives considered**:
- `std::thread` with a timer channel — works but adds complexity, requires
  `Arc<Mutex<BoardState>>`, crosses thread boundary. Rejected (Principle II).
- `tokio` async runtime — massive overkill for a local single-user TUI. Rejected.
- Saving on every J/K (current approach generalised) — the original concern:
  excessive I/O on rapid keypresses. Rejected per spec.
