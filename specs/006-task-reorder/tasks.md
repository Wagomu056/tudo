# Tasks: Task Reordering Within Columns

**Input**: Design documents from `specs/006-task-reorder/`
**Prerequisites**: plan.md ✓, spec.md ✓, research.md ✓, data-model.md ✓, contracts/ ✓, quickstart.md ✓

**Organization**: Tasks are grouped by user story. US1 and US2 are both P1 — implement sequentially in the same files. US3 builds on the key dispatch added in US1/US2.

**Test strategy**: Test-first (Constitution Principle I). Each story's tests are written and confirmed failing before implementation begins.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (US1, US2, US3)
- Exact file paths are included in all descriptions

---

## Phase 1: Setup (Baseline Verification)

**Purpose**: Confirm the existing test suite is green before any changes.
No new project setup is required (existing Rust project, no new crates).

- [X] T001 Run `cargo test` from repo root to establish a clean baseline — all existing tests must pass before changes begin

**Checkpoint**: All pre-existing tests green. Ready to add new failing tests.

---

## Phase 2: Foundational (Blocking Prerequisites)

*No foundational phase required.* This feature adds to an existing Rust project with
no new crates, no new files, and no new modules. Proceed directly to User Story phases.

---

## Phase 3: User Story 1 — Move Task Down (Priority: P1) 🎯 MVP

**Goal**: Users can press J (Shift+J) in Normal mode to move the focused task one position down within its column. Focus follows the moved task. Pressing J at the last position is a no-op.

**Independent Test**: Launch the app, create ≥2 tasks in Todo, focus the first, press J — verify it moves to position 2, focus stays on it. Press J again at the last position — no movement.

### Tests for User Story 1 ⚠️ Write and confirm FAILING before T006

- [X] T002 [US1] Add 4 failing inline tests for `reorder_task_down` to the `#[cfg(test)] mod tests` block in `src/app.rs`: `test_reorder_task_down_swaps_and_follows_focus`, `test_reorder_task_down_at_last_is_noop`, `test_reorder_does_not_affect_other_columns`, `test_reorder_preserves_task_status` (exact test bodies in `specs/006-task-reorder/quickstart.md` Step 1)
- [X] T003 [US1] Run `cargo test` to confirm all 4 T002 tests fail (red) — do not proceed until confirmed failing

### Implementation for User Story 1

- [X] T004 [US1] Implement `reorder_task_down(&mut self) -> bool` on `AppState` in `src/app.rs` (exact implementation in `specs/006-task-reorder/quickstart.md` Step 2): collect Vec indices for the focused column, swap `board.tasks[col_indices[cursor]]` with `board.tasks[col_indices[cursor+1]]`, advance `focused_card[col]`, return `true`; return `false` (no-op) if cursor is already last
- [X] T005 [US1] Run `cargo test` to confirm all 4 T002 tests pass (green) — do not proceed until all pass
- [X] T006 [US1] Add `KeyCode::Char('J') => { app.reorder_task_down(); }` to `handle_normal_key` in `src/main.rs` (alongside existing `'D'` and `'E'` bindings)

**Checkpoint**: J key moves tasks down, focus follows, boundary is silent no-op. User Story 1 fully functional.

---

## Phase 4: User Story 2 — Move Task Up (Priority: P1)

**Goal**: Users can press K (Shift+K) in Normal mode to move the focused task one position up within its column. Focus follows the moved task. Pressing K at the first position is a no-op.

**Independent Test**: Launch the app, create ≥2 tasks in Todo, focus the last one, press K — verify it moves to the position above, focus stays on it. Press K again at the first position — no movement.

### Tests for User Story 2 ⚠️ Write and confirm FAILING before T009

- [X] T007 [US2] Add 2 failing inline tests for `reorder_task_up` to the `#[cfg(test)] mod tests` block in `src/app.rs`: `test_reorder_task_up_swaps_and_follows_focus`, `test_reorder_task_up_at_first_is_noop` (exact test bodies in `specs/006-task-reorder/quickstart.md` Step 1)
- [X] T008 [US2] Run `cargo test` to confirm both T007 tests fail (red) — do not proceed until confirmed failing

### Implementation for User Story 2

- [X] T009 [US2] Implement `reorder_task_up(&mut self) -> bool` on `AppState` in `src/app.rs` (exact implementation in `specs/006-task-reorder/quickstart.md` Step 2): return `false` immediately if `cursor == 0`; otherwise collect Vec indices for the focused column, swap `board.tasks[col_indices[cursor-1]]` with `board.tasks[col_indices[cursor]]`, decrement `focused_card[col]`, return `true`
- [X] T010 [US2] Run `cargo test` to confirm both T007 tests pass (green) — do not proceed until all pass
- [X] T011 [US2] Add `KeyCode::Char('K') => { app.reorder_task_up(); }` to `handle_normal_key` in `src/main.rs` (alongside the J binding added in T006)

**Checkpoint**: K key moves tasks up, focus follows, boundary is silent no-op. User Story 2 fully functional. Both J and K work independently.

---

## Phase 5: User Story 3 — Debounced Persistence (Priority: P2)

**Goal**: Reorder saves are debounced (1 s after last J/K press). Any other save event cancels the debounce and saves immediately, capturing all pending reorder changes. Quitting flushes the pending save before exit.

