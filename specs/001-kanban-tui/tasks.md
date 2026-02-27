---
description: "Task list for tudo Kanban TUI implementation"
---

# Tasks: tudo Kanban TUI

**Input**: Design documents from `specs/001-kanban-tui/`
**Prerequisites**: plan.md ✅, spec.md ✅, research.md ✅, data-model.md ✅, contracts/ ✅

**Tests**: Constitution Principle I (Test-First) is NON-NEGOTIABLE.
Tests MUST be written and verified failing before each implementation block.

**Organization**: Tasks are grouped by user story for independent delivery.

## Format: `[ID] [P?] [Story?] Description`

- **[P]**: Can run in parallel (different files, no incomplete task dependencies)
- **[Story]**: US1, US2, US3, US4 — maps to user stories in spec.md
- Exact file paths are included in every task description

## Path Conventions

- Source: `src/` at repository root
- Tests: `tests/` at repository root
- Config: repository root

---

## Phase 1: Setup

**Purpose**: Initialize the Rust project and establish shared infrastructure.

- [x] T001 Run `cargo init . --name tudo` in repository root; verify `Cargo.toml` and `src/main.rs` are created
- [x] T002 Replace `[dependencies]` section in `Cargo.toml` with: `ratatui = { version = "0.28", features = ["crossterm"] }`, `serde = { version = "1.0", features = ["derive"] }`, `serde_json = "1.0"`, `chrono = { version = "0.4", features = ["serde"] }`
- [x] T003 [P] Create empty placeholder source files: `src/model.rs`, `src/app.rs`, `src/ui.rs`, `src/input.rs`, `src/storage.rs`; add `mod model; mod app; mod ui; mod input; mod storage;` declarations to `src/main.rs`
- [x] T004 [P] Create `rustfmt.toml` at repo root with `edition = "2021"`; create `.cargo/config.toml` with `[target.'cfg(all())'] rustflags = ["-D", "warnings"]` to enforce clippy deny-warnings

**Checkpoint**: `cargo check` succeeds with no errors

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Core data types and terminal shell — MUST be complete before any user story work.

**⚠️ CRITICAL**: No user story work can begin until this phase is complete.

- [x] T005 Write failing unit tests in `tests/model_tests.rs` for `Status::next()` (Todo→Doing→Checking→Done→None) and `Status::prev()` (Done→Checking→Doing→Todo→None); verify `cargo test` reports compile or test failures
- [x] T006 Implement `Task`, `Status`, `BoardState`, `DoneEntry` structs/enums in `src/model.rs` with `#[derive(Debug, Clone, Serialize, Deserialize)]`; implement `Status::next()` and `Status::prev()` returning `Option<Status>`; run `cargo test` to confirm T005 tests pass
- [x] T007 Write additional failing tests in `tests/model_tests.rs` for `BoardState` invariants: `next_id > max task id`, `version == 1`, `task titles non-empty after trim`
- [x] T008 Add `AppState`, `AppMode`, `InputState` structs to `src/app.rs` with fields matching data-model.md (board, focused_col, focused_card, mode, input, status_msg); implement `AppState::new()` returning an empty board
- [x] T009 Implement terminal initialization and cleanup in `src/main.rs`: `enable_raw_mode()`, `execute!(stdout, EnterAlternateScreen)`, install panic hook that calls cleanup before unwinding, `disable_raw_mode()` + `LeaveAlternateScreen` on exit; run a blank draw loop that exits on `q`; verify `cargo run` shows a blank screen and exits cleanly on `q`

**Checkpoint**: `cargo test` passes all model tests; `cargo run` shows a blank screen and exits on `q`

---

## Phase 3: User Story 1 — Kanban Board Display & Navigation (Priority: P1) 🎯 MVP

**Goal**: Four-column board renders with task cards; vim-style navigation moves focus between cards and columns; detail panel shows focused card text.

