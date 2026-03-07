# Tasks: Status-Ordered Task Lists

**Input**: Design documents from `/specs/011-status-ordered-tasks/`
**Prerequisites**: plan.md ✅, spec.md ✅, research.md ✅, data-model.md ✅, quickstart.md ✅

**Tests**: TDD is explicitly required by the feature spec (FR-008) and Constitution Principle I.
Tests MUST be written first and confirmed FAILING before any implementation code is written.

**Organization**: Tasks are grouped by user story to enable independent implementation and testing.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to
- Exact file paths are included in all descriptions

## Path Conventions

Single project: `src/`, `tests/` at repository root.

---

## Phase 1: Setup (Baseline Verification)

**Purpose**: Confirm the baseline is clean before touching the data model.

- [X]T001 Confirm all existing tests pass by running `cargo test` and `cargo clippy -- -D warnings`

---

## Phase 2: Foundational — `StatusTaskMap` Data Structure

**Purpose**: Introduce the per-status ordered list type that all user stories depend on.
All subsequent phases MUST wait for this phase to complete.

**⚠️ CRITICAL**: No user story work can begin until this phase is complete.

> **TDD Rule**: Write and confirm each test FAILS before writing the implementation (T002–T004 before T005, T006 before T007).

### Tests for `StatusTaskMap` (write first — must FAIL)

- [X]T002 Write failing unit tests for `StatusTaskMap::new()` returns empty lists for all statuses, `insert_at_top()` places at index 0 and pushes existing items down, and `tasks_for()` returns correct slice in `src/model.rs`
- [X]T003 Write failing unit test for `StatusTaskMap::remove_by_id()` removes the correct task and returns it, and is a no-op when id not found, in `src/model.rs`
- [X]T004 Write failing unit test for `StatusTaskMap::from_flat()` distributes tasks into per-status Vecs preserving relative order within each status in `src/model.rs`

### Implementation of `StatusTaskMap`

- [X]T005 Implement `StatusTaskMap` struct with fields `todo`, `doing`, `checking`, `done` (each `Vec<Task>`), and methods `new()`, `tasks_for(&Status) -> &Vec<Task>`, `tasks_for_mut(&Status) -> &mut Vec<Task>`, `insert_at_top(Status, Task)`, `remove_by_id(Status, u64) -> Option<Task>`, `all_tasks() -> impl Iterator`, `from_flat(Vec<Task>) -> Self` in `src/model.rs` — all T002–T004 tests must now pass

### Tests for `BoardState` serde migration (write first — must FAIL)

- [X]T006 Write failing unit test in `tests/storage_tests.rs` that saves a `BoardState` with tasks across multiple statuses, reloads it, and verifies the flat JSON `"tasks"` array format is preserved and all tasks round-trip correctly

### Implementation of `BoardState` migration

- [X]T007 Replace `BoardState.tasks: Vec<Task>` with `tasks: StatusTaskMap` in `src/model.rs`; add custom `Serialize` (flatten all status Vecs into a single `"tasks"` JSON array) and `Deserialize` (read flat array, call `StatusTaskMap::from_flat()`) so the JSON format stays backward-compatible with existing `current.log` files; update `BoardState::with_tasks()`, `BoardState::default()`, `with_tasks()` helper in `src/model.rs`

### Update query layer

- [X]T008 Update `AppState::tasks_for_column()` in `src/app.rs` to delegate to `self.board.tasks.tasks_for(status)` instead of filtering the flat Vec; update `focus_task_by_id()` and `clamp_focus()` to use `StatusTaskMap`; run `cargo test` — all previously passing tests must still pass

**Checkpoint**: `StatusTaskMap` type is complete and integrated. `AppState::tasks_for_column()` returns tasks from per-status lists. All existing tests pass. User story implementation can now begin.

---

## Phase 3: User Story 1+2 — Newest Task at Top (Priority: P1) 🎯 MVP

**Goal**: When a task is advanced to a new status, it appears at index 0 (top) of that column. When a new task is created, it appears at the top of Todo.

**Independent Test**: Create tasks A then B; verify B is at index 0 of Todo. Advance A to Doing; verify A is at index 0 of Doing. Advance B to Doing; verify B is now at index 0 of Doing and A is at index 1.

### Tests for US1+US2 (write first — must FAIL)

