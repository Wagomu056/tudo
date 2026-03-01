# Tasks: Memo Panel

**Input**: Design documents from `/specs/007-memo-panel/`
**Prerequisites**: plan.md ✅ spec.md ✅ research.md ✅ data-model.md ✅ contracts/keybindings.md ✅ quickstart.md ✅

**Organization**: Tasks are grouped by user story to enable independent implementation and testing.
**Tests**: Included per Constitution Principle I (Test-First, NON-NEGOTIABLE). Write each test, confirm it FAILS, then implement.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (independent scope, no incomplete dependencies)
- **[Story]**: User story label — US1/US2/US3/US4

---

## Phase 1: Setup (Baseline Verification)

**Purpose**: Confirm the codebase is clean before any modifications

- [X] T001 Run `cargo test && cargo clippy -- -D warnings` and confirm all pass (zero failures required before starting)

---

## Phase 2: Foundational (Data Model & AppState Extensions)

**Purpose**: Core type and state changes that ALL user stories depend on. No user story work can begin until this phase is complete.

**⚠️ CRITICAL**: The `Memo` type, `FocusArea` enum, and extended `BoardState`/`AppState` fields are prerequisites for every subsequent phase.

### Foundational Tests (write first, confirm each FAILS before implementing)

- [X] T002 [P] Write test `memo_new_has_correct_fields` in `tests/model_tests.rs`: assert `Memo::new(1, "title")` produces `id=1`, `title="title"`, `detail=""`
- [X] T003 [P] Write test `board_state_with_memos_round_trips` in `tests/model_tests.rs`: serialize `BoardState` containing two `Memo` items then deserialize and assert fields match
- [X] T004 [P] Write test `board_state_missing_memos_field_deserializes_to_empty` in `tests/model_tests.rs`: parse a JSON string that has no `memos`/`next_memo_id` fields and assert `board.memos.is_empty()` and `board.next_memo_id == 1`
- [X] T005 [P] Write test `save_and_load_board_with_memos` in `tests/storage_tests.rs`: save a `BoardState` containing one `Memo` to a temp file, load it back, assert memo fields are preserved

### Foundational Implementation

- [X] T006 Add `Memo` struct to `src/model.rs` with fields `id: u64`, `title: String`, `detail: String`; derive `Debug, Clone, Serialize, Deserialize`; add `Memo::new(id: u64, title: String) -> Self`
- [X] T007 [P] Add `FocusArea` enum to `src/model.rs` with variants `Kanban` and `Memo`; derive `Debug, Clone, PartialEq, Eq`
- [X] T008 [P] Add `is_memo: bool` field (default `false`) to `InputState` in `src/model.rs`; update `InputState::default()` and `InputState::clear()` to reset `is_memo`
- [X] T009 Add `memos: Vec<Memo>` with `#[serde(default)]` and `next_memo_id: u64` with `#[serde(default = "default_next_memo_id")]` to `BoardState` in `src/model.rs`; add `fn default_next_memo_id() -> u64 { 1 }` and `BoardState::alloc_memo_id(&mut self) -> u64` (depends on T006)
- [X] T010 Add `focus_area: FocusArea`, `focused_memo: usize`, `memo_cols: usize` (default `4`) fields to `AppState` in `src/app.rs`; update `AppState::new` to initialise them; add `focused_memo_item(&self) -> Option<&Memo>` and `clamp_memo_focus(&mut self)` helpers (depends on T006, T007)
- [X] T011 Add a vertical `Layout` split of `kanban_area` in `src/ui.rs` using `[Constraint::Percentage(80), Constraint::Percentage(20)]` producing `columns_area` and `memo_area`; pass `columns_area` to the existing 4-column horizontal split; add a stub `render_memo_panel(frame, memo_area, app)` that renders an empty bordered block titled `" Memo "` (depends on T010)

**Checkpoint**: `cargo test` passes — `Memo` type exists, `BoardState` deserialises from legacy JSON, storage round-trip works.

---

