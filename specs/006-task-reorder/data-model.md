# Data Model: Task Reordering Within Columns

**Branch**: `006-task-reorder` | **Date**: 2026-02-28

## Overview

No schema changes. The existing `BoardState.tasks: Vec<Task>` already encodes
column order implicitly. This document clarifies the ordering convention and
the invariants that the new reorder operations must maintain.

---

## Existing Schema (unchanged)

### `BoardState` (persisted as `current.log`)

```json
{
  "version": 1,
  "next_id": 5,
  "saved_at": "2026-02-28T10:00:00+09:00",
  "tasks": [
    { "id": 1, "title": "Task A", "detail": "", "status": "Todo", "created_at": "...", "done_at": null },
    { "id": 2, "title": "Task B", "detail": "", "status": "Todo", "created_at": "...", "done_at": null },
    { "id": 3, "title": "Task C", "detail": "", "status": "Doing", "created_at": "...", "done_at": null },
    { "id": 4, "title": "Task D", "detail": "", "status": "Todo", "created_at": "...", "done_at": null }
  ]
}
```

### Column Order Convention (NEW — documented, not a code change)

The visual order of tasks in a column equals their **relative order** in the
`tasks` array filtered by `status`.

**Example** (from the JSON above):
- Todo column displays: `[Task A (idx 0), Task B (idx 1), Task D (idx 3)]`
- Doing column displays: `[Task C (idx 2)]`

To move Task B down in the Todo column, swap `tasks[1]` and `tasks[3]` in the
Vec. The resulting `tasks` array order is:
`[Task A, Task D, Task C, Task B]`
The Todo column now displays `[Task A, Task D, Task B]`. ✓

---

## `AppState` — new fields (runtime only, not persisted)

| Field | Type | Default | Purpose |
|-------|------|---------|---------|
| `reorder_save_pending` | `bool` | `false` | Whether a debounced reorder save is waiting to fire. Declared in `run_app` (local var), not in `AppState`. |
| `last_reorder_at` | `Option<Instant>` | `None` | Timestamp of the most recent reorder keypress. Declared in `run_app` (local var). |

These are event-loop-local variables in `main.rs::run_app`, not fields on
`AppState` or `BoardState`. They are never persisted.

---

## `AppState` — new methods

### `reorder_task_down(&mut self) -> bool`

- **Pre-condition**: `focused_col` is valid; `focused_card[focused_col]` is a
  valid cursor into the column.
- **Effect**: If the focused task is not last in its column, swaps it with the
  next task (by Vec index). Advances `focused_card[focused_col]` by 1. Returns
  `true` if a swap occurred, `false` (no-op) if already last.
- **Invariants maintained**: All tasks remain in `board.tasks`; no task changes
  `status`; `focused_col` is unchanged; all other column cursors are unchanged.

### `reorder_task_up(&mut self) -> bool`

- **Pre-condition**: Same as above.
- **Effect**: If the focused task is not first in its column, swaps it with the
  previous task. Decrements `focused_card[focused_col]` by 1. Returns `true` if
  a swap occurred, `false` (no-op) if already first.
- **Invariants maintained**: Same as above.

---

## Ordering Invariants

1. Tasks belonging to different columns are never swapped with each other.
2. A task's `status` field is never modified by a reorder operation.
3. The total number of tasks is unchanged after a reorder.
4. `focused_card[col]` always refers to the same task after the swap (it follows
   the moved task to its new position).
5. New tasks appended via `confirm_input` are placed at the end of the Vec,
   which maps to the last position in the Todo column — consistent with
   existing behaviour.
