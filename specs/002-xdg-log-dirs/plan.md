# Implementation Plan: Store Log Files in Platform Data Directory

**Branch**: `002-xdg-log-dirs` | **Date**: 2026-02-28 | **Spec**: [spec.md](spec.md)
**Input**: Feature specification from `specs/002-xdg-log-dirs/spec.md`

## Summary

Add a `resolve_data_dir()` function to `storage.rs` that uses the `directories` crate to resolve the OS-appropriate local data directory (`~/Library/Application Support/tudo` on macOS, `~/.local/share/tudo` on Linux, `%LOCALAPPDATA%\tudo` on Windows). Update the three high-level storage convenience functions — `load_board()`, `save_board()`, and `append_done_entry()` — to resolve their file paths through this function instead of using the current working directory. The low-level `_from` / `_to` variants remain unchanged. No schema changes, no UI changes.

## Technical Context

**Language/Version**: Rust stable ≥ 1.75 (via rustup)
**Primary Dependencies**: ratatui 0.29 (crossterm), serde + serde_json 1.0, chrono 0.4; adding `directories = "5"`
**Storage**: Local files — `current.log` (JSON), `YYYYMMDD.log` (JSON Lines)
**Testing**: `cargo test`; integration tests in `tests/`
**Target Platform**: macOS, Linux (primary); Windows (secondary)
**Project Type**: TUI desktop application (CLI)
**Performance Goals**: Directory resolution < 1 ms (env var reads only); no perceptible impact on save latency
**Constraints**: Must never write to the current working directory; zero breaking changes to low-level storage API
**Scale/Scope**: Single user, local machine only; 2 files affected

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Status | Notes |
|-----------|--------|-------|
| **I. Test-First** | ✅ Pass | New tests for `resolve_data_dir()` and updated convenience functions must be written before implementation code. Tasks order: write failing tests first, then implement. |
| **II. Simplicity / YAGNI** | ✅ Pass | One new function, one new dependency, three modified function bodies. No new abstractions. `AppError::Other` reused — no new variant. Low-level `_from`/`_to` functions unchanged. |
| **III. TUI-First** | ✅ Pass | No UI changes. `resolve_data_dir()` is called on the existing save path (post-keypress) — same thread, same timing as current saves. Directory resolution is env-var reads only (no blocking I/O during rendering). |
| **IV. Data Portability** | ✅ Pass | File formats (JSON, JSON Lines) are unchanged. No schema changes. The `version` field is unaffected. |
| **V. Correctness Over Performance** | ✅ Pass | `resolve_data_dir()` returns `Result` — no `unwrap()`. Error propagated via `?`. `AppError::Other(String)` provides a descriptive message. `create_dir_all` failure propagates as `AppError::Io`. |

**Post-design re-check**: No new violations introduced. Complexity Tracking table not required — no violations to justify.

## Project Structure

### Documentation (this feature)

```text
specs/002-xdg-log-dirs/
├── plan.md              ← this file
├── research.md          ✅ Phase 0 complete
├── data-model.md        ✅ Phase 1 complete
├── quickstart.md        ✅ Phase 1 complete
├── checklists/
│   └── requirements.md
└── tasks.md             ← Phase 2 output (/speckit.tasks — not yet created)
```

### Source Code (repository root)

```text
src/
├── lib.rs         (unchanged)
├── main.rs        (unchanged)
├── app.rs         (unchanged)
├── input.rs       (unchanged)
├── model.rs       (unchanged — AppError::Other reused)
├── storage.rs     ← add resolve_data_dir(); modify load_board(), save_board(), append_done_entry()
└── ui.rs          (unchanged)

tests/
├── model_tests.rs       (unchanged)
└── storage_tests.rs     ← add tests for resolve_data_dir() and dir-aware functions

Cargo.toml             ← add directories = "5"
```

**Structure Decision**: Single-project layout (Option 1). Only `storage.rs`, `storage_tests.rs`, and `Cargo.toml` require changes.

## Phase 0: Research

**Status**: ✅ Complete — see [research.md](research.md)

Key decisions resolved:
- Crate: `directories = "5"`, API: `ProjectDirs::from("", "", "tudo").data_local_dir()`
- Fallback: `$HOME/.tudo` via `std::env::var("HOME")` if `ProjectDirs` returns `None`
- Resolution timing: per-call (no caching); `create_dir_all` called inside `resolve_data_dir()`
- Error type: reuse `AppError::Other(String)`

## Phase 1: Design & Contracts

**Status**: ✅ Complete

### New Function: `storage::resolve_data_dir`

```
Signature: pub fn resolve_data_dir() -> Result<PathBuf, AppError>

Steps:
  1. Call ProjectDirs::from("", "", "tudo")
  2. If Some(dirs): use dirs.data_local_dir().to_path_buf()
  3. If None:
     a. Try std::env::var("HOME") → PathBuf::from(home).join(".tudo")
     b. On Windows, also try std::env::var("USERPROFILE")
     c. If all fail: return Err(AppError::Other("cannot determine application data directory ..."))
  4. Call fs::create_dir_all(&path)?
  5. Return Ok(path)
```

### Modified Functions

```
load_board():
  - Call resolve_data_dir()?
  - Call load_board_from(data_dir.join("current.log").to_str()...)

save_board(board):
  - Call resolve_data_dir()?
  - Call save_board_to(board, data_dir.join("current.log").to_str()...)

append_done_entry(entry):
  - Call resolve_data_dir()?
  - Build daily filename: format!("{}.log", chrono::Local::now().format("%Y%m%d"))
  - Call append_done_entry_to(entry, &data_dir.join(filename))
```

### Unchanged Functions (test surface)

```
load_board_from(path: &str) → Result<BoardState, AppError>   — no change
save_board_to(board, path: &str) → Result<(), AppError>      — no change
append_done_entry_to(entry, path: &Path) → Result<(), AppError> — no change
```

### Contracts

No external contracts — this is a purely internal change to file path resolution. The public API surface visible to `main.rs` is unchanged in signature.

### Agent Context

Updated via `.specify/scripts/bash/update-agent-context.sh claude` — see CLAUDE.md.