## Phase 3: User Story 1 — Add and View Memo Items (Priority: P1) 🎯 MVP

**Goal**: User can create a new memo item (title + optional detail) and see it reflected in the Detail panel when focused.

**Independent Test**: With `app.focus_area` set to `FocusArea::Memo` in a unit test, call `open_create_memo()`, fill `input.buffer`, call `confirm_input()`, and assert the memo appears in `board.memos` and `focused_memo_item()` returns it.

### Tests for User Story 1 (write first, confirm each FAILS)

- [X] T012 [P] [US1] Write test `memo_create_adds_to_board` in `src/app.rs` `#[cfg(test)]`: call `open_create_memo`, set buffer to `"Buy milk"`, call `confirm_input`; assert `board.memos.len() == 1` and `board.memos[0].title == "Buy milk"`
- [X] T013 [P] [US1] Write test `memo_create_focuses_new_memo` in `src/app.rs` `#[cfg(test)]`: after creating a memo, assert `focus_area == FocusArea::Memo` and `focused_memo == 0`
- [X] T014 [P] [US1] Write test `memo_create_empty_title_rejected` in `src/app.rs` `#[cfg(test)]`: call `open_create_memo`, set buffer to `"  "`, call `confirm_input`; assert `board.memos.is_empty()` and `status_msg.is_some()`
- [X] T015 [P] [US1] Write test `memo_edit_title` in `src/app.rs` `#[cfg(test)]`: create a memo then call `open_edit_memo_title`, set buffer to `"Updated"`, call `confirm_input`; assert `board.memos[0].title == "Updated"`
- [X] T016 [P] [US1] Write test `memo_edit_detail` in `src/app.rs` `#[cfg(test)]`: create a memo then call `open_edit_memo_detail`, set buffer to `"Some detail"`, call `confirm_input`; assert `board.memos[0].detail == "Some detail"`

### Implementation for User Story 1

- [X] T017 [US1] Add `open_create_memo(&mut self)`, `open_edit_memo_title(&mut self)`, and `open_edit_memo_detail(&mut self)` methods to `AppState` in `src/app.rs`; each sets `input.is_memo = true`, appropriate `is_create` and `mode` values, and pre-fills the buffer for edit methods (depends on T010)
- [X] T018 [US1] Update `confirm_input` in `src/app.rs` to branch on `self.input.is_memo`: on create, allocate memo id via `alloc_memo_id`, push `Memo::new(id, value)` to `board.memos`, set `focused_memo` to the new index; on edit, find `board.memos[focused_memo]` and update `title` or `detail` based on `mode` (depends on T017)
- [X] T019 [P] [US1] Update `render_detail_panel` in `src/ui.rs` to check `app.focus_area`: when `FocusArea::Memo`, read from `app.focused_memo_item()` and display its `title` and `detail` using the same layout as task display; when `FocusArea::Kanban`, keep existing task display (depends on T010)
- [X] T020 [P] [US1] Update `render_input_popup` in `src/ui.rs` to handle `is_memo = true` cases: display `" Add Memo "` (create), `" Edit Memo Title "` (edit title), `" Edit Memo Detail "` (edit detail) as popup title (depends on T010)
- [X] T021 [US1] Update `handle_normal_key` in `src/main.rs` to route `a`, `e`, `E` keys to `open_create_memo`, `open_edit_memo_title`, `open_edit_memo_detail` respectively when `app.focus_area == FocusArea::Memo`; existing task bindings remain active when `focus_area == Kanban` (depends on T017)

**Checkpoint**: User can launch the app, the memo panel area is visible at the bottom, pressing `a` with focus set to Memo opens the Add Memo popup, confirm creates a memo, Detail panel shows it.

---

## Phase 4: User Story 2 — Navigate Into and Within the Memo Panel (Priority: P2)

**Goal**: Pressing `j` from the bottom of a kanban column enters the memo panel; `h/l/j/k` navigate within it; `k` on the top row returns to kanban.

**Independent Test**: Unit tests verify all navigation method contracts; manual test: navigate to a column's bottom task and press `j` to confirm focus enters the memo panel.

