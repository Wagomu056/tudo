# Tasks: Detail Field Cursor Movement and Unicode-Aware Text Wrapping

**Input**: Design documents from `/specs/009-detail-cursor-wrap/`
**Prerequisites**: plan.md, spec.md, research.md, data-model.md

**Tests**: Included per Constitution Principle I (Test-First). Tests MUST be written and fail before implementation.

**Organization**: Tasks are grouped by user story to enable independent implementation and testing of each story.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3)
- Include exact file paths in descriptions

---

## Phase 1: Setup

**Purpose**: No new project setup needed. Existing project structure is sufficient.

(No tasks — project already initialized with all required dependencies including `unicode-width`.)

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Add `cursor` field to `InputState` and implement cursor navigation methods that all user stories depend on.

**CRITICAL**: No user story work can begin until this phase is complete.

- [X] T001 Add `cursor: usize` field to `InputState` and write unit tests for cursor invariants (cursor <= buffer.len(), always on char boundary) in `src/model.rs`
- [X] T002 Write failing tests for `move_left`, `move_right`, `move_home`, `move_end` methods with ASCII, CJK, and mixed text in `src/model.rs`
- [X] T003 Implement `move_left`, `move_right`, `move_home`, `move_end` methods on `InputState` to pass T002 tests in `src/model.rs`
- [X] T004 Write failing tests for `insert_char` and `delete_char_back` at various cursor positions (beginning, middle, end) with ASCII and CJK characters in `src/model.rs`
- [X] T005 Implement `insert_char` and `delete_char_back` methods on `InputState` to pass T004 tests in `src/model.rs`
- [X] T006 Update `clear` method to reset cursor to 0, add `set_buffer` method that sets cursor to end of buffer in `src/model.rs`
- [X] T007 Update all call sites that use `push_char`, `pop_char`, and direct `buffer` assignment to use new cursor-aware methods (`insert_char`, `delete_char_back`, `set_buffer`) in `src/main.rs` and `src/app.rs`

**Checkpoint**: `InputState` now supports cursor-based editing. All existing tests still pass. New cursor methods have full test coverage.

---

## Phase 3: User Story 1 - Cursor Movement in Detail Field (Priority: P1) MVP

**Goal**: Users can move cursor left/right with arrow keys, insert/delete characters at any position, and use Home/End keys during detail editing.

**Independent Test**: Enter Detail input mode, type text, use arrow keys to navigate, insert and delete characters at various positions. Verify Japanese character navigation works correctly.

### Tests for User Story 1

> **NOTE: Write these tests FIRST, ensure they FAIL before implementation**

- [X] T008 [US1] Write failing test: Left/Right arrow keys in InputDetail mode move cursor via `move_left`/`move_right` in `src/main.rs` (or test module)
- [X] T009 [US1] Write failing test: Home/End keys in InputDetail mode call `move_home`/`move_end` in `src/main.rs` (or test module)

### Implementation for User Story 1

- [X] T010 [US1] Add `KeyCode::Left` and `KeyCode::Right` handlers for `InputTitle`/`InputDetail` mode that call `app.input.move_left()` and `app.input.move_right()` in `src/main.rs`
- [X] T011 [US1] Add `KeyCode::Home` and `KeyCode::End` handlers for `InputTitle`/`InputDetail` mode that call `app.input.move_home()` and `app.input.move_end()` in `src/main.rs`
- [X] T012 [US1] Replace `app.input.push_char(c)` with `app.input.insert_char(c)` and `app.input.pop_char()` with `app.input.delete_char_back()` in the input mode key handler in `src/main.rs`
- [X] T013 [US1] Update `Ctrl+j` newline insertion to use `app.input.insert_char('\n')` instead of `app.input.push_char('\n')` in `src/main.rs`

**Checkpoint**: Cursor movement and positional editing work in InputDetail (and InputTitle) mode. Arrow keys, Home, End, insert, and backspace all operate at cursor position.

---

## Phase 4: User Story 2 - Unicode-Aware Text Wrapping in Detail Panel (Priority: P2)

**Goal**: Detail text wraps at panel boundary using display width, correctly handling CJK full-width characters (2 columns each).

**Independent Test**: Enter detail text exceeding panel width with ASCII, Japanese, and mixed content. Verify lines break at correct visual boundaries with no character splitting.

### Tests for User Story 2

> **NOTE: Write these tests FIRST, ensure they FAIL before implementation**

- [X] T014 [P] [US2] Write failing tests for wrapping detail text with `wrap_str`: multi-line detail (split by `\n` then wrap each line), ASCII-only, CJK-only, and mixed content in `src/ui.rs` test module
- [X] T015 [P] [US2] Write failing test: full-width character at wrap boundary (only 1 column remaining) moves to next line in `src/ui.rs` test module

### Implementation for User Story 2

- [X] T016 [US2] Update `render_detail_panel` to pre-wrap detail text using `wrap_str` per logical line (split by `\n`, then wrap each segment to `available_width`) instead of relying solely on ratatui's `Wrap` in `src/ui.rs`
- [X] T017 [US2] Update `render_input_popup` to wrap the input buffer display using `wrap_str` for correct CJK handling when detail text is long in `src/ui.rs`

