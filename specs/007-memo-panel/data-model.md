# Data Model: Memo Panel

**Feature**: 007-memo-panel
**Date**: 2026-03-01

## New Entity: `Memo`

Represents a single memo item — a freeform note with title and optional detail.

| Field    | Type     | Constraints                            | Notes                        |
|----------|----------|----------------------------------------|------------------------------|
| `id`     | `u64`    | Unique within memos; auto-incremented  | Allocated from `next_memo_id`|
| `title`  | `String` | Non-empty after trimming               | Single-line display text     |
| `detail` | `String` | May be empty; `\n`-delimited multiline | Same format as `Task.detail` |

**No status field.** Memos have no lifecycle — they exist until deleted.

**Serialization**: `serde::Serialize + Deserialize`. Stored as JSON objects inside the `memos` array in `current.log`.

```json
{
  "id": 1,
  "title": "API rate limit",
  "detail": "Max 100 req/min per token.\nSee docs section 4."
}
```

---

## Modified Entity: `BoardState`

Two new fields added with `#[serde(default)]` for backward compatibility.

| Field           | Type        | Default | Notes                                      |
|-----------------|-------------|---------|--------------------------------------------|
| `version`       | `u32`       | —       | Remains `1`; no migration needed           |
| `next_id`       | `u64`       | —       | Existing; task ID counter                  |
| `tasks`         | `Vec<Task>` | —       | Existing                                   |
| `saved_at`      | `DateTime`  | —       | Existing                                   |
| `memos`         | `Vec<Memo>` | `[]`    | **NEW** — ordered list of memo items       |
| `next_memo_id`  | `u64`       | `1`     | **NEW** — memo ID allocator                |

**Backward compatibility**: An existing `current.log` without `memos` / `next_memo_id` deserializes to `memos = []` and `next_memo_id = 1`. No version bump.

**Memo ordering**: `memos` is a `Vec` ordered by insertion time (append-only on create). Display order is `memos[0]` left-to-right, `memos[1]`, …

---

## New Enum: `FocusArea`

Tracks whether keyboard focus is in the kanban columns or the memo panel.

```
FocusArea::Kanban   — focus is on one of the 4 kanban columns
FocusArea::Memo     — focus is in the memo panel
```

Stored in `AppState.focus_area`. Default: `FocusArea::Kanban`.

---

## Modified Type: `InputState`

One new boolean field added.

| Field       | Type   | Default | Notes                                                    |
|-------------|--------|---------|----------------------------------------------------------|
| `buffer`    | String | `""`    | Existing                                                 |
| `is_create` | bool   | false   | Existing — true when creating (vs editing)               |
| `is_memo`   | bool   | false   | **NEW** — true when the input target is a memo (not task)|

---

## Modified Type: `AppState`

Three new fields added.

| Field            | Type         | Default          | Notes                                                  |
|------------------|--------------|------------------|--------------------------------------------------------|
| `board`          | BoardState   | —                | Existing                                               |
| `focused_col`    | usize        | 0                | Existing                                               |
| `focused_card`   | [usize; 4]   | [0,0,0,0]        | Existing                                               |
| `mode`           | AppMode      | Normal           | Existing                                               |
| `input`          | InputState   | default          | Existing                                               |
| `status_msg`     | Option<Str>  | None             | Existing                                               |
| `clickable_urls` | Vec<…>       | []               | Existing                                               |
| `focus_area`     | FocusArea    | FocusArea::Kanban| **NEW** — current focus zone                           |
| `focused_memo`   | usize        | 0                | **NEW** — flat index into `board.memos`                |
| `memo_cols`      | usize        | 4                | **NEW** — cached items-per-row from last render frame  |

---

## State Transitions

### Memo lifecycle

```
(none) --[a in MemoFocus]--> Created (appended to board.memos)
Created --[e in MemoFocus]--> Title editing
Created --[E in MemoFocus]--> Detail editing
Created --[D in MemoFocus]--> Deleted (removed from board.memos)
```

Memos have no status transitions (no advance/retreat).

### Focus area transitions

```
FocusArea::Kanban
  --[j at bottom of column]--> FocusArea::Memo (focused_memo clamped to 0)

FocusArea::Memo
  --[k when focused_memo < memo_cols]--> FocusArea::Kanban (focused_col/card preserved)
```

---

## Storage File Impact

Only `current.log` is affected. The daily `YYYYMMDD.log` (append-only done entries) is never written by memo operations — memos are never "done".

### Example `current.log` with memos

```json
{
  "version": 1,
  "next_id": 5,
  "tasks": [ ... ],
  "saved_at": "2026-03-01T10:00:00+09:00",
  "memos": [
    { "id": 1, "title": "Rate limit", "detail": "100 req/min" },
    { "id": 2, "title": "Staging URL", "detail": "" }
  ],
  "next_memo_id": 3
}
```

### Legacy `current.log` (no memo fields)

Deserializes to `memos = []`, `next_memo_id = 1`. No error.

---

## Validation Rules

| Rule | Where enforced |
|------|----------------|
| Memo title must be non-empty after trimming | `confirm_input` (same as task title) |
| `focused_memo` must be `< board.memos.len()` when `memos` non-empty | `clamp_memo_focus` called after any mutation |
| `memo_cols` must be `>= 1` | Set in `render_memo_panel`; floor-clamped to 1 |
