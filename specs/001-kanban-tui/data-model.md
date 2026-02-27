# Data Model: tudo Kanban TUI

**Feature**: 001-kanban-tui
**Date**: 2026-02-27

---

## Entities

### Task

The core unit of work on the board.

| Field        | Rust Type                  | Required | Notes                                    |
|--------------|----------------------------|----------|------------------------------------------|
| `id`         | `u64`                      | Yes      | Sequential; unique within BoardState     |
| `title`      | `String`                   | Yes      | Single-line; non-empty after trim        |
| `detail`     | `String`                   | No       | Multiline (raw `\n`); default `""`       |
| `status`     | `Status`                   | Yes      | One of: Todo, Doing, Checking, Done      |
| `created_at` | `DateTime<Local>`          | Yes      | Set at task creation; RFC 3339 on disk   |
| `done_at`    | `Option<DateTime<Local>>`  | No       | Set when status → Done; cleared on revert|

### Status (enum)

```
Todo  →  Doing  →  Checking  →  Done
         ↑                         |
         └──────── BackSpace ───────┘
```

| Value      | Forward key | Backward key |
|------------|-------------|--------------|
| `Todo`     | Enter → Doing | BackSpace: no-op |
| `Doing`    | Enter → Checking | BackSpace → Todo |
| `Checking` | Enter → Done | BackSpace → Doing |
| `Done`     | Enter: no-op | BackSpace → Checking |

When a task transitions **to** Done:
- Set `done_at = Some(Local::now())`
- Append a `DoneEntry` to `YYYYMMDD.log`

When a task transitions **away from** Done (BackSpace):
- Clear `done_at = None`
- No entry is removed from `YYYYMMDD.log` (the log is append-only)

### BoardState

Persisted to `current.log` after every board mutation.

| Field      | Rust Type         | Notes                                     |
|------------|-------------------|-------------------------------------------|
| `version`  | `u32`             | Schema version; current value: `1`        |
| `next_id`  | `u64`             | Counter for next Task ID (monotonic)      |
| `tasks`    | `Vec<Task>`       | All tasks; filter by status for each column |
| `saved_at` | `DateTime<Local>` | Timestamp of last successful save         |

### DoneEntry

Immutable record appended to `YYYYMMDD.log` when a task reaches Done.
One JSON object per line (JSON Lines format).

| Field          | Rust Type          | Notes                          |
|----------------|--------------------|--------------------------------|
| `title`        | `String`           | Task title at time of completion |
| `detail`       | `String`           | Task detail at time of completion |
| `completed_at` | `DateTime<Local>`  | RFC 3339 timestamp             |

---

## Done Column Filtering

On board load:

1. Deserialize `BoardState` from `current.log`.
2. For each task with `status == Done`:
   - If `done_at.map(|d| d.date_naive()) == Some(Local::now().date_naive())`
     → **show** in Done column.
   - Otherwise → **discard** from in-memory state (already logged to a past
     YYYYMMDD.log; no longer needed in the active board).
3. Save the filtered state back to `current.log` (pruning old Done tasks).

This ensures the Done column is automatically empty on the next calendar day
with zero user intervention (SC-005).

---

## Validation Rules

- `Task.title` MUST be non-empty after `trim()`.
- `Task.id` MUST be unique within `BoardState.tasks`.
- `BoardState.next_id` MUST be strictly greater than the maximum `id` among
  all tasks (or `0` if tasks is empty).
- `BoardState.version` MUST equal `1`; if a different version is found on
  disk, log an error and start with an empty board.

---

## File Schemas

### `current.log` — JSON (pretty-printed)

```json
{
  "version": 1,
  "next_id": 4,
  "saved_at": "2026-02-27T10:05:00+09:00",
  "tasks": [
    {
      "id": 1,
      "title": "Write unit tests",
      "detail": "Cover model transitions and storage round-trips",
      "status": "Doing",
      "created_at": "2026-02-27T09:00:00+09:00",
      "done_at": null
    },
    {
      "id": 2,
      "title": "Review PR #42",
      "detail": "",
      "status": "Done",
      "created_at": "2026-02-26T14:00:00+09:00",
      "done_at": "2026-02-27T09:45:00+09:00"
    }
  ]
}
```

### `YYYYMMDD.log` — JSON Lines (one entry per line, append-only)

Example `20260227.log`:

```jsonl
{"title":"Review PR #42","detail":"","completed_at":"2026-02-27T09:45:00+09:00"}
{"title":"Deploy to staging","detail":"Check canary metrics after","completed_at":"2026-02-27T17:30:00+09:00"}
```

---

## AppState (runtime only — not persisted)

| Field            | Type              | Notes                                          |
|------------------|-------------------|------------------------------------------------|
| `board`          | `BoardState`      | Loaded from current.log; mutated during session |
| `focused_col`    | `usize`           | Index 0–3 (Todo=0, Doing=1, Checking=2, Done=3)|
| `focused_card`   | `[usize; 4]`      | Per-column card cursor (index into column list) |
| `mode`           | `AppMode`         | Normal \| InputTitle \| InputDetail            |
| `input`          | `InputState`      | Active only in Input modes                     |
| `status_msg`     | `Option<String>`  | Transient error/info displayed at bottom       |

### AppMode (enum)

| Variant       | Description                                     |
|---------------|-------------------------------------------------|
| `Normal`      | Board navigation; all action keys active        |
| `InputTitle`  | Text input popup open for title (a or e key)    |
| `InputDetail` | Text input popup open for detail (E key)        |

### InputState

| Field       | Type     | Notes                                       |
|-------------|----------|---------------------------------------------|
| `buffer`    | `String` | Current text in the input popup             |
| `is_create` | `bool`   | `true` = creating new task; `false` = editing |
