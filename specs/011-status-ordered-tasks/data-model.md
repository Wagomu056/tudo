# Data Model: Status-Ordered Task Lists

**Feature**: 011-status-ordered-tasks
**Date**: 2026-03-07

## Entity Changes

### BoardState (modified)

**Before**:
```
BoardState {
    version: u32,
    next_id: u64,
    tasks: Vec<Task>,        // flat list, all statuses mixed
    saved_at: DateTime<Local>,
    memos: Vec<Memo>,
    next_memo_id: u64,
}
```

**After**:
```
BoardState {
    version: u32,
    next_id: u64,
    tasks: StatusTaskMap,     // per-status ordered lists
    saved_at: DateTime<Local>,
    memos: Vec<Memo>,
    next_memo_id: u64,
}
```

### StatusTaskMap (new type)

```
StatusTaskMap {
    todo: Vec<Task>,
    doing: Vec<Task>,
    checking: Vec<Task>,
    done: Vec<Task>,
}
```

**Behaviors**:
- `get(status) -> &Vec<Task>`: Returns the task list for a given status.
- `get_mut(status) -> &mut Vec<Task>`: Returns mutable reference to the task list.
- `insert_at_top(status, task)`: Inserts task at index 0 of the given status's list.
- `remove_by_id(status, id) -> Option<Task>`: Removes and returns a task by ID from the given status's list.
- `all_tasks() -> impl Iterator<Item = &Task>`: Iterates all tasks across all statuses (for serialization).
- `from_flat(tasks: Vec<Task>) -> Self`: Constructs from a flat list, distributing tasks by status field, preserving order within each status.

### Task (unchanged)

```
Task {
    id: u64,
    title: String,
    detail: String,
    status: Status,
    created_at: DateTime<Local>,
    done_at: Option<DateTime<Local>>,
}
```

The `status` field on `Task` remains the source of truth and must always match the list it belongs to.

## Serialization

### JSON Format (unchanged for backward compatibility)

```json
{
    "version": 1,
    "next_id": 5,
    "tasks": [
        {"id": 1, "title": "...", "status": "Todo", ...},
        {"id": 2, "title": "...", "status": "Doing", ...}
    ],
    "saved_at": "...",
    "memos": [...],
    "next_memo_id": 1
}
```

The `tasks` field serializes as a flat JSON array (all tasks from all statuses concatenated). On deserialization, tasks are distributed into per-status Vecs based on each task's `status` field.

## State Transitions

### Task Creation
1. Create `Task` with `status: Todo`
2. `StatusTaskMap::insert_at_top(Todo, task)`
3. Focus -> `(Todo.col_index(), 0)`

### Advance Status (e.g., Todo -> Doing)
1. `StatusTaskMap::remove_by_id(old_status, id)` -> task
2. Set `task.status = new_status`
3. `StatusTaskMap::insert_at_top(new_status, task)`
4. Focus -> `(new_status.col_index(), 0)`

### Retreat Status (e.g., Doing -> Todo)
1. `StatusTaskMap::remove_by_id(old_status, id)` -> task
2. Set `task.status = prev_status`
3. `StatusTaskMap::insert_at_top(prev_status, task)`
4. Focus -> `(prev_status.col_index(), 0)`

### Delete Task
1. `StatusTaskMap::remove_by_id(status, id)`
2. `clamp_focus()`

### Reorder Within Column
1. `StatusTaskMap::get_mut(status).swap(cursor, cursor +/- 1)`
2. Update focus cursor

## Migration

No explicit migration needed. The custom `Deserialize` implementation reads the flat `"tasks"` array and calls `StatusTaskMap::from_flat()` to distribute tasks. Existing files work transparently.
