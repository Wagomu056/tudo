# Tasks: Store Log Files in Platform Data Directory

**Input**: Design documents from `specs/002-xdg-log-dirs/`
**Prerequisites**: plan.md ✅, spec.md ✅, research.md ✅, data-model.md ✅, quickstart.md ✅

**Tests**: Included — Constitution Principle I (Test-First) is NON-NEGOTIABLE. All tests must be written and confirmed failing before the implementation tasks that make them pass.

**Organization**: Tasks are grouped by user story to enable independent implementation and testing.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files / no dependency on incomplete tasks)
- **[Story]**: User story this task belongs to ([US1], [US2])
- Exact file paths are included in every description

## Path Conventions

Single-project layout — `src/` and `tests/` at repository root.

---

## Phase 1: Setup

**Purpose**: Add the new dependency required by `resolve_data_dir()`.

- [x] T001 Add `directories = "5"` to the `[dependencies]` table in `Cargo.toml`

---

## Phase 2: Foundational — `resolve_data_dir()`

**Purpose**: Implement the core directory-resolution function that both user stories depend on. This MUST be complete before any user story work begins.

**⚠️ CRITICAL**: Write and confirm all tests fail before implementing T003.

> **NOTE: Write T002 tests FIRST, confirm they FAIL, then implement T003**

- [x] T002 Write 3 failing unit tests for `resolve_data_dir()` in `tests/storage_tests.rs`:
  1. `resolve_data_dir_returns_ok` — call `storage::resolve_data_dir()`, assert `result.is_ok()`
  2. `resolve_data_dir_creates_directory` — call `resolve_data_dir()`, assert the returned `PathBuf` points to an existing directory on disk
  3. `resolve_data_dir_path_ends_with_tudo` — call `resolve_data_dir()`, assert the returned path's last component equals `"tudo"`

- [x] T003 Implement `pub fn resolve_data_dir() -> Result<PathBuf, AppError>` in `src/storage.rs`:
  - Use `directories::ProjectDirs::from("", "", "tudo")` → `.data_local_dir().to_path_buf()` as the primary path
  - If `ProjectDirs::from()` returns `None`: fall back to `std::env::var("HOME").ok()` (Unix) or `std::env::var("USERPROFILE").ok()` (Windows) → append `".tudo"`; if both fail, return `Err(AppError::Other("cannot determine application data directory".to_string()))`
  - Call `fs::create_dir_all(&path)?` before returning
  - Return `Ok(path)`
  - Add `use directories::ProjectDirs;` and `use std::path::PathBuf;` imports

**Checkpoint**: `cargo test` passes T002 tests. `resolve_data_dir()` compiles and resolves the correct platform path.

---

## Phase 3: User Story 1 — No CWD Pollution (Priority: P1) 🎯 MVP

**Goal**: `current.log` and `YYYYMMDD.log` are never written to the current working directory. All three high-level convenience functions route through `resolve_data_dir()`.

**Independent Test**: `cargo test` passes T004 tests; after running the app from `/tmp`, no `.log` files appear in `/tmp`.

> **NOTE: Write T004 tests FIRST, confirm they FAIL, then implement T005–T007**

- [x] T004 Write 3 failing integration tests in `tests/storage_tests.rs`:
  1. `save_board_writes_to_data_dir_not_cwd` — call `storage::save_board(&mut BoardState::default())`, then:
     - assert `Path::new("current.log").exists()` is `false` (not in CWD)
     - assert `storage::resolve_data_dir().unwrap().join("current.log").exists()` is `true`
  2. `load_board_reads_from_data_dir` — write valid board JSON to `resolve_data_dir().unwrap().join("current.log")`; call `storage::load_board()`; assert `result.is_ok()` and board version equals `1`
  3. `append_done_entry_writes_to_data_dir_not_cwd` — create a `DoneEntry`, call `storage::append_done_entry(&entry)`, then:
     - assert no file matching today's date pattern (`YYYYMMDD.log`) exists in CWD
     - assert `resolve_data_dir().unwrap().join(format!("{}.log", Local::now().format("%Y%m%d"))).exists()` is `true`

- [x] T005 [P] [US1] Update `load_board()` in `src/storage.rs`:
  - Replace `load_board_from(CURRENT_LOG)` with:
    ```
    let data_dir = resolve_data_dir()?;
    load_board_from(data_dir.join("current.log").to_str().unwrap_or("current.log"))
    ```
  - Note: `.to_str().unwrap_or("current.log")` is acceptable only because `unwrap_or` provides a safe fallback — prefer returning an error if the path is not valid UTF-8 for correctness

- [x] T006 [P] [US1] Update `save_board()` in `src/storage.rs`:
  - Replace `save_board_to(board, CURRENT_LOG)` with:
    ```
    let data_dir = resolve_data_dir()?;
    save_board_to(board, data_dir.join("current.log").to_str().unwrap_or("current.log"))
    ```