**Independent Test**: Seed `current.log` with several tasks across columns, launch `tudo` — verify four labeled columns appear, the focused card is highlighted, and pressing h/j/k/l moves focus; detail panel updates on each focus change.

### Tests for User Story 1 ⚠️ Write and verify FAILING before implementation

- [x] T010 [US1] Write failing unit test in `tests/model_tests.rs` for `AppState::tasks_for_column(status)` — verify it returns only tasks with the matching `Status` in the correct order
- [x] T011 [US1] Write failing unit test in `tests/model_tests.rs` for `AppState::focused_task()` — verify it returns `Some(&Task)` for a valid focus position and `None` for an empty column

### Implementation for User Story 1

- [x] T012 [US1] Implement `AppState::tasks_for_column(status: Status) -> Vec<&Task>` in `src/app.rs` — filters `board.tasks` by status; run T010 to confirm it passes
- [x] T013 [US1] Implement `AppState::focused_task() -> Option<&Task>` in `src/app.rs` — returns task at `focused_card[focused_col]` in the current column's task list; run T011 to confirm it passes
- [x] T014 [US1] Implement `render()` top-level function in `src/ui.rs`: split `frame.area()` 75%/25% horizontally; split left area into 4 equal columns with `Constraint::Ratio(1, 4)`; call `render_column()` for each status and `render_detail_panel()` for the right area
- [x] T015 [P] [US1] Implement `render_column(frame, area, app, status)` in `src/ui.rs`: build `ListItem` vec from `tasks_for_column(status)`; highlight focused item with `Style::default().bg(Color::Blue).fg(Color::White)` when column is focused; render as `List` inside a `Block` with column title and `Borders::ALL`
- [x] T016 [P] [US1] Implement `render_detail_panel(frame, area, app)` in `src/ui.rs`: retrieve `focused_task()`; render title + detail text as `Paragraph` inside a `Block` with title "Detail" and `Borders::ALL`; show placeholder text when no card is focused
- [x] T017 [P] [US1] Implement `render_status_bar(frame, area, app)` in `src/ui.rs`: render `app.status_msg` (or hint text) in a single-line area at the bottom of the screen; update `render()` to carve out this bottom row using a vertical layout
- [x] T018 [US1] Implement Normal-mode navigation key handlers in `src/app.rs`: `h`/`←` decrements `focused_col` (min 0); `l`/`→` increments `focused_col` (max 3); `j`/`↓` increments `focused_card[col]` (clamped to column length − 1); `k`/`↑` decrements `focused_card[col]` (min 0); no-op at boundaries
- [x] T019 [US1] Wire `render()` into `terminal.draw()` in `src/main.rs`; dispatch navigation keys to `AppState` key handler; verify manual end-to-end: launch with seeded `current.log`, navigate board with keyboard

**Checkpoint**: Board display and navigation fully functional and independently testable

---

## Phase 4: User Story 2 — Task Creation & Editing (Priority: P2)

**Goal**: Pressing `a` opens a title input popup for a new task; `e` opens title edit for focused card; `E` opens detail edit; `Enter` confirms; `Esc` cancels.

**Independent Test**: Press `a`, type a title, press `Enter` — verify new card appears at bottom of Todo column. Focus the card, press `e`, change title, confirm — verify title updates on board and in detail panel. Press `E`, type detail, confirm — verify detail shows in right panel.

### Tests for User Story 2 ⚠️ Write and verify FAILING before implementation

- [x] T020 [US2] Write failing unit tests in `tests/model_tests.rs` for `InputState`: `push_char()` appends character, `pop_char()` removes last character (no-op when empty), `value()` returns current buffer string, `clear()` resets buffer to empty

### Implementation for User Story 2

