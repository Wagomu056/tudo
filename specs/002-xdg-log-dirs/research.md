# Research: Platform Data Directory Storage

**Feature**: 002-xdg-log-dirs
**Date**: 2026-02-28
**Status**: Complete — all unknowns resolved

---

## Decision 1: Which crate to use for platform directory resolution

**Decision**: `directories` crate v5.0

**Rationale**:
- The `directories` crate is the most widely used, actively maintained solution for cross-platform application directory resolution in Rust.
- It covers macOS, Linux (XDG spec), and Windows with a single consistent API.
- License: MIT/Apache-2.0 — compatible with project requirements.
- No transitive dependencies; compile-time impact is negligible.
- The project constitution requires evaluating new crates for maintenance status, license, and compile-time impact — `directories` passes all three.

**Alternatives considered**:
- `dirs` crate: simpler API, but fewer project-specific helpers. Would work but `directories` is idiomatic for named applications.
- `xdg` crate: Linux/XDG only; doesn't support macOS or Windows.
- Manual `$HOME` env var reads: fragile, does not follow Windows conventions, re-invents solved problem.

---

## Decision 2: Which ProjectDirs method to call

**Decision**: `ProjectDirs::from("", "", "tudo")` → `.data_local_dir()`

**Rationale**:
- `from("", "", "tudo")` is idiomatic for open-source CLI apps without a formal organization domain.
- `data_local_dir()` returns local (non-synced) storage — correct for runtime data files like task state and completion logs.
- On macOS and Linux, `data_local_dir()` and `data_dir()` (roaming) are identical; on Windows, `data_local_dir()` uses `%LOCALAPPDATA%` (not synced to other machines), which is more appropriate for local task state.

**Resolved paths by platform**:

| Platform | Resolved Path |
|----------|--------------|
| macOS    | `~/Library/Application Support/tudo` |
| Linux    | `~/.local/share/tudo` (or `$XDG_DATA_HOME/tudo`) |
| Windows  | `%LOCALAPPDATA%\tudo` |

---

## Decision 3: Fallback when `ProjectDirs::from()` returns `None`

**Decision**: Fall back to `$HOME/.tudo` (via `std::env::var("HOME")`); on Windows use `$USERPROFILE\.tudo`. Return `AppError::Other(...)` if neither is available.

**Rationale**:
- `ProjectDirs::from()` returns `None` only in pathological environments (no valid home directory, broken environment variables). This should virtually never happen in practice.
- A predictable `~/.tudo` fallback ensures users on unusual setups can still run the app and find their data.
- The working directory must never be used as a fallback — this is the core requirement of the feature.
- `std::env::home_dir()` is deprecated in std; using `std::env::var("HOME")` directly is more explicit and avoids the deprecation warning on stable Rust.

**Alternatives considered**:
- `home` crate (maintained by Cargo team): would be more robust for Windows edge cases but adds a dependency. Rejected per YAGNI — the fallback path is for pathological cases only.
- Returning a hard error immediately if `ProjectDirs` is `None`: acceptable, but providing a fallback path is more user-friendly.

---

## Decision 4: When to resolve the data directory

**Decision**: Resolve inside each storage function (`load_board`, `save_board`, `append_done_entry`) as needed. Do not cache at module level.

**Rationale**:
- `ProjectDirs::from()` is pure environment variable reads — cheap, no filesystem I/O.
- `create_dir_all` on an already-existing directory is a no-op (safe, idempotent, single fast syscall).
- Caching (e.g., `OnceLock<PathBuf>`) adds complexity (Principle II: YAGNI/Simplicity). Three simple calls are preferable to a premature abstraction.
- The low-level `load_board_from(path)` / `save_board_to(board, path)` / `append_done_entry_to(entry, path)` functions remain unchanged and are used by tests directly — no test infrastructure changes needed.

---

## Decision 5: AppError variant for directory resolution failure

**Decision**: Reuse the existing `AppError::Other(String)` variant with a descriptive message.

**Rationale**:
- A new `DataDirNotFound` variant would be a new abstraction for an error that is essentially "a string error about a missing path." `AppError::Other` already serves this purpose.
- Principle II (YAGNI): do not add a new enum variant if the existing one suffices.
- The message will include the expected path so users can diagnose and act.
