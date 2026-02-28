# Implementation Plan: Keep Task Focus on Status Change

**Branch**: `003-keep-task-focus` | **Date**: 2026-02-28 | **Spec**: [spec.md](spec.md)
**Input**: Feature specification from `/specs/003-keep-task-focus/spec.md`

## Summary

After pressing Enter or BackSpace to change a task's status in the kanban TUI, the visual focus/highlight must remain on that specific task in its new position. Currently, `focused_col` is never updated after a status change, so focus drifts to an unrelated task. The fix adds a `focus_task_by_id(id)` method to `AppState` that sets both `focused_col` and `focused_card` to the task's new column and position after each mutation.

## Technical Context

**Language/Version**: Rust stable ≥ 1.75 (via rustup)
**Primary Dependencies**: ratatui 0.29 (crossterm backend), serde + serde_json 1.0, chrono 0.4
**Storage**: JSON files (`current.log`, `YYYYMMDD.log`) — unchanged by this feature
**Testing**: Rust built-in test framework (`cargo test`); unit tests in `src/app.rs`
**Target Platform**: macOS / Linux terminal emulators (≥ 80 columns)
**Project Type**: TUI desktop application
**Performance Goals**: ≤ 16 ms per frame (60 fps) — this change is O(n) in task count, negligible
**Constraints**: No `unwrap()` in production paths; no new crate dependencies
**Scale/Scope**: Single-user local tool; task count expected in the tens to low hundreds

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Status | Notes |
|-----------|--------|-------|
| I. Test-First | PASS | Tests will be written before implementation code. Unit tests for `focus_task_by_id`, `advance_status`, and `retreat_status` will be written first, confirmed red, then implementation added. |
| II. Simplicity / YAGNI | PASS | One new private method (`focus_task_by_id`), two call-site changes. No new abstractions, no new crates, no speculative future-proofing. |
| III. TUI-First | PASS | Pure focus-tracking state change; no I/O on the UI thread. Executes in microseconds; frame budget unaffected. |
| IV. Data Portability | PASS | No data model changes. Storage format unchanged. |
| V. Correctness Over Performance | PASS | No `unwrap()` introduced. Task lookup uses `Option` pattern. Borrow checker constraints explicitly addressed in research.md. |

**Post-Design Re-check**: All gates still pass. Implementation is a localized mutation to `src/app.rs` only.

## Project Structure

### Documentation (this feature)

```text
specs/003-keep-task-focus/
├── plan.md          # This file
├── research.md      # Phase 0 output
├── spec.md          # Feature specification
├── checklists/
│   └── requirements.md
└── tasks.md         # Phase 2 output (/speckit.tasks — NOT created by /speckit.plan)
```

### Source Code (repository root)

```text
src/
├── app.rs           # MODIFIED: add focus_task_by_id(); update advance_status, retreat_status
├── main.rs          # Unchanged
├── model.rs         # Unchanged (Status::col_index already exists)
├── ui.rs            # Unchanged
├── storage.rs       # Unchanged
└── lib.rs           # Unchanged

tests/               # Unchanged (existing integration tests unaffected)
```

**Structure Decision**: Single project (Option 1). Only `src/app.rs` is modified. Unit tests are added as `#[cfg(test)]` modules inside `src/app.rs`, consistent with the existing test pattern in the codebase.

## Implementation Design

### New Method: `focus_task_by_id`

Added to `impl AppState` in `src/app.rs` as a private method:

```rust
fn focus_task_by_id(&mut self, id: u64) {
    let status = match self.board.tasks.iter().find(|t| t.id == id) {
        Some(task) => task.status,
        None => return,
    };
    let col = status.col_index();
    let pos = {
        let col_tasks = self.tasks_for_column(status);
        col_tasks.iter().position(|t| t.id == id).unwrap_or(0)
    };
    self.focused_col = col;
    self.focused_card[col] = pos;
}
```

**Why `unwrap_or(0)` is acceptable here**: The task was found in `self.board.tasks` immediately above, so it will always appear in `tasks_for_column(status)`. The `unwrap_or(0)` is a defensive fallback for an unreachable branch — no production panic risk.

### Updated `advance_status`

Replace trailing `self.clamp_focus()` with `self.focus_task_by_id(id)` followed by `self.clamp_focus()`:

```rust
pub fn advance_status(&mut self) {
    let id = match self.focused_task_id() {
        Some(id) => id,
        None => return,
    };
    if let Some(task) = self.board.tasks.iter_mut().find(|t| t.id == id) {
        if let Some(next) = task.status.next() {
            task.status = next;
            if next == Status::Done {
                task.done_at = Some(Local::now());
            }
        }
    }
    self.focus_task_by_id(id);
    self.clamp_focus();
}
```

### Updated `retreat_status`

Same pattern:

```rust
pub fn retreat_status(&mut self) {
    let id = match self.focused_task_id() {
        Some(id) => id,
        None => return,
    };
    if let Some(task) = self.board.tasks.iter_mut().find(|t| t.id == id) {
        if let Some(prev) = task.status.prev() {
            task.status = prev;
            if prev != Status::Done {
                task.done_at = None;
            }
        }
    }
    self.focus_task_by_id(id);
    self.clamp_focus();
}
```

### Why Call Both `focus_task_by_id` and `clamp_focus`

- `focus_task_by_id` sets the destination column and position correctly for the moved task
- `clamp_focus` subsequently corrects the cursor for the **source column** (which lost a task and may need its cursor adjusted)
- The two calls are complementary, not redundant

## Test Plan

All tests are written before implementation code (Constitution Principle I).

### Unit Tests (inside `src/app.rs` `#[cfg(test)]` block)

| Test | Scenario | Expected |
|------|----------|----------|
| `test_advance_status_keeps_focus_on_task` | Task in Doing column; press Enter | `focused_col` = Checking, `focused_card[Checking]` = task's index in Checking |
| `test_retreat_status_keeps_focus_on_task` | Task in Doing column; press BackSpace | `focused_col` = Todo, `focused_card[Todo]` = task's index in Todo |
| `test_advance_status_at_done_boundary` | Task already at Done; press Enter | Status unchanged, focus unchanged, no panic |
| `test_retreat_status_at_todo_boundary` | Task already at Todo; press BackSpace | Status unchanged, focus unchanged, no panic |
| `test_focus_follows_task_across_multiple_advances` | Task advanced through all statuses one step at a time | Focus correctly tracks task through Todo→Doing→Checking→Done |
| `test_source_column_cursor_clamped_after_status_change` | Only task in a column is moved out; press Enter | Source column cursor clamped to 0; destination column focused on moved task |

## Phases Summary

### Phase 0: Research — COMPLETE

See `research.md`. All unknowns resolved. No NEEDS CLARIFICATION markers remain.

### Phase 1: Design — COMPLETE (this document)

- No external interface contracts needed (purely internal state change)
- No data model changes
- No new dependencies

### Phase 2: Tasks — NOT YET CREATED

Run `/speckit.tasks` to generate `tasks.md`.