**Checkpoint**: Detail panel and input popup correctly wrap long text. Japanese text wraps at the right visual column.

---

## Phase 5: User Story 3 - Visible Cursor Position Indicator (Priority: P3)

**Goal**: Cursor is visually rendered at the correct position during detail editing, including on wrapped lines and after full-width characters.

**Independent Test**: Enter Detail input mode, type mixed ASCII/Japanese text that wraps, move cursor around. Verify the cursor indicator appears at the correct row and column.

### Tests for User Story 3

> **NOTE: Write these tests FIRST, ensure they FAIL before implementation**

- [X] T018 [US3] Write failing test for a helper function `cursor_visual_position(buffer: &str, cursor: usize, wrap_width: usize) -> (row, col)` that computes the visual row and column of the cursor in wrapped text, testing with ASCII, CJK, and multi-line content in `src/ui.rs` test module

### Implementation for User Story 3

- [X] T019 [US3] Implement `cursor_visual_position` helper that splits text by `\n`, wraps each line with `wrap_str`, and finds the visual (row, col) of the given byte offset in `src/ui.rs`
- [X] T020 [US3] Update `render_input_popup` to call `frame.set_cursor_position(x, y)` at the computed cursor position (replacing the trailing `_` approach) during `InputDetail` and `InputTitle` modes in `src/ui.rs`
- [X] T021 [US3] Add scroll offset logic: if cursor row exceeds visible height, adjust vertical offset so the cursor line is visible, and render only the visible portion of wrapped lines in `src/ui.rs`

**Checkpoint**: Cursor is visible at the correct position in the input popup, including on wrapped lines. Panel scrolls to keep cursor in view.

---

## Phase 6: Polish & Cross-Cutting Concerns

**Purpose**: Final validation and cleanup

- [X] T022 Run `cargo clippy -- -D warnings` and fix any new warnings
- [X] T023 Run `cargo test` and verify all existing and new tests pass
- [ ] T024 Run quickstart.md manual testing validation (cursor movement, wrapping, cursor display)

---

## Dependencies & Execution Order

### Phase Dependencies

- **Phase 1 (Setup)**: Skipped — no setup needed
- **Phase 2 (Foundational)**: No dependencies — start immediately. BLOCKS all user stories
- **Phase 3 (US1)**: Depends on Phase 2 completion
- **Phase 4 (US2)**: Depends on Phase 2 completion. Can run in parallel with Phase 3
- **Phase 5 (US3)**: Depends on Phase 2 AND Phase 4 (needs wrapping logic for cursor position calculation)
- **Phase 6 (Polish)**: Depends on all user stories being complete

### User Story Dependencies

- **User Story 1 (P1)**: Depends only on Foundational (Phase 2)
- **User Story 2 (P2)**: Depends only on Foundational (Phase 2) — can run in parallel with US1
- **User Story 3 (P3)**: Depends on US2 (needs `wrap_str` integration to compute cursor visual position)

### Within Each User Story

- Tests MUST be written and FAIL before implementation
- Implementation tasks are sequential within each story
- Story complete before moving to next priority (unless parallelizing US1 and US2)

### Parallel Opportunities

- T014 and T015 (US2 tests) can run in parallel
- Phase 3 (US1) and Phase 4 (US2) can run in parallel after Phase 2
- T010, T011 (US1 key handlers) could be parallelized if in separate match arms

---

## Parallel Example: User Stories 1 and 2

```text
# After Phase 2 (Foundational) is complete, launch US1 and US2 in parallel:

# Stream A — User Story 1 (Cursor Movement):
Task: T008 — Write failing test for Left/Right arrow keys
Task: T009 — Write failing test for Home/End keys
Task: T010 — Implement Left/Right key handlers
Task: T011 — Implement Home/End key handlers
Task: T012 — Replace push_char/pop_char with cursor-aware methods
Task: T013 — Update Ctrl+j newline insertion

# Stream B — User Story 2 (Unicode Wrapping):
Task: T014 — Write failing tests for detail wrapping
Task: T015 — Write failing test for full-width boundary
Task: T016 — Update render_detail_panel wrapping
Task: T017 — Update render_input_popup wrapping
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 2: Foundational (cursor field + methods)
2. Complete Phase 3: User Story 1 (key handlers for cursor movement)
3. **STOP and VALIDATE**: Test cursor movement independently
4. Users can now edit detail text with cursor navigation

### Incremental Delivery

1. Foundational → Cursor-aware InputState ready
2. Add US1 → Cursor movement works → MVP!
3. Add US2 → Detail text wraps correctly with CJK support
4. Add US3 → Visual cursor indicator on wrapped lines
5. Each story adds value without breaking previous stories

---

## Notes

- [P] tasks = different files, no dependencies
- [Story] label maps task to specific user story for traceability
- Constitution Principle I: All tests written before implementation (Red-Green-Refactor)
- Constitution Principle V: No `unwrap()` in new production code; use pattern matching or `?`
- Commit after each task or logical group
- Stop at any checkpoint to validate story independently