### Tests for User Story 2 (write first, confirm each FAILS)

- [X] T022 [P] [US2] Write test `memo_enter_from_kanban_bottom` in `src/app.rs` `#[cfg(test)]`: build app with one task in Todo, focus at bottom (idx 0), call `kanban_try_move_down`; assert `focus_area == FocusArea::Memo` and `focused_memo == 0`
- [X] T023 [P] [US2] Write test `memo_enter_from_empty_kanban_column` in `src/app.rs` `#[cfg(test)]`: build app with empty kanban columns, call `kanban_try_move_down`; assert `focus_area == FocusArea::Memo`
- [X] T024 [P] [US2] Write test `memo_exit_to_kanban_from_first_row` in `src/app.rs` `#[cfg(test)]`: build app with `focus_area = Memo`, `focused_memo = 0`, `memo_cols = 4`; call `move_memo_up`; assert `focus_area == FocusArea::Kanban`
- [X] T025 [P] [US2] Write tests `memo_move_right_advances_index` and `memo_move_right_boundary_noop` in `src/app.rs` `#[cfg(test)]`: verify `move_memo_right` increments `focused_memo` and is a no-op at the last item
- [X] T026 [P] [US2] Write tests `memo_move_left_decrements_index` and `memo_move_left_boundary_noop` in `src/app.rs` `#[cfg(test)]`: verify `move_memo_left` decrements `focused_memo` and is a no-op at index 0
- [X] T027 [P] [US2] Write tests `memo_move_down_advances_by_memo_cols` and `memo_move_down_last_row_noop` in `src/app.rs` `#[cfg(test)]`: set `memo_cols = 3`, build 3 memos, call `move_memo_down` from index 0; assert `focused_memo == 3` (if no item at index 3, noop)
- [X] T028 [P] [US2] Write test `memo_move_up_row_subtracts_memo_cols` in `src/app.rs` `#[cfg(test)]`: set `memo_cols = 3`, `focused_memo = 3`; call `move_memo_up`; assert `focused_memo == 0`

### Implementation for User Story 2

- [X] T029 [US2] Add `kanban_try_move_down(&mut self)` method to `AppState` in `src/app.rs`: if `focus_area == Kanban`, check if already at bottom of current column (or column empty); if so, set `focus_area = FocusArea::Memo`, call `clamp_memo_focus`; otherwise call existing `move_down` (depends on T010)
- [X] T030 [US2] Add `move_memo_left`, `move_memo_right`, `move_memo_up`, `move_memo_down` methods to `AppState` in `src/app.rs`: `left` decrements `focused_memo` (floor 0); `right` increments (ceil `memos.len()-1`); `up` subtracts `memo_cols` if `focused_memo >= memo_cols`, else sets `focus_area = Kanban`; `down` adds `memo_cols` if result `< memos.len()`, else noop (depends on T029)
- [X] T031 [US2] Update `handle_normal_key` in `src/main.rs`: replace the `j` handler to call `app.kanban_try_move_down()` when `focus_area == Kanban`; add an else branch routing `h/l/k/j` to `move_memo_left/right/up/down` when `focus_area == Memo`; scope the `is_reorder` flag to `FocusArea::Kanban` only (depends on T029, T030)
- [X] T032 [US2] Implement full `render_memo_panel` in `src/ui.rs`: add `const MEMO_ITEM_WIDTH: u16 = 24`; compute `items_per_row = max(1, memo_area.width / MEMO_ITEM_WIDTH)` and write `app.memo_cols = items_per_row as usize`; render each memo as a `Paragraph` widget at its computed grid position; highlight focused item with blue/bold style when `focus_area == Memo` (depends on T011, T030)

**Checkpoint**: Pressing `j` at the bottom of any kanban column moves focus into the memo panel; `h/l/j/k` navigate between items; `k` from the top row returns to kanban.

---

## Phase 5: User Story 3 — Delete Memo Items (Priority: P3)

**Goal**: Pressing `D` while a memo item is focused removes it from the panel and from storage.