- [x] T021 [US2] Implement `InputState` struct in `src/input.rs` with `buffer: String`, `is_create: bool` fields; implement `push_char()`, `pop_char()`, `value() -> &str`, `clear()` methods; run T020 to confirm passing
- [x] T022 [US2] Implement `centered_rect(pct_x, pct_y, area) -> Rect` helper in `src/ui.rs` (vertical + horizontal `Layout` nesting to produce a centred sub-rect)
- [x] T023 [US2] Implement `render_input_popup(frame, area, app)` in `src/ui.rs`: render `Clear` widget over popup area, then a `Block` with title ("Add Task" or "Edit Title" or "Edit Detail") and `Borders::ALL`, then a `Paragraph` with `app.input.value()`; only call when `app.mode != AppMode::Normal`
- [x] T024 [US2] Implement `AppState::open_create(app)` in `src/app.rs` — triggered by `a`: set `mode = AppMode::InputTitle`, `input.is_create = true`, `input.clear()`; implement `AppState::open_edit_title(app)` — triggered by `e`: set `mode = AppMode::InputTitle`, `input.is_create = false`, pre-fill `input.buffer` with focused task's current title (no-op if no focused task)
- [x] T025 [US2] Implement `AppState::open_edit_detail(app)` in `src/app.rs` — triggered by `E`: set `mode = AppMode::InputDetail`, `input.is_create = false`, pre-fill `input.buffer` with focused task's current detail (no-op if no focused task)
- [x] T026 [US2] Implement `AppState::confirm_input(app)` in `src/app.rs` — `Enter` in any Input mode: validate title non-empty after trim; if `is_create` append new `Task` to `board.tasks` with status `Todo` and `next_id`; otherwise update focused task's title or detail; reset `mode = AppMode::Normal` and `input.clear()`
- [x] T027 [US2] Implement `AppState::cancel_input(app)` in `src/app.rs` — `Esc` in any Input mode: reset `mode = AppMode::Normal` and `input.clear()` without modifying any task; add Input-mode character handlers (`KeyCode::Char(c)` → `input.push_char(c)`, `KeyCode::Backspace` → `input.pop_char()`) in `src/main.rs` event dispatch

**Checkpoint**: Task creation and editing fully functional independently

---

## Phase 5: User Story 3 — Status Lifecycle Management (Priority: P3)

**Goal**: `Enter` advances focused card one status; `BackSpace` retreats one status; `D` permanently deletes focused card; boundary statuses produce no-op.

**Independent Test**: Load app with a card in Todo, press `Enter` three times — verify card reaches Done; press `BackSpace` — verify card returns to Checking; press `D` — verify card is removed and does not appear in any column.

### Tests for User Story 3 ⚠️ Write and verify FAILING before implementation

- [x] T028 [US3] Write failing unit tests in `tests/model_tests.rs` for `AppState::advance_status()`: Todo→Doing, Doing→Checking, Checking→Done, Done→no-op; and `retreat_status()`: Done→Checking, Checking→Doing, Doing→Todo, Todo→no-op; and `delete_focused_card()` removes task from `board.tasks`

### Implementation for User Story 3

- [x] T029 [US3] Implement `AppState::advance_status()` in `src/app.rs`: get focused task, call `status.next()`, update `task.status`; if new status is Done set `task.done_at = Some(Local::now())`; no-op if no focused task or status is already Done; run T028 tests to confirm advancing cases pass
- [x] T030 [US3] Implement `AppState::retreat_status()` in `src/app.rs`: get focused task, call `status.prev()`, update `task.status`; if reverting from Done clear `task.done_at = None`; no-op if no focused task or status is Todo; run T028 tests to confirm retreating cases pass
- [x] T031 [US3] Implement `AppState::delete_focused_card()` in `src/app.rs`: remove the focused task from `board.tasks` by id; clamp `focused_card[focused_col]` to new column length; no-op if column is empty
- [x] T032 [US3] Wire `Enter` → `advance_status()`, `BackSpace` → `retreat_status()`, `D` → `delete_focused_card()` in the Normal-mode key dispatch in `src/main.rs`; all three keys are no-ops while in Input mode

**Checkpoint**: Full status lifecycle and deletion functional and independently testable

---

## Phase 6: User Story 4 — Data Persistence & Daily Done Reset (Priority: P4)

