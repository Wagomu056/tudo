# Research: Status-Ordered Task Lists

**Feature**: 011-status-ordered-tasks
**Date**: 2026-03-07

## R1: Data Structure for Per-Status Task Lists

**Decision**: Use a flat `Vec<Task>` internally but maintain per-status ordering invariants, with tasks grouped by status in the Vec. Each status group is contiguous, and within each group the most recently added task is at index 0 of that group.

**Rationale**: Changing to `HashMap<Status, Vec<Task>>` would require significant serialization changes and break the simple JSON array format. Instead, keeping `Vec<Task>` but enforcing ordering (tasks of the same status are contiguous, newest first within each group) achieves the same user-visible behavior with minimal disruption. However, the user explicitly requested "per-status Vec" — so we use `HashMap<Status, Vec<Task>>` as the internal structure.

**Revised Decision**: Use `HashMap<Status, Vec<Task>>` (or equivalent) within `BoardState`. Custom `Serialize`/`Deserialize` implementations flatten this to a single JSON array for backward compatibility.

**Alternatives considered**:
1. **Keep flat Vec with ordering invariants**: Simpler serialization but fragile invariants scattered across many methods. Rejected because invariant maintenance is error-prone.
2. **BTreeMap<Status, Vec<Task>>**: Deterministic iteration order but unnecessary — we always iterate via `ALL_STATUSES` constant. Rejected as over-engineering.
3. **Four named fields (todo_tasks, doing_tasks, etc.)**: Explicit but verbose and doesn't scale to status additions. Rejected.

## R2: Serialization Backward Compatibility

**Decision**: Custom `Serialize` for `BoardState` flattens per-status Vecs into a single `"tasks"` JSON array (ordered by status, then by position within status). Custom `Deserialize` reads the flat array and distributes tasks into per-status Vecs based on each task's `status` field, preserving original order within each status as the default order.

**Rationale**: Existing `current.log` files have `"tasks": [...]` as a flat array. Users must be able to upgrade without data loss. The status field on each task is the source of truth for which list it belongs to.

**Alternatives considered**:
1. **Version bump + migration**: Add version 2 format with per-status arrays. Rejected because it adds complexity and the flat format works fine for persistence.
2. **Separate JSON fields per status**: `"todo_tasks": [], "doing_tasks": []` etc. Rejected because it changes the file format unnecessarily and complicates migration.

## R3: Insertion Position on Status Transition

**Decision**: When a task transitions to a new status (advance, retreat, or creation), it is inserted at index 0 of the destination status's Vec. This makes the most recently transitioned task appear at the top of the column.

**Rationale**: Directly fulfills the spec requirement "一番上には一番最近追加されたタスクが表示されます" (the most recently added task is displayed at the top).

## R4: Focus Tracking After Transition

**Decision**: After a task is moved to a new status and inserted at index 0, focus follows to `(new_status.col_index(), 0)`. The source column's cursor is clamped via `clamp_focus()`.

**Rationale**: Consistent with existing `focus_task_by_id()` behavior. The task is always at index 0 after insertion, simplifying focus tracking.