**Independent Test**: Create and reorder tasks; quit within 1 second of last J/K press; relaunch — verify order is preserved. Also: reorder, then immediately press Enter (status change) — verify no data loss on relaunch.

### Implementation for User Story 3

- [X] T012 [US3] Add two local variables at the top of `run_app` in `src/main.rs`: `let mut reorder_save_pending = false;` and `let mut last_reorder_at: Option<std::time::Instant> = None;` (full code in `specs/006-task-reorder/quickstart.md` Step 3b)
- [X] T013 [US3] Add debounce check in the poll-timeout branch of `run_app` in `src/main.rs`: when `reorder_save_pending` is true and `last_reorder_at.elapsed() >= 1 s`, call `storage::save_board`, clear `reorder_save_pending` (full code in `specs/006-task-reorder/quickstart.md` Step 3b)
- [X] T014 [US3] Refactor the key-event save logic in `run_app` in `src/main.rs`: for J/K keys set `reorder_save_pending = true; last_reorder_at = Some(Instant::now())` and skip the immediate save; for all other key events set `reorder_save_pending = false` and call `storage::save_board` (full code in `specs/006-task-reorder/quickstart.md` Step 3b)
- [X] T015 [US3] Add quit-flush guard in `run_app` in `src/main.rs`: before each `break` (both `q` and `Ctrl+C` paths), check `if reorder_save_pending { let _ = storage::save_board(...); }` (full code in `specs/006-task-reorder/quickstart.md` Step 3b)

**Checkpoint**: Debounced saves work; quit never discards pending reorder changes; other operations preempt debounce and save immediately.

---

## Phase 6: Polish & Verification

**Purpose**: Ensure code quality gates pass and the full feature is manually verified.

- [X] T016 Run `cargo test` for final full suite — all tests (pre-existing + new) must pass
- [X] T017 Run `cargo clippy -- -D warnings` — zero warnings allowed
- [X] T018 Run `cargo fmt --check` — zero formatting issues allowed
- [ ] T019 Run manual smoke test per `specs/006-task-reorder/quickstart.md` acceptance smoke test: create 3 tasks, J×2, boundary J no-op, K back, quit, relaunch — verify order preserved

**Checkpoint**: All CI gates pass. Feature is complete and verified.

---

## Dependencies & Execution Order

### Phase Dependencies

- **Phase 1 (Setup)**: No dependencies — start immediately
- **Phase 2 (Foundational)**: Skipped — not applicable
- **Phase 3 (US1)**: Depends on Phase 1 baseline passing
- **Phase 4 (US2)**: Depends on Phase 3 complete (same files — sequential required)
- **Phase 5 (US3)**: Depends on Phase 4 complete (refactors `main.rs` save logic that US1/US2 key bindings were just added to)
- **Phase 6 (Polish)**: Depends on all story phases complete

### User Story Dependencies

- **US1 (P1)**: Blocking for US2 and US3 (same files — sequential in practice)
- **US2 (P1)**: Can logically start after Phase 1; sequential after US1 in practice
- **US3 (P2)**: Must follow US1 and US2 (refactors the save logic around the J/K dispatch added in those phases)

### Within Each User Story

```
Write tests → Confirm RED → Implement → Confirm GREEN → Wire key dispatch
```

The Constitution requires tests to be written and failing before implementation code is touched.

---

## Parallel Opportunities

All tasks in this feature modify just two files (`src/app.rs` and `src/main.rs`), so no true parallel implementation is possible for a single developer. For a two-developer team:

```
# If two developers are available after T001:
Developer A: T002 → T003 → T004 → T005 → T006  (US1 in app.rs + main.rs)
Developer B: T007 → T008 → T009 → T010 → T011  (US2 in app.rs + main.rs)
# RISK: both touch same files — requires coordination or separate commits merged carefully
# Recommended: sequential (single developer, ~1 hour total)
```

US3 (T012–T015) must follow both US1 and US2 as it refactors the save block around the newly added J/K dispatch.

---

## Implementation Strategy

### MVP (User Story 1 Only)

1. T001: Baseline
2. T002–T005: Implement `reorder_task_down` test-first
3. T006: Wire J key
4. **STOP and validate**: J key works, order persists at next save (standard save-on-every-key still active at this point)

### Full Feature (All Stories)

1. MVP above
2. T007–T011: Add K key (reorder_task_up)
3. T012–T015: Debounce save logic
4. T016–T019: Polish

### Total Estimates

| Phase | Tasks | Files |
|-------|-------|-------|
| Setup | 1 | — |
| US1 (Move Down) | 5 | src/app.rs, src/main.rs |
| US2 (Move Up) | 5 | src/app.rs, src/main.rs |
| US3 (Debounce) | 4 | src/main.rs |
| Polish | 4 | — |
| **Total** | **19** | **2 source files** |

---

## Notes

- No [P] markers on implementation tasks — both source files are shared across all phases
- Test confirmation tasks (T003, T005, T008, T010) are mandatory workflow steps per Constitution Principle I
- `quickstart.md` contains exact code for each implementation task — reference it directly
- All 6 new test functions are specified in `specs/006-task-reorder/quickstart.md` Step 1
- `Vec::swap` is `O(1)` stdlib — no performance concerns
- `std::time::Instant` is stdlib — no new crates