**Independent Test**: Unit test verifies `delete_focused_memo` removes the correct item and clamps focus; manual test: create two memos, focus the first, press `D`, confirm only one remains.

### Tests for User Story 3 (write first, confirm each FAILS)

- [X] T033 [P] [US3] Write test `memo_delete_removes_correct_item` in `src/app.rs` `#[cfg(test)]`: build app with two memos (id 1 and id 2), set `focused_memo = 0`, call `delete_focused_memo`; assert `board.memos.len() == 1` and the remaining memo has `id == 2`
- [X] T034 [P] [US3] Write test `memo_delete_clamps_focus` in `src/app.rs` `#[cfg(test)]`: build app with two memos, `focused_memo = 1`, delete; assert `focused_memo == 0` (clamped after deletion)
- [X] T035 [P] [US3] Write test `memo_delete_last_item_leaves_empty` in `src/app.rs` `#[cfg(test)]`: build app with one memo, delete it; assert `board.memos.is_empty()` and `focused_memo == 0`

### Implementation for User Story 3

- [X] T036 [US3] Add `delete_focused_memo(&mut self)` method to `AppState` in `src/app.rs`: remove `board.memos[focused_memo]` (no-op if empty); call `clamp_memo_focus` (depends on T010)
- [X] T037 [US3] Update `handle_normal_key` in `src/main.rs` to route `D` to `app.delete_focused_memo()` when `app.focus_area == FocusArea::Memo`; ensure the existing task-delete `D` path only fires when `focus_area == Kanban` (depends on T036)

**Checkpoint**: Create two memos; focus the first and press `D`; confirm the memo is gone; restart the app and confirm it is not present in `current.log`.

---

## Phase 6: User Story 4 — Layout and Visual Separation (Priority: P4)

**Goal**: The memo panel visually spans only below the 4 kanban columns (not below the Detail panel), occupies ~20% of screen height, and items wrap left-to-right.

**Independent Test**: Open the app at 80×24 and ≥160×48 terminal sizes; confirm the memo panel height is visually ~20%, the Detail panel is unaffected, and items wrap when the panel is narrower than the total item count.

### Implementation for User Story 4

- [X] T038 [US4] Verify the `Constraint::Percentage(20)` for `memo_area` in `src/ui.rs` renders correctly at 80-column minimum; update the too-small terminal guard (`area.height < 10` → adjust threshold to account for memo panel minimum usable height, e.g. `< 12`) (depends on T011)
- [X] T039 [US4] Confirm `render_memo_panel` in `src/ui.rs` correctly wraps items: given `items_per_row = 3` and 5 memos, items 0-2 appear on row 0 and items 3-4 on row 1; assert memo panel renders only visible rows (no scroll) and items beyond the panel height are not rendered (depends on T032)
- [X] T040 [US4] Verify that `render_detail_panel` in `src/ui.rs` is unaffected by the kanban/memo vertical split by confirming it still receives the full `detail_area` height (the `detail_area` is derived from the horizontal split of `board_area`, which is unchanged) (depends on T019)

**Checkpoint**: At all tested terminal sizes, the Detail panel occupies its full right-side height; the memo panel sits below the 4 kanban columns only; items wrap at row boundaries without scrolling.

---

## Phase 7: Polish & Cross-Cutting Concerns

**Purpose**: Final integration, status bar updates, and validation.

- [X] T041 Update the status bar hint text in `render_status_bar` in `src/ui.rs`: when `focus_area == Kanban`, append `j:memo` to existing hints; when `focus_area == Memo`, show `"a:add  e:title  E:detail  D:del  hjkl:nav  k:back  q:quit"`
- [X] T042 Audit `src/main.rs` `run_app` to confirm `is_reorder` / `reorder_save_pending` logic is never triggered for memo key events; add guard if needed
- [X] T043 Run `cargo test && cargo clippy -- -D warnings && cargo fmt --check`; fix all warnings and formatting issues

---

## Dependencies & Execution Order

### Phase Dependencies

