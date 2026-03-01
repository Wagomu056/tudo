# Tasks: Click-to-Focus Items

**Input**: Design documents from `/specs/008-click-item-focus/`
**Prerequisites**: plan.md, spec.md, research.md, data-model.md, quickstart.md

**Tests**: Included — Constitution Principle I (Test-First) requires tests before implementation.

**Organization**: Tasks are grouped by user story to enable independent implementation and testing of each story.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2)
- Include exact file paths in descriptions

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Add hit-region data structures shared by both user stories

- [x] T001 [P] Add `TaskHitRegion` struct to `src/model.rs` with fields: `row_start: u16`, `row_end: u16`, `col_start: u16`, `col_end: u16`, `column: usize`, `card_index: usize`
- [x] T002 [P] Add `MemoHitRegion` struct to `src/model.rs` with fields: `row: u16`, `col_start: u16`, `col_end: u16`, `memo_index: usize`
- [x] T003 Add `clickable_tasks: Vec<TaskHitRegion>` and `clickable_memos: Vec<MemoHitRegion>` fields to `AppState` in `src/app.rs`, initialize empty in `AppState::new()`
- [x] T004 Clear `clickable_tasks` and `clickable_memos` at frame start alongside `clickable_urls` in `src/main.rs` (line 72)

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Input mode guard that applies to all click-to-focus behavior

- [x] T005 Write test in `src/app.rs` tests: clicking a task hit region while in `AppMode::InputTitle` must NOT change `focused_col` or `focused_card` — verify focus is unchanged after `handle_left_click`
- [x] T006 Add input mode guard to `handle_left_click` in `src/app.rs`: after URL region check, return early if `app.mode != AppMode::Normal` to prevent focus changes during editing (FR-006)

**Checkpoint**: Foundation ready — hit-region structs exist, input mode guard active, user story implementation can begin

---

## Phase 3: User Story 1 - Click a Task to Focus It (Priority: P1) 🎯 MVP

**Goal**: Left-clicking a task card in any kanban column moves focus to that task, switching focus area if needed

**Independent Test**: Click any task in any column → blue highlight moves to that task, detail panel updates

### Tests for User Story 1 ⚠️

> **NOTE: Write these tests FIRST, ensure they FAIL before implementation**

- [x] T007 [US1] Write test in `src/app.rs` tests: given tasks in Todo column and a `TaskHitRegion` at (row=3, col_start=0, col_end=20, column=0, card_index=2), calling `handle_left_click(app, 10, 3)` sets `focused_col=0` and `focused_card[0]=2`
- [x] T008 [US1] Write test in `src/app.rs` tests: given focus on Doing column, clicking a `TaskHitRegion` for Done column switches `focused_col` to Done's col_index
- [x] T009 [US1] Write test in `src/app.rs` tests: given `focus_area=FocusArea::Memo`, clicking a task hit region switches `focus_area` to `FocusArea::Kanban` and sets correct `focused_col`/`focused_card`
- [x] T010 [US1] Write test in `src/app.rs` tests: clicking at coordinates that match NO `TaskHitRegion` and NO `MemoHitRegion` leaves focus unchanged (empty area / border click)

### Implementation for User Story 1

- [x] T011 [US1] Populate `clickable_tasks` in `render_column()` in `src/ui.rs`: for each task, compute `TaskHitRegion` using `area.x`/`area.y`, border offset (+1), and `cumulative_rows` for wrapped titles, then push to `app.clickable_tasks`
- [x] T012 [US1] Extend `handle_left_click` in `src/app.rs`: after URL check and input mode guard, iterate `app.clickable_tasks` to find matching region for `(col, row)`, then set `app.focused_col`, `app.focused_card[region.column]`, and `app.focus_area = FocusArea::Kanban`

**Checkpoint**: Clicking any task card focuses it. Detail panel updates automatically (already driven by `focused_task()`). All US1 tests pass.

---

## Phase 4: User Story 2 - Click a Memo to Focus It (Priority: P2)

**Goal**: Left-clicking a memo item in the memo panel moves focus to that memo, switching focus area if needed

**Independent Test**: Click any memo in the grid → focus switches to memo panel, clicked memo highlighted, detail panel shows memo info

