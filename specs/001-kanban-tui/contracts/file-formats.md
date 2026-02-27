# File Format Contract: tudo Kanban TUI

**Feature**: 001-kanban-tui
**Date**: 2026-02-27

This document is the authoritative specification for all on-disk file formats.
Implementation MUST produce and consume files exactly as described here.

---

## `current.log` — Board State

**Location**: Current working directory (`./current.log`)
**Format**: JSON (pretty-printed, UTF-8, LF line endings)
**Lifecycle**: Created on first save; overwritten on every board mutation

### Schema (version 1)

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
    }
  ]
}
```

### Field Constraints

| Field              | Type              | Constraints                                   |
|--------------------|-------------------|-----------------------------------------------|
| `version`          | integer           | MUST equal `1`                                |
| `next_id`          | u64               | MUST be > max task id (or 0 if tasks empty)   |
| `saved_at`         | RFC 3339 string   | Local timezone offset included                |
| `tasks[].id`       | u64               | Unique, non-zero                              |
| `tasks[].title`    | string            | Non-empty after trim                          |
| `tasks[].detail`   | string            | May be empty string `""`                      |
| `tasks[].status`   | string enum       | One of: `"Todo"`, `"Doing"`, `"Checking"`, `"Done"` |
| `tasks[].created_at` | RFC 3339 string | Local timezone offset included                |
| `tasks[].done_at`  | RFC 3339 or null  | `null` unless status is `"Done"`              |

### Error Handling on Load

- File does not exist → start with empty board (no error).
- File exists but cannot be read → display error, start empty.
- File is valid JSON but `version != 1` → display version mismatch error, start empty.
- File is malformed JSON → display parse error, start empty.

---

## `YYYYMMDD.log` — Daily Done Log

**Location**: Current working directory (e.g., `./20260227.log`)
**Format**: JSON Lines (UTF-8; each line is one complete JSON object; LF terminated)
**Lifecycle**: Appended to (never overwritten) each time a card reaches Done

### Line Schema

```json
{"title":"Review PR #42","detail":"","completed_at":"2026-02-27T09:45:00+09:00"}
```

| Field          | Type            | Constraints                          |
|----------------|-----------------|--------------------------------------|
| `title`        | string          | Task title at time of completion     |
| `detail`       | string          | Task detail at time of completion    |
| `completed_at` | RFC 3339 string | Local timezone; set at time of Done  |

### Write Protocol

1. Open file with `append = true`, `create = true`.
2. Serialize `DoneEntry` as compact (non-pretty) JSON.
3. Write the JSON string followed by a single `\n`.
4. Flush and close.

If the file cannot be opened or written, surface a user-visible error in the
status bar; do not silently swallow the failure.

### Filename Convention

| Pattern   | Meaning                           |
|-----------|-----------------------------------|
| `%Y%m%d`  | `chrono::Local::now().format("%Y%m%d")` |
| Example   | `20260227.log`                    |

---

## Timestamp Format

All timestamps stored in RFC 3339 format with local timezone offset:

```
2026-02-27T10:05:00+09:00
```

chrono serializes `DateTime<Local>` in this format automatically when the
`serde` feature is enabled.

---

## Schema Versioning

The `version` field in `current.log` is reserved for forward-compatible
migration. Rules:

- Current version: `1`
- A future MAJOR schema change MUST increment the version integer.
- The loader MUST reject (with a user-visible error) any file with an
  unrecognized version.
- Old `YYYYMMDD.log` files are never rewritten; their format is implicitly
  versioned by the date of the `tudo` binary that produced them.