- [X]T009 [US1] Write failing test in `src/app.rs`: given task A in Doing, advancing a second task B from Todo to Doing places B at index 0 of Doing and A at index 1 (most recently moved is at top)
- [X]T010 [US2] Write failing test in `src/app.rs`: given an existing Todo task, creating a new task places it at index 0 of Todo (new task is at top, existing task is below)

### Implementation for US1+US2

- [X]T011 [US1] Update `advance_status()` in `src/app.rs` to call `board.tasks.remove_by_id(old_status, id)` then `board.tasks.insert_at_top(new_status, task)` instead of mutating status in-place; update `focus_task_by_id()` so focus follows to index 0 of the destination status — T009 test must now pass
- [X]T012 [US2] Update the create-task path in `confirm_input()` in `src/app.rs` to call `board.tasks.insert_at_top(Status::Todo, task)` and set focus to `(Todo.col_index(), 0)` instead of the previous position-search insertion — T010 test must now pass

**Checkpoint**: `cargo test` passes. New tasks appear at top of Todo; advanced tasks appear at top of destination column.

---

## Phase 4: User Story 3 — Retreat Places Task at Top (Priority: P2)

**Goal**: When a task is retreated to a previous status, it appears at index 0 (top) of the destination column.

**Independent Test**: Given tasks B, C in Todo (C at top), retreat A from Doing back to Todo; verify A is now at index 0 of Todo above C and B.

### Tests for US3 (write first — must FAIL)

- [X]T013 [US3] Write failing test in `src/app.rs`: given task A in Doing and tasks B, C already in Todo, retreating A back to Todo places A at index 0 of Todo (most recently retreated is at top)

### Implementation for US3

- [X]T014 [US3] Update `retreat_status()` in `src/app.rs` to call `board.tasks.remove_by_id(old_status, id)` then `board.tasks.insert_at_top(prev_status, task)` and clear `done_at` when appropriate; update focus tracking to land at index 0 of destination — T013 test must now pass

**Checkpoint**: `cargo test` passes. Retreated tasks appear at top of destination column.

---

## Phase 5: User Story 4 — Reorder Within Column (Priority: P2)

**Goal**: J/K manual reorder operations continue to work correctly against the new per-status Vecs.

**Independent Test**: Given 3 tasks in Todo (C at top, B in middle, A at bottom), use reorder-down on C; verify C moves to index 1 and B is at index 0.

### Tests for US4 (write first — must FAIL)

- [X]T015 [US4] Write failing test in `src/app.rs` confirming `reorder_task_down()` and `reorder_task_up()` swap adjacent tasks within a `StatusTaskMap` Vec and update the focus cursor correctly

### Implementation for US4

- [X]T016 [US4] Update `reorder_task_down()` and `reorder_task_up()` in `src/app.rs` to operate on `board.tasks.tasks_for_mut(status)` using direct Vec index swapping instead of the previous flat-Vec index collection approach — T015 and all existing reorder tests must now pass

**Checkpoint**: `cargo test` passes. Manual reorder still works.

---

## Phase 6: User Story 5 — Data Persistence & Backward Compatibility (Priority: P2)

**Goal**: Per-status order survives save/reload. Legacy flat JSON files load correctly.

**Independent Test**: Save a board with tasks at various statuses and specific ordering, reload it, verify per-status order is preserved. Load a hand-crafted legacy JSON file with flat `"tasks"` array; verify tasks distribute correctly by status.

### Tests for US5 (write first — must FAIL)

- [X]T017 [P] [US5] Write failing test in `tests/storage_tests.rs`: save a board where tasks in Todo are ordered [B, A] (B at top), reload from JSON; verify the Todo list still has B at index 0 and A at index 1 (order preserved through serialization round-trip)
- [X]T018 [P] [US5] Write failing test in `tests/storage_tests.rs`: parse a hand-crafted legacy JSON string with a flat `"tasks"` array containing tasks of mixed statuses; verify `load_board_from()` succeeds and each task lands in the correct per-status list

### Implementation and fixes for US5

- [X]T019 [US5] Verify custom `Serialize`/`Deserialize` (implemented in T007) makes T017 and T018 pass; if not, fix the serde implementation in `src/model.rs` to correctly preserve list order on round-trip and handle legacy flat format