### Tests for User Story 2 ⚠️

> **NOTE: Write these tests FIRST, ensure they FAIL before implementation**

- [x] T013 [US2] Write test in `src/app.rs` tests: given memos and a `MemoHitRegion` at (row=20, col_start=0, col_end=24, memo_index=3), calling `handle_left_click(app, 10, 20)` sets `focus_area=FocusArea::Memo` and `focused_memo=3`
- [x] T014 [US2] Write test in `src/app.rs` tests: given `focus_area=FocusArea::Kanban`, clicking a memo hit region switches `focus_area` to `FocusArea::Memo`
- [x] T015 [US2] Write test in `src/app.rs` tests: given `focus_area=FocusArea::Memo` with `focused_memo=0`, clicking a `MemoHitRegion` with `memo_index=3` updates `focused_memo` to 3

### Implementation for User Story 2

- [x] T016 [US2] Populate `clickable_memos` in `render_memo_panel()` in `src/ui.rs`: for each memo item, compute `MemoHitRegion` using `inner.x`, `inner.y`, grid position (`row * item_h`, `col * item_w`), and push to `app.clickable_memos`
- [x] T017 [US2] Extend `handle_left_click` in `src/app.rs`: after task region check, iterate `app.clickable_memos` to find matching region for `(col, row)`, then set `app.focused_memo = region.memo_index` and `app.focus_area = FocusArea::Memo`

**Checkpoint**: Clicking any memo item focuses it. Both US1 and US2 work independently. All tests pass.

---

## Phase 5: Polish & Cross-Cutting Concerns

**Purpose**: Final validation and cleanup

- [x] T018 Run `cargo test` — all existing and new tests pass
- [x] T019 Run `cargo clippy -- -D warnings` — no warnings
- [x] T020 Run quickstart.md manual verification steps (click tasks, click memos, input mode guard, empty area clicks, URL clicks)

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies — can start immediately
- **Foundational (Phase 2)**: Depends on T003 (AppState fields exist)
- **User Story 1 (Phase 3)**: Depends on Phase 1 + Phase 2 completion
- **User Story 2 (Phase 4)**: Depends on Phase 1 + Phase 2 completion; independent of US1
- **Polish (Phase 5)**: Depends on all stories being complete

### User Story Dependencies

- **User Story 1 (P1)**: Can start after Phase 2 — no dependency on US2
- **User Story 2 (P2)**: Can start after Phase 2 — no dependency on US1
- Both stories share Phase 1 structs and Phase 2 guard, but implementation is independent

### Within Each User Story

- Tests MUST be written and FAIL before implementation
- Hit-region population (ui.rs) before click handler extension (app.rs)

### Parallel Opportunities

- T001 and T002 can run in parallel (different structs, same file but independent)
- US1 and US2 can proceed in parallel after Phase 2 (different hit region types, different render functions)
- All tests within a story (marked [P] implicitly by being in different test functions) can be written together

---

## Parallel Example: User Story 1

```text
# Write all US1 tests together:
T007: Test task click sets focused_col and focused_card
T008: Test cross-column click switches focused_col
T009: Test click switches focus_area from Memo to Kanban
T010: Test empty area click is ignored

# Then implement sequentially:
T011: Populate clickable_tasks in render_column (ui.rs)
T012: Extend handle_left_click for tasks (app.rs)
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup (T001–T004)
2. Complete Phase 2: Foundational (T005–T006)
3. Complete Phase 3: User Story 1 (T007–T012)
4. **STOP and VALIDATE**: Click any task → focus moves correctly
5. Deploy/demo if ready — task click-to-focus is independently valuable

### Incremental Delivery

1. Setup + Foundational → Hit-region infrastructure ready
2. Add User Story 1 → Task click-to-focus works → MVP!
3. Add User Story 2 → Memo click-to-focus works → Feature complete
4. Polish → All tests pass, clippy clean, manual verification done

---

## Notes

- [P] tasks = different files, no dependencies
- [Story] label maps task to specific user story for traceability
- Constitution Principle I enforced: all tests written before implementation
- Existing `clickable_urls` and keyboard navigation must remain unaffected (FR-007, SC-004)
- No new crate dependencies required
