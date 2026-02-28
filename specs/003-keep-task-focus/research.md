# Research: Keep Task Focus on Status Change

**Branch**: `003-keep-task-focus` | **Date**: 2026-02-28

## Current Focus Management Behavior

### How Focus Works Today

`AppState` (in `src/app.rs`) tracks focus with two fields:

- `focused_col: usize` — which column (0=Todo, 1=Doing, 2=Checking, 3=Done) is active
- `focused_card: [usize; 4]` — per-column cursor, storing the positional index of the selected task within each column's filtered task list

When the user presses Enter or BackSpace, the call chain is:

1. `advance_status()` / `retreat_status()` — mutates `task.status` to move it to the next/prev status
2. `clamp_focus()` — clamps `focused_card[col]` to stay within the now-updated column length

### The Bug

After a status change, `focused_col` is **never updated**. It stays pointing at the original column. On the next render frame, `tasks_for_column(original_status)` no longer includes the moved task, so the highlight lands on whatever task happens to be at the same cursor index in the original column (or nothing if the column is now empty).

**Example**: User is on "Doing" column, cursor at index 0 (Task A). Presses Enter. Task A moves to "Checking". `focused_col` is still 1 (Doing). On next render, Doing column shows the task that is now at index 0 in Doing — a completely different task. Task A has no focus highlight.

### What Needs to Change

After mutating a task's status:

1. Capture the task's `id` before mutation (already done in `advance_status`)
2. After mutation, look up the task by `id` to find its new `status`
3. Use `Status::col_index()` to get the new column index
4. Find the task's position within the new column's filtered list
5. Set `focused_col` to the new column
6. Set `focused_card[new_col]` to the task's position

## Decision Log

### Decision 1: Where to implement focus-following logic

- **Decision**: Add a new private method `focus_task_by_id(id: u64)` to `AppState`
- **Rationale**: Keeps the logic reusable between `advance_status` and `retreat_status`. Each method already captures the task id before mutation; calling `self.focus_task_by_id(id)` after mutation is a clean, minimal extension.
- **Alternatives considered**:
  - Inline logic directly in each method: rejected because it duplicates code across two methods
  - Compute new focus in `clamp_focus()`: rejected because `clamp_focus` is semantics-only for clamping bounds, not for following a specific task

### Decision 2: Relationship between `focus_task_by_id` and `clamp_focus`

- **Decision**: `focus_task_by_id` replaces the call to `clamp_focus` inside `advance_status` and `retreat_status`. `focus_task_by_id` internally sets an explicit position rather than clamping.
- **Rationale**: If the task exists (it just had its status changed so it must exist), we can compute its exact position. `clamp_focus` is still valid as a standalone helper for other navigation paths.
- **Alternatives considered**:
  - Call both `clamp_focus` then `focus_task_by_id`: redundant; `focus_task_by_id` sets the values correctly already

### Decision 3: Boundary case (no status change possible)

- **Decision**: When `task.status.next()` or `task.status.prev()` returns `None`, the status is unchanged and the task stays in the same column. No focus update needed in this case — the cursor is already on the correct task.
- **Rationale**: The existing code already returns early (the mutation block is skipped), and `clamp_focus` still runs to handle any edge. No change needed for this path.

### Decision 4: Empty column edge case

- **Decision**: Handled by the existing `clamp_focus()` call at the end of `focus_task_by_id`. If `focused_card[old_col]` is now out-of-bounds after the task left, clamping corrects it. For `focus_task_by_id` itself, the moved task is always found by `id`, so no empty-column issue occurs for the destination.
- **Rationale**: We focus the specific task by id in its new column, so the destination column always has at least one task (the one we just moved). The source column cursor is managed by `clamp_focus`.

## Source Files Affected

| File | Change |
|------|--------|
| `src/app.rs` | Add `focus_task_by_id(id: u64)` method; replace `clamp_focus()` call in `advance_status` and `retreat_status` with `focus_task_by_id(id)` followed by `clamp_focus()` on remaining cols |

No other files require changes. The render logic (`ui.rs`), data model (`model.rs`), event handling (`main.rs`), and storage (`storage.rs`) are unaffected.

## Borrow Checker Strategy

`focus_task_by_id` needs to:
1. Immutably borrow `self.board.tasks` to find the task's new status
2. Immutably borrow `self` again to call `tasks_for_column()`
3. Mutably assign to `self.focused_col` and `self.focused_card`

This is safe because the immutable borrows end (via NLL) before the mutable assignments. Structure:

```
let status = { ... find task status ... };  // immutable borrow ends
let col = status.col_index();
let pos = { let v = self.tasks_for_column(status); ... };  // immutable borrow ends
self.focused_col = col;   // mutable assignment — no active borrows
self.focused_card[col] = pos;
```
