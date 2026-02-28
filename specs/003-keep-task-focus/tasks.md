# Tasks: Keep Task Focus on Status Change

**Input**: Design documents from `/specs/003-keep-task-focus/`
**Prerequisites**: plan.md ✅, spec.md ✅, research.md ✅

**Tests**: Included — Constitution Principle I mandates test-first (TDD). All tests MUST be written and confirmed to fail (red) before the corresponding implementation.

**Organization**: Tasks grouped by user story to enable independent implementation and testing.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (US1, US2)
- Exact file paths included in all task descriptions

## Path Conventions

Single project: `src/` at repository root. All changes confined to `src/app.rs`.

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: No project initialization needed — the Rust project already exists with all dependencies. This phase adds the shared test helper that all test tasks will use.

- [x] T001 Add private test helper `make_app_with_tasks` to `#[cfg(test)]` module in `src/app.rs` that constructs an `AppState` with a configurable list of tasks (id, title, status) and a specified initial `focused_col` and `focused_card`

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: The `focus_task_by_id` private method is the single mechanism shared by both user stories. It must exist before either `advance_status` or `retreat_status` can be updated.

**⚠️ CRITICAL**: No user story implementation can begin until this phase is complete.

- [x] T002 Add private method `focus_task_by_id(id: u64)` to `impl AppState` in `src/app.rs` — looks up the task by id, reads its new `Status`, computes its position in that column's filtered list, and sets `self.focused_col` and `self.focused_card[col]` accordingly

**Checkpoint**: Foundation ready — user story phases can now begin.

---

## Phase 3: User Story 1 - Maintain Focus After Forward Status Change (Priority: P1) 🎯 MVP

**Goal**: After pressing Enter on a focused task, `focused_col` and `focused_card` follow the task to its new column and position.

**Independent Test**: Run `cargo test` with only the US1 tests enabled; verify focus tracking for `advance_status` works for normal moves and boundary cases.

### Tests for User Story 1 ⚠️ Constitution Principle I — Write BEFORE Implementation, Confirm Red

- [x] T003 [US1] Add failing test `test_advance_status_keeps_focus_on_task` in `src/app.rs` `#[cfg(test)]` — create a task at `Doing`, call `advance_status`, assert `focused_col == Checking.col_index()` and `focused_card[Checking.col_index()] == 0`
- [x] T004 [US1] Add failing test `test_advance_moves_focus_across_all_statuses` in `src/app.rs` `#[cfg(test)]` — advance a task through Todo→Doing→Checking→Done one step at a time and assert focus follows at each step
- [x] T005 [US1] Add passing test `test_advance_at_done_boundary_preserves_focus` in `src/app.rs` `#[cfg(test)]` — create a task at `Done`, call `advance_status`, assert `focused_col` and `focused_card` are unchanged (boundary no-op; this test should already pass before implementation)
- [x] T006 [US1] Run `cargo test` and confirm T003 and T004 fail (red), T005 passes

### Implementation for User Story 1

- [x] T007 [US1] Update `advance_status` in `src/app.rs` to call `self.focus_task_by_id(id)` immediately after the status mutation block, followed by `self.clamp_focus()` (replacing the single `self.clamp_focus()` call that was there)
- [x] T008 [US1] Run `cargo test` and confirm T003, T004, T005 all pass (green)

**Checkpoint**: User Story 1 is fully functional. Press Enter on any task in the TUI and verify the highlight moves with the task.

---

## Phase 4: User Story 2 - Maintain Focus After Backward Status Change (Priority: P2)

**Goal**: After pressing BackSpace on a focused task, `focused_col` and `focused_card` follow the task to its previous column and position.

**Independent Test**: Run `cargo test` with the US2 tests enabled; verify focus tracking for `retreat_status` works for normal moves and boundary cases.

### Tests for User Story 2 ⚠️ Constitution Principle I — Write BEFORE Implementation, Confirm Red