**Goal**: Board state is saved to `current.log` after every mutation; Done entries are appended to `YYYYMMDD.log` on Done transition; on launch, Done tasks from previous days are discarded; all other tasks are restored.

**Independent Test**: Create tasks, close and reopen app — verify board is fully restored. Advance a card to Done — verify entry in `YYYYMMDD.log`. Change system date to tomorrow (or mock date), relaunch — verify Done column is empty; Todo/Doing/Checking columns retain their cards.

### Tests for User Story 4 ⚠️ Write and verify FAILING before implementation

- [x] T033 [US4] Write failing integration tests in `tests/storage_tests.rs` for `storage::save_board()` + `storage::load_board()`: save a `BoardState`, load it back, assert fields are identical (round-trip)
- [x] T034 [US4] Write failing integration test in `tests/storage_tests.rs` for `storage::append_done_entry()`: call twice, read file lines, assert two valid JSON objects with correct fields
- [x] T035 [US4] Write failing integration test in `tests/storage_tests.rs` for daily filter logic: board with one Done task where `done_at` is yesterday — after applying filter, Done column is empty; Todo/Doing/Checking tasks remain

### Implementation for User Story 4

- [x] T036 [US4] Implement `storage::load_board(path: &str) -> Result<BoardState, AppError>` in `src/storage.rs`: return empty `BoardState` (version=1, next_id=0, tasks=vec![]) if file not found; return `Err` on corrupt JSON or version mismatch; run T033 to confirm round-trip passes
- [x] T037 [US4] Implement `storage::save_board(board: &BoardState, path: &str) -> Result<(), AppError>` in `src/storage.rs`: serialize with `serde_json::to_string_pretty`, write to file; run T033 round-trip test to confirm
- [x] T038 [US4] Implement `storage::append_done_entry(entry: &DoneEntry, path: &str) -> Result<(), AppError>` in `src/storage.rs`: open file with `OpenOptions::append(true).create(true)`, write compact JSON + `\n`; run T034 to confirm
- [x] T039 [US4] Create `AppError` type in `src/model.rs` (or a new `src/error.rs`) wrapping `std::io::Error` and `serde_json::Error`; implement `Display`; update all `storage` functions to return `Result<_, AppError>`
- [x] T040 [US4] Implement daily Done filter in `src/app.rs`: after loading board, call `board.tasks.retain(|t| t.status != Status::Done || t.done_at.map_or(false, |d| d.date_naive() == Local::now().date_naive()))`; save filtered board back to `current.log`; run T035 to confirm
- [x] T041 [US4] Call `storage::load_board("current.log")` in `AppState::new()` (or early in `main.rs`) and apply daily filter; surface load errors in `app.status_msg`
- [x] T042 [US4] Call `storage::save_board()` in `src/app.rs` after every mutation (`advance_status`, `retreat_status`, `delete_focused_card`, `confirm_input`); capture errors into `app.status_msg` instead of panicking
- [x] T043 [US4] Call `storage::append_done_entry()` inside `advance_status()` when the new status is Done: construct `DoneEntry { title, detail, completed_at: Local::now() }`, compute filename `format!("{}.log", Local::now().format("%Y%m%d"))`, call `append_done_entry()`; surface errors in `app.status_msg`

**Checkpoint**: Full persistence and daily Done reset working — all four user stories independently functional

---

## Phase 7: Polish & Cross-Cutting Concerns

**Purpose**: Robustness, code quality, and developer experience across all stories.

- [x] T044 [P] Add `Ctrl+C` key handling to the event loop in `src/main.rs` — trigger the same graceful shutdown as `q`
- [x] T045 [P] Handle `Event::Resize(w, h)` in the event loop in `src/main.rs` — call `terminal.autoresize()` or simply trigger a redraw; verify the app does not crash on resize
- [x] T046 [P] Add minimum terminal size guard in `render()` in `src/ui.rs`: if `frame.area().width < 40 || frame.area().height < 10`, render a single centred `Paragraph` with a minimum-size warning instead of the board
- [x] T047 Run `cargo clippy -- -D warnings` and fix all warnings in `src/`
- [x] T048 Run `cargo fmt` and ensure all source files pass `cargo fmt --check`
- [x] T049 Run `cargo test` and confirm all tests in `tests/model_tests.rs` and `tests/storage_tests.rs` pass with zero failures
- [x] T050 Walk through `specs/001-kanban-tui/quickstart.md` step by step on a clean checkout and fix any discrepancies found