### Remaining `BoardState` consumers

- [X]T020 [US5] Update `apply_daily_filter()` in `src/app.rs` to retain tasks per-status: for each status, filter its Vec; keep all non-Done statuses untouched; for Done, retain only tasks where `done_at` is today
- [X]T021 [US5] Update `delete_focused_card()` in `src/app.rs` to call `board.tasks.remove_by_id(status, id)` instead of `board.tasks.retain()`

**Checkpoint**: `cargo test` passes. All five user stories are functionally complete.

---

## Phase 7: Polish & Cross-Cutting Concerns

**Purpose**: Lint, format, and verify the complete test suite.

- [X]T022 [P] Run `cargo clippy -- -D warnings` and fix all warnings in `src/model.rs` and `src/app.rs`
- [X]T023 [P] Run `cargo fmt` to ensure consistent formatting across changed files
- [X]T024 Run `cargo test` and confirm 100% of tests pass (existing + new)

---

## Dependencies & Execution Order

### Phase Dependencies

- **Phase 1 (Setup)**: No dependencies — start immediately
- **Phase 2 (Foundational)**: Depends on Phase 1 — **BLOCKS** all user story phases
- **Phase 3 (US1+US2)**: Depends on Phase 2 completion
- **Phase 4 (US3)**: Depends on Phase 2 completion; can run in parallel with Phase 3
- **Phase 5 (US4)**: Depends on Phase 2 completion; can run in parallel with Phases 3–4
- **Phase 6 (US5)**: Depends on Phase 2 completion; T020–T021 depend on Phase 3 completion
- **Phase 7 (Polish)**: Depends on all prior phases

### User Story Dependencies

- **US1+US2 (P1, Phase 3)**: Depends on Foundational only
- **US3 (P2, Phase 4)**: Depends on Foundational only — independent of US1+US2
- **US4 (P2, Phase 5)**: Depends on Foundational only — independent of all other stories
- **US5 (P2, Phase 6)**: Depends on Foundational; T020–T021 also depend on US1+US2

### Within Each Phase (TDD order)

1. Write test tasks → confirm tests FAIL (red)
2. Implement code → confirm tests PASS (green)
3. Refactor while keeping tests green
4. All prior tests must still pass before moving to next phase

### Parallel Opportunities

- T002, T003, T004 can run in parallel (different test functions in the same file — coordinate to avoid conflicts)
- T017 and T018 can run in parallel (different test functions)
- T022 and T023 can run in parallel (lint vs. format)
- Phases 3, 4, 5 can run in parallel (different methods in `src/app.rs` — coordinate to avoid conflicts)

---

## Parallel Example: Phase 2 Tests

```bash
# T002, T003, T004 — all write tests in src/model.rs (different test functions)
# Coordinate by splitting the test module into sections:
Task T002: StatusTaskMap construction and insert tests
Task T003: StatusTaskMap remove_by_id tests
Task T004: StatusTaskMap from_flat tests
```

---

## Implementation Strategy

### MVP First (US1+US2 Only)

1. Complete Phase 1: Baseline verification
2. Complete Phase 2: Foundational (`StatusTaskMap` + migration) — **CRITICAL, blocks everything**
3. Complete Phase 3: US1+US2 — advance and create place at top
4. **STOP and VALIDATE**: Newest task appears at top in each column
5. Demo/ship if sufficient

### Incremental Delivery

1. Phase 1 + Phase 2 → Foundation ready
2. Add Phase 3 (US1+US2) → Core ordering works (MVP)
3. Add Phase 4 (US3) → Retreat also places at top
4. Add Phase 5 (US4) → Reorder still works
5. Add Phase 6 (US5) → Persistence verified; backward compat confirmed
6. Phase 7 → Clean up and ship

---

## Notes

- **TDD is non-negotiable** (Constitution Principle I): Each test task must result in a FAILING test before the paired implementation task runs
- `StatusTaskMap` in Phase 2 is the **only** foundational blocker — it's a small, self-contained type
- `src/app.rs` has the most churn; coordinate task ordering within that file to avoid conflicts
- `src/ui.rs` and `src/storage.rs` need **no changes** — `tasks_for_column()` API is unchanged for the UI, and the JSON format is preserved by custom serde
- Commit after each checkpoint to make progress visible