- [x] T009 [US2] Add failing test `test_retreat_status_keeps_focus_on_task` in `src/app.rs` `#[cfg(test)]` — create a task at `Doing`, call `retreat_status`, assert `focused_col == Todo.col_index()` and `focused_card[Todo.col_index()] == 0`
- [x] T010 [US2] Add failing test `test_retreat_moves_focus_across_all_statuses` in `src/app.rs` `#[cfg(test)]` — retreat a task through Done→Checking→Doing→Todo one step at a time and assert focus follows at each step
- [x] T011 [US2] Add passing test `test_retreat_at_todo_boundary_preserves_focus` in `src/app.rs` `#[cfg(test)]` — create a task at `Todo`, call `retreat_status`, assert `focused_col` and `focused_card` are unchanged (boundary no-op; should already pass)
- [x] T012 [US2] Run `cargo test` and confirm T009 and T010 fail (red), T011 passes

### Implementation for User Story 2

- [x] T013 [US2] Update `retreat_status` in `src/app.rs` to call `self.focus_task_by_id(id)` immediately after the status mutation block, followed by `self.clamp_focus()` (replacing the single `self.clamp_focus()` call)
- [x] T014 [US2] Run `cargo test` and confirm T009, T010, T011 all pass (green)

**Checkpoint**: Both user stories are fully functional. Press BackSpace on any task and verify the highlight moves with the task.

---

## Phase 5: Polish & Cross-Cutting Concerns

**Purpose**: Edge-case coverage and CI compliance checks.

- [x] T015 Add failing test `test_source_column_clamped_when_last_task_moves_out` in `src/app.rs` `#[cfg(test)]` — create a board with one task in `Doing` and one in `Checking`; focus the `Doing` task and call `advance_status`; assert `focused_card[Doing.col_index()] == 0` (clamped after Doing becomes empty) and focus is on the moved task in `Checking`
- [x] T016 Run `cargo test` to verify all tests pass including T015
- [x] T017 Run `cargo clippy --all-targets -- -D warnings` in repo root and fix any warnings introduced by new code
- [x] T018 Run `cargo fmt --check` in repo root; run `cargo fmt` if formatting issues are found

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies — start immediately
- **Foundational (Phase 2)**: Depends on Phase 1 (test helper must exist first)
- **User Story 1 (Phase 3)**: Depends on Phase 2 (`focus_task_by_id` must exist)
- **User Story 2 (Phase 4)**: Depends on Phase 3 (same method; sequential for safety)
- **Polish (Phase 5)**: Depends on Phases 3 and 4 complete

### User Story Dependencies

- **US1 (P1)**: Depends on Foundational (T002) — no dependency on US2
- **US2 (P2)**: Depends on US1 completing T002; shares `focus_task_by_id` method

### Within Each User Story

1. Tests written and confirmed FAILING (red) — never skip this step
2. Implementation added
3. Tests confirmed PASSING (green)
4. `cargo test` run to check for regressions

### Parallel Opportunities

All tasks in this feature touch the single file `src/app.rs`, so true parallelism is limited. However:

- T003, T004, T005 (US1 tests) can be written in one sitting as a batch
- T009, T010, T011 (US2 tests) can be written in one sitting as a batch
- T017 and T018 (clippy + fmt) can be run concurrently

---

## Parallel Example: User Story 1 Tests

```bash
# All three US1 tests can be added in sequence during a single editing session:
# T003: test_advance_status_keeps_focus_on_task
# T004: test_advance_moves_focus_across_all_statuses
# T005: test_advance_at_done_boundary_preserves_focus
# Then run: cargo test 2>&1 | grep -E "(FAILED|ok)" to confirm red/green state
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup (T001 — test helper)
2. Complete Phase 2: Foundational (T002 — `focus_task_by_id` method)
3. Complete Phase 3: User Story 1 (T003–T008)
4. **STOP and VALIDATE**: Run `cargo test`; manually test Enter key in TUI
5. Ship if ready

### Incremental Delivery

1. Setup + Foundational → shared infrastructure ready
2. User Story 1 → Enter key focus tracking works → demo/validate
3. User Story 2 → BackSpace key focus tracking works → demo/validate
4. Polish → edge cases + CI compliance → merge-ready

---

## Notes

- ALL tests must be written FIRST and confirmed red before implementation (Constitution Principle I)
- No new crates introduced (Constitution Principle II)
- All `Option` returns handled explicitly — no `unwrap()` in production paths (Constitution Principle V)
- `clamp_focus()` must still be called after `focus_task_by_id()` to correct the **source column** cursor
- `[P]` is not marked on most tasks because all changes are in the same file (`src/app.rs`)
- Commit after each logical group: after tests (red), after implementation (green), after clippy/fmt