- [x] T007 [P] [US1] Update `append_done_entry()` in `src/storage.rs`:
  - Replace the current-directory filename construction with:
    ```
    let data_dir = resolve_data_dir()?;
    let filename = format!("{}.log", chrono::Local::now().format("%Y%m%d"));
    append_done_entry_to(entry, &data_dir.join(filename))
    ```
  - Remove the `CURRENT_LOG` constant if it is now unused, or keep it as a filename-only constant (without path meaning) per YAGNI

**Checkpoint**: `cargo test` — all T004 tests pass. Build from any directory; no `.log` files appear in CWD.

---

## Phase 4: User Story 2 — Consistent Location (Priority: P2)

**Goal**: Regardless of the directory from which the app is launched, all data files land in one predictable location and previously saved data is always found.

**Independent Test**: Pre-seed `resolve_data_dir()/current.log` with a known board state; call `load_board()` without any changes to CWD; assert the expected board state is returned.

> **NOTE: Write T008 tests FIRST. These tests already pass after Phase 3 — confirm they pass without new implementation.**

- [x] T008 [US2] Write 2 tests in `tests/storage_tests.rs` that verify consistent-location behaviour:
  1. `save_and_load_round_trip_via_data_dir` — call `save_board(&mut board_with_tasks)`, then `load_board()`; assert loaded board matches saved board (title, status, next_id). This proves a "second session" finds data written by the "first session".
  2. `resolve_data_dir_returns_same_path_on_repeated_calls` — call `resolve_data_dir()` twice in the same test, assert both return `Ok` and the two paths are equal. This proves the directory is stable.

**Checkpoint**: `cargo test` — all US2 tests pass without any implementation changes beyond Phase 3.

---

## Phase 5: Polish & Cross-Cutting Concerns

**Purpose**: Ensure code quality gates pass and manual verification matches expected behaviour.

- [x] T009 Run `cargo test` and confirm all tests pass; run `cargo clippy -- -D warnings` and fix any warnings introduced by the new `directories` import or `resolve_data_dir()` implementation in `src/storage.rs`

- [x] T010 [P] Follow the manual verification steps in `specs/002-xdg-log-dirs/quickstart.md`: build and run from `/tmp`, interact with the app, quit, and confirm no `.log` files exist in `/tmp`; confirm `current.log` and `YYYYMMDD.log` exist in the platform data directory

---

## Dependencies & Execution Order

### Phase Dependencies

- **Phase 1 (Setup)**: No dependencies — start immediately
- **Phase 2 (Foundational)**: Depends on Phase 1 (T001 must add the `directories` crate before T003 compiles)
- **Phase 3 (US1)**: Depends on Phase 2 complete — `resolve_data_dir()` must exist before updating callers
- **Phase 4 (US2)**: Depends on Phase 3 complete — consistency tests verify US1 implementation
- **Phase 5 (Polish)**: Depends on all user stories complete

### User Story Dependencies

- **US1 (P1)**: Depends only on Phase 2 (Foundational) — no dependency on US2
- **US2 (P2)**: Depends on US1 completion — shares the same implementation; only adds verification tests

### Within Each Phase

- **T002 before T003**: Tests must be written and confirmed failing before implementation
- **T004 before T005/T006/T007**: Tests must be written and confirmed failing before implementation
- **T005, T006, T007**: All [P] — each modifies a different function body in `storage.rs`; can be executed in parallel
- **T008 before T009**: Tests written before verification

### Parallel Opportunities

- T005, T006, T007 are independently modifiable (different function bodies, no shared state) — can be worked in parallel if desired

---

## Parallel Example: User Story 1

After T004 tests are confirmed failing, launch T005, T006, T007 in parallel:

```
Task T005: Update load_board() in src/storage.rs
Task T006: Update save_board() in src/storage.rs
Task T007: Update append_done_entry() in src/storage.rs
```

Each modifies a distinct function; merge after all three pass T004 tests.

---

## Implementation Strategy

### MVP (User Story 1 Only)

1. Complete Phase 1: Add `directories` dependency
2. Complete Phase 2: Implement `resolve_data_dir()` (test-first)
3. Complete Phase 3: Update three convenience functions (test-first)
4. **STOP and VALIDATE**: `cargo test` passes; manual verification shows no CWD pollution
5. Ship as MVP

### Incremental Delivery

1. Phase 1 + Phase 2 → `resolve_data_dir()` complete and tested
2. Phase 3 → US1 complete: no CWD pollution (MVP)
3. Phase 4 → US2 verified: consistent location guaranteed
4. Phase 5 → Polished: lint-clean, manually verified

---

## Notes

- `[P]` tasks within Phase 3 modify different function bodies in `src/storage.rs` — verify no editing conflicts if worked simultaneously
- The low-level `load_board_from`, `save_board_to`, `append_done_entry_to` functions are intentionally unchanged — existing tests in `storage_tests.rs` (T033–T035) continue to pass without modification
- `CURRENT_LOG` constant: after T005–T007, check whether it is still referenced; if not, remove it (Principle II: no dead code)
- Tests that call `save_board()` or `append_done_entry()` will create files in the actual platform data directory — this is expected and correct behaviour, not a test pollution issue
