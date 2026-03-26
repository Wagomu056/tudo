# Tasks: Panel Focus Keyboard Shortcuts

**Input**: Design documents from `/specs/012-panel-focus-keys/`
**Prerequisites**: plan.md ✓, spec.md ✓, research.md ✓, data-model.md ✓, quickstart.md ✓

**Organization**: Tasks are grouped by user story to enable independent implementation and testing of each story.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (US1, US2)

## Path Conventions

- Single project: `src/` at repository root

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: No new project structure is needed. This phase confirms prerequisites and writes the shared test helper used by both user stories.

- [X] T001 Add `AppState::new_for_test()` helper or equivalent minimal constructor in `src/app.rs` `#[cfg(test)]` module so unit tests can build a default `AppState` without a live terminal

**Checkpoint**: Test infrastructure in place — user story test tasks can now be written

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: The `handle_normal_key` function in `src/main.rs` currently only dispatches based on `focus_area`. No new shared infrastructure is required beyond the test helper above — both user stories are fully independent.

**⚠️ CRITICAL**: T001 must be complete before any user story test tasks can compile.

**Checkpoint**: Foundation ready — user story phases can now proceed

---

## Phase 3: User Story 1 — Focus Memo Panel with 'm' Key (Priority: P1) 🎯 MVP

**Goal**: Pressing `m` in Normal mode moves `app.focus_area` to `FocusArea::Memo`.

**Independent Test**: Run `cargo test test_m_key` — all 'm'-key tests pass; press `m` in the running app and confirm the memo panel gains visual focus.

### Tests for User Story 1 ⚠️ Write FIRST — confirm they FAIL before implementing

- [X] T002 [US1] Add failing unit test `test_m_key_focuses_memo` in `src/app.rs` `#[cfg(test)]` — start with `focus_area = FocusArea::Kanban`, call key handler, assert `focus_area == FocusArea::Memo`
- [X] T003 [US1] Add failing unit test `test_m_key_idempotent_when_memo_focused` in `src/app.rs` `#[cfg(test)]` — start with `focus_area = FocusArea::Memo`, call key handler, assert `focus_area == FocusArea::Memo`

### Implementation for User Story 1

- [X] T004 [US1] Add `KeyCode::Char('m') => { app.focus_area = FocusArea::Memo; }` match arm inside `handle_normal_key()` in `src/main.rs`

**Checkpoint**: `cargo test test_m_key` passes — User Story 1 fully functional and independently testable

---

## Phase 4: User Story 2 — Focus Todo Panel with 't' Key (Priority: P1)

**Goal**: Pressing `t` in Normal mode moves `app.focus_area` to `FocusArea::Kanban`.

**Independent Test**: Run `cargo test test_t_key` — all 't'-key tests pass; press `t` in the running app and confirm the kanban panel gains visual focus.

### Tests for User Story 2 ⚠️ Write FIRST — confirm they FAIL before implementing

- [X] T005 [US2] Add failing unit test `test_t_key_focuses_kanban` in `src/app.rs` `#[cfg(test)]` — start with `focus_area = FocusArea::Memo`, call key handler, assert `focus_area == FocusArea::Kanban`
- [X] T006 [US2] Add failing unit test `test_t_key_idempotent_when_kanban_focused` in `src/app.rs` `#[cfg(test)]` — start with `focus_area = FocusArea::Kanban`, call key handler, assert `focus_area == FocusArea::Kanban`

### Implementation for User Story 2

- [X] T007 [US2] Add `KeyCode::Char('t') => { app.focus_area = FocusArea::Kanban; }` match arm inside `handle_normal_key()` in `src/main.rs`

**Checkpoint**: `cargo test test_t_key` passes — User Story 2 fully functional and independently testable

---

## Phase 5: Polish & Cross-Cutting Concerns

**Purpose**: Verify no regressions, linting clean, and manual smoke test.

- [X] T008 [P] Run `cargo clippy -- -D warnings` in repo root and fix any new warnings introduced by T004 and T007
- [X] T009 [P] Run `cargo fmt --check` in repo root and format if needed
- [X] T010 Run `cargo test` (full suite) and confirm all existing tests still pass
- [ ] T011 Smoke test: launch app with `cargo run`, press `m` → memo panel focused, press `t` → kanban focused, press `m` while editing a task title → character 'm' inserted (no focus switch)

---

## Dependencies & Execution Order

### Phase Dependencies

- **Phase 1 (Setup)**: No dependencies — start immediately
- **Phase 2 (Foundational)**: No new work; T001 is the only prerequisite
- **Phase 3 (US1)**: T001 must be complete; T002–T003 must FAIL before T004
- **Phase 4 (US2)**: T001 must be complete; T005–T006 must FAIL before T007
- **Phase 5 (Polish)**: T004 and T007 must be complete

### User Story Dependencies

- **User Story 1 (P1)**: Independent after T001
- **User Story 2 (P1)**: Independent after T001; can run in parallel with US1

### Within Each User Story

- Tests written first (T002–T003, T005–T006) → confirm failure → implement (T004, T007) → confirm green

### Parallel Opportunities

- T002 and T005 can be written in parallel (both test files in `src/app.rs`, non-conflicting test functions)
- T008 and T009 (clippy + fmt) can run in parallel

---

## Parallel Example: Both User Stories Together

```
# Write all tests first (can be done in one editing session):
T002: test_m_key_focuses_memo
T003: test_m_key_idempotent_when_memo_focused
T005: test_t_key_focuses_kanban
T006: test_t_key_idempotent_when_kanban_focused

# Confirm all 4 fail, then implement both arms in src/main.rs:
T004: Char('m') arm
T007: Char('t') arm

# Confirm all 4 tests pass
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete T001 (test helper)
2. Complete T002–T003 (failing tests for 'm')
3. Complete T004 (implement 'm' arm)
4. **STOP and VALIDATE**: `cargo test test_m_key` passes; smoke test 'm' key
5. Continue to US2 if satisfied

### Full Delivery (Both Stories)

1. T001 → T002, T003, T005, T006 (all tests, confirm fail) → T004, T007 (implement) → T008–T011 (polish)
2. Total: 11 tasks across 2 source files

---

## Notes

- [P] tasks = different concerns, no file conflicts
- Constitution Principle I (Test-First) is NON-NEGOTIABLE: never run T004/T007 before their respective test tasks fail
- `handle_normal_key()` is only reached in `AppMode::Normal` — no additional guard needed in the new arms
- No new crate dependencies required
- Commit after each checkpoint to preserve test-first evidence in git history
