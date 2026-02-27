# Data Model: Platform Data Directory Storage

**Feature**: 002-xdg-log-dirs
**Date**: 2026-02-28

---

## Entities

### Storage Root (`PathBuf`)

The resolved, OS-specific local data directory for the `tudo` application.

**Resolved at**: startup (inside each storage function call; see research Decision 4)

**Resolution logic**:
1. Try `ProjectDirs::from("", "", "tudo")` → `.data_local_dir()` → append app name if not already included by the crate
2. On failure (None), try `$HOME/.tudo` (Unix) or `$USERPROFILE\.tudo` (Windows)
3. On failure of both, return `AppError::Other` with a descriptive message

**Platform values**:
| Platform | Path |
|----------|------|
| macOS    | `~/Library/Application Support/tudo` |
| Linux    | `~/.local/share/tudo` |
| Windows  | `%LOCALAPPDATA%\tudo` |
| Fallback | `~/.tudo` |

**Directory creation**: `create_dir_all(storage_root)` is called as part of resolution. This is idempotent — safe on first run and on subsequent runs.

---

### Current Board File

**Filename**: `current.log`
**Full path**: `{storage_root}/current.log`
**Format**: Pretty-printed JSON (existing schema, unchanged)
**Access pattern**: Read once at startup; overwritten on every mutation
**Validation**: `version` field must equal `1`

---

### Daily Log File

**Filename**: `{YYYYMMDD}.log` where YYYYMMDD is the local calendar date
**Full path**: `{storage_root}/{YYYYMMDD}.log`
**Format**: JSON Lines (one JSON object per line, append-only)
**Access pattern**: Append-only; one entry per task that reaches Done status
**Content**: `DoneEntry` — title, detail, completed_at timestamp

---

## Unchanged Entities

The following entities are **not modified** by this feature:

- `BoardState` (schema unchanged)
- `Task` (schema unchanged)
- `DoneEntry` (schema unchanged)
- `AppError` variants (no new variant; `Other(String)` is reused)

---

## File → Function Mapping

| File | Function | Change |
|------|----------|--------|
| `current.log` | `load_board()` | Now resolves path via `resolve_data_dir()` |
| `current.log` | `save_board()` | Now resolves path via `resolve_data_dir()` |
| `YYYYMMDD.log` | `append_done_entry()` | Now resolves path via `resolve_data_dir()` |
| Any path | `load_board_from(path)` | **Unchanged** (used by tests) |
| Any path | `save_board_to(board, path)` | **Unchanged** (used by tests) |
| Any path | `append_done_entry_to(entry, path)` | **Unchanged** (used by tests) |

---

## New Internal Function

### `storage::resolve_data_dir() -> Result<PathBuf, AppError>`

**Visibility**: `pub` (needed for tests)
**Side effects**: Creates the data directory if it does not exist
**Returns**: Absolute `PathBuf` to the app data directory, with directory guaranteed to exist
**Errors**: `AppError::Other(String)` if no valid home directory can be found