```
Phase 1 (Setup)          → no dependencies
Phase 2 (Foundational)   → depends on Phase 1 — BLOCKS all user stories
Phase 3 (US1)            → depends on Phase 2
Phase 4 (US2)            → depends on Phase 2 (can start in parallel with US1 after Phase 2)
Phase 5 (US3)            → depends on Phase 2 (can start in parallel with US1/US2 after Phase 2)
Phase 6 (US4)            → depends on Phase 3 and Phase 4 (needs render_memo_panel complete)
Phase 7 (Polish)         → depends on all prior phases complete
```

### User Story Dependencies

| Story | Depends On | Notes |
|-------|-----------|-------|
| US1 (P1) | Phase 2 complete | Standalone — create + view |
| US2 (P2) | Phase 2 complete | Can develop in parallel with US1 |
| US3 (P3) | Phase 2 complete | Can develop in parallel with US1/US2 |
| US4 (P4) | US1 + US2 render (T032) | Needs full render_memo_panel and layout for visual verification |

### Within Each Phase

1. Tests MUST be written first and confirmed **RED** before any implementation
2. Foundational tasks (T006-T009) complete before AppState tasks (T010-T011)
3. AppState changes (T010) before app methods (T017-T018, T029-T030)
4. App methods before key routing (T021, T031, T037)
5. App methods before UI updates that read them (T019, T032)

---

## Parallel Execution Examples

### Phase 2 Parallel (Foundational Tests)

```
T002: memo_new_has_correct_fields        ← write in parallel
T003: board_state_with_memos_round_trips ← write in parallel
T004: board_state_missing_memos_field    ← write in parallel
T005: save_and_load_board_with_memos     ← write in parallel
```

Then: T006 + T007 + T008 in parallel (independent model.rs additions)

### Phase 3 Parallel (US1 Tests)

```
T012: memo_create_adds_to_board        ← write in parallel
T013: memo_create_focuses_new_memo     ← write in parallel
T014: memo_create_empty_title_rejected ← write in parallel
T015: memo_edit_title                  ← write in parallel
T016: memo_edit_detail                 ← write in parallel
```

Then T017 before T018; T019 and T020 in parallel.

### Phase 4 Parallel (US2 Tests)

```
T022–T028: all navigation tests ← write in parallel (independent test functions)
```

Then T029 before T030; T031 and T032 after T030.

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Baseline verification
2. Complete Phase 2: Foundational (data model + AppState)
3. Complete Phase 3: User Story 1 (create + view)
4. **STOP and VALIDATE**: Create a memo, view it in Detail panel, restart app and confirm persistence
5. Ship as MVP

### Incremental Delivery

1. Phase 1 + Phase 2 → Foundation ready
2. Phase 3 → Memos can be created and viewed (MVP)
3. Phase 4 → Memos are navigable via hjkl (full interactive use)
4. Phase 5 → Memos can be deleted (lifecycle complete)
5. Phase 6 → Visual layout verified at all terminal sizes
6. Phase 7 → Polish and CI gates pass

---

## Task Count Summary

| Phase | Tasks | Tests | Impl |
|-------|-------|-------|------|
| Phase 1: Setup | 1 | — | 1 |
| Phase 2: Foundational | 10 | 4 | 6 |
| Phase 3: US1 (P1) | 9 | 5 | 4 |
| Phase 4: US2 (P2) | 11 | 7 | 4 |
| Phase 5: US3 (P3) | 5 | 3 | 2 |
| Phase 6: US4 (P4) | 3 | — | 3 |
| Phase 7: Polish | 3 | — | 3 |
| **Total** | **42** | **19** | **23** |

## Notes

- All [P] tasks have no file conflicts with each other within the same phase
- Each user story is independently completable and testable after Phase 2
- `cargo test` must pass GREEN before any implementation task begins (Constitution Principle I)
- Memo items are NEVER written to daily log files (`YYYYMMDD.log`) — verify this invariant in Phase 7
- No new crate dependencies required — zero `Cargo.toml` changes