---

## Dependencies & Execution Order

### Phase Dependencies

- **Phase 1 (Setup)**: No dependencies — start immediately
- **Phase 2 (Foundational)**: Depends on Phase 1 — BLOCKS all user stories
- **Phase 3 (US1)**: Depends on Phase 2 — no other story dependencies
- **Phase 4 (US2)**: Depends on Phase 2 + Phase 3 UI foundation (uses `centered_rect`)
- **Phase 5 (US3)**: Depends on Phase 2; can be worked in parallel with Phase 4 (different files)
- **Phase 6 (US4)**: Depends on Phase 2; can start after Phase 3 (storage is independent of UI)
- **Phase 7 (Polish)**: Depends on Phases 3–6 being complete

### User Story Dependencies

- **US1 (P1)**: Can start after Phase 2 — no story dependencies
- **US2 (P2)**: Can start after Phase 2; shares no code with US3/US4
- **US3 (P3)**: Can start after Phase 2; shares `src/app.rs` with US2 — coordinate if parallel
- **US4 (P4)**: Can start after Phase 2; `src/storage.rs` is independent of UI changes in US2/US3

### Within Each Phase

- Write failing tests FIRST, verify they fail, THEN implement
- Within implementation tasks: types (model.rs) → logic (app.rs) → rendering (ui.rs) → wiring (main.rs)
- All `[P]` tasks within a phase can run in parallel

---

## Parallel Opportunities

### Phase 1 (after T001+T002 complete)
```
T003: Create placeholder src files
T004: Create rustfmt.toml + .cargo/config.toml
```

### Phase 3 — User Story 1 (after T012+T013 complete)
```
T015: render_column()
T016: render_detail_panel()
T017: render_status_bar() + centered_rect()
```

### Phase 7 — Polish (all independent)
```
T044: Ctrl+C handler
T045: Resize handler
T046: Minimum size guard
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup (T001–T004)
2. Complete Phase 2: Foundational (T005–T009)
3. Complete Phase 3: US1 Board Display (T010–T019)
4. **STOP AND VALIDATE**: Navigate the board with pre-seeded `current.log`; confirm all US1 acceptance scenarios pass
5. Demo: functional kanban TUI with navigation

### Incremental Delivery

1. **Phase 1+2**: Project compiles, blank terminal loop runs
2. **Phase 3 (US1)**: Board renders, navigation works → MVP!
3. **Phase 4 (US2)**: Tasks can be created and edited → usable
4. **Phase 5 (US3)**: Cards move between statuses → full kanban workflow
5. **Phase 6 (US4)**: Persistence + daily reset → production-ready
6. **Phase 7**: Polish → release candidate

### Parallel Team Strategy (if two developers)

After Phase 2 is complete:
- **Developer A**: Phase 3 (US1) + Phase 4 (US2) sequentially
- **Developer B**: Phase 5 (US3) in `src/app.rs` + Phase 6 (US4) in `src/storage.rs`
- Coordinate on `src/app.rs` for US2 vs US3 (both modify this file)

---

## Notes

- `[P]` = different files or logically independent, safe to parallelise
- `[USN]` label maps every task to its user story for traceability
- Constitution Principle I is enforced: every implementation phase is preceded by test tasks
- `unwrap()` / `expect()` forbidden in `src/` production paths — use `?` operator or explicit `match`
- `app.status_msg` is the sole error surface; never `panic!` on I/O errors
- Commit after each checkpoint (T009, T019, T027, T032, T043)
