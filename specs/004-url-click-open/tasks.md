# Tasks: Clickable URLs in TUI

**Input**: Design documents from `/specs/004-url-click-open/`
**Prerequisites**: plan.md ✅, spec.md ✅, research.md ✅, data-model.md ✅, quickstart.md ✅

**Tests**: Included — the project constitution (Principle I: Test-First) is NON-NEGOTIABLE. All tasks follow Red-Green-Refactor: write a failing test, confirm it fails, implement, confirm it passes.

**Organization**: Tasks are grouped by user story to enable independent implementation and testing of each story.

## Format: `[ID] [P?] [Story?] Description`

- **[P]**: Can run in parallel (different files, no conflicting dependencies)
- **[Story]**: Which user story this task belongs to (US1, US2, US3)
- Exact file paths are included in every description

---

## Phase 1: Setup

**Purpose**: Create new files and wire them into the module tree before any code is written.

- [X] T001 Create `src/url.rs` as an empty module file and add `pub mod url;` to `src/lib.rs`
- [X] T002 Create `tests/url_tests.rs` with a single `use tudo::url;` smoke-import (verifies module is reachable); confirm `cargo test` compiles

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Data structures and the core URL-extraction algorithm that every user story depends on.

**⚠️ CRITICAL**: No user story work can begin until this phase is complete.

- [X] T003 [P] Write failing tests for `UrlHitRegion` struct in `tests/model_tests.rs`: assert fields `row: u16`, `col_start: u16`, `col_end: u16`, `url: String` are publicly accessible; confirm `cargo test` fails to compile
- [X] T004 [P] Add `UrlHitRegion { row: u16, col_start: u16, col_end: u16, url: String }` (derive `Debug, Clone, PartialEq`) to `src/model.rs`; confirm T003 tests pass
- [X] T005 Write failing test in `tests/model_tests.rs`: construct `AppState::new()` and assert `.clickable_urls` is an empty `Vec`; confirm test fails
- [X] T006 Add `pub clickable_urls: Vec<UrlHitRegion>` field (initialized to `Vec::new()`) to `AppState` in `src/app.rs`; confirm T005 test passes; update any existing `AppState` construction in tests that would now fail to compile
- [X] T007 Write failing unit tests for `extract_url_spans` in `tests/url_tests.rs` covering: plain URL in middle of text, URL at start, URL at end, URL with query params and fragment, URL followed by period (sentence punctuation stripped), URL in parentheses (matched parens included, unmatched close paren terminates), `http://` and `https://` both detected, no URL returns empty vec; confirm all tests fail (function does not exist yet)
- [X] T008 Implement `pub fn extract_url_spans(text: &str) -> Vec<(usize, usize)>` in `src/url.rs` using `str::find` scheme search and `char_indices` scanning with bracket-depth tracking and soft-terminator logic (see research.md §3); confirm all T007 tests pass

**Checkpoint**: Foundation ready — user story implementation can now begin

---

## Phase 3: User Story 1 — Click URL to Open Browser (Priority: P1) 🎯 MVP

**Goal**: Left-clicking a URL in a kanban column task title opens it in the system default browser.

**Independent Test**: Add a task with title containing `https://example.com`, run tudo, left-click the URL text in the kanban list; verify browser opens the page.

> **NOTE: Write tests FIRST (T009, T010, T013), confirm they FAIL before implementing (T011, T012, T014, T015)**

- [X] T009 [P] [US1] Write failing tests for `open_url` in `tests/url_tests.rs`: (a) assert `open_url` returns `Ok(())` for a valid https URL on the current platform, (b) assert function signature is `pub fn open_url(url: &str) -> std::io::Result<()>`; confirm tests fail (function does not exist)
- [X] T010 [P] [US1] Write failing tests for a pure helper `list_item_url_regions(title: &str, item_row: u16, text_x: u16) -> Vec<UrlHitRegion>` in `tests/url_tests.rs`: given title `"See https://example.com here"` at row 3, x-offset 1, assert correct `col_start`, `col_end`, `row`, and `url` values; confirm tests fail (function does not exist)
- [X] T011 [US1] Implement `pub fn open_url(url: &str) -> std::io::Result<()>` in `src/url.rs` using `std::process::Command::spawn()` with `#[cfg(target_os)]` for macOS (`open`), Linux (`xdg-open`), Windows (`cmd /C start "" <url>`); confirm T009 tests pass
- [X] T012 [US1] Implement `pub fn list_item_url_regions(title: &str, item_row: u16, text_x: u16) -> Vec<UrlHitRegion>` in `src/url.rs`: calls `extract_url_spans(title)`, converts byte offsets to char-count column offsets using `title[..start].chars().count()`; confirm T010 tests pass
- [X] T013 [US1] Write failing test for `handle_left_click` behavior in `tests/url_tests.rs`: given an `AppState` with one `clickable_urls` entry `{row:5, col_start:4, col_end:20, url:"https://x.com"}`, assert that calling the click-handling logic with `(col=10, row=5)` attempts to open the URL (use a flag or inspect the call path); confirm test fails
- [X] T014 [US1] In `src/main.rs`: add `MouseEvent`, `MouseEventKind`, `MouseButton` to the existing `use ratatui::crossterm::event::` import; add `app.clickable_urls.clear();` immediately before the `terminal.draw(...)` call in `run_app`
- [X] T015 [US1] In `src/main.rs`: add `Event::Mouse(mouse_event) =>` arm to the event-loop match; inside it, on `MouseEventKind::Down(MouseButton::Left)` call `handle_left_click(&mut app, mouse_event.column, mouse_event.row)`; implement `handle_left_click`: iterate `app.clickable_urls`, on first hit call `url::open_url(&region.url)`, on `Err` set `app.status_msg = Some(format!("Cannot open URL: {e}"))`, return early; also call `list_item_url_regions` inside `render_column` for each visible item and push results into `app.clickable_urls`; confirm T013 test passes and `cargo test` is green

**Checkpoint**: User Story 1 is fully functional — clicking a URL in the kanban list opens the browser

---

## Phase 4: User Story 2 — URL Detection in Task Content (Priority: P2)

**Goal**: URLs anywhere in task text (including the detail panel) are all recognized as clickable regions.

**Independent Test**: Add a task with a URL in the detail text; run tudo; focus the task so the detail panel shows; left-click the URL in the detail panel; verify browser opens.

> **NOTE: Write tests FIRST (T016, T018), confirm they FAIL before implementing (T017, T019)**

- [X] T016 [P] [US2] Write failing unit tests for `simulate_wrap(text: &str, width: u16) -> Vec<(usize, u16, u16)>` (returns `(byte_offset, display_col, display_row)` for each char) in `tests/url_tests.rs`: test cases include a short line (no wrap), a line exactly at width (no wrap), a line that wraps at a word boundary, and a multi-word text that wraps twice; confirm tests fail
- [X] T017 [US2] Implement `fn simulate_wrap(text: &str, width: u16) -> Vec<(usize, u16, u16)>` (crate-private) in `src/url.rs`: iterate words, track current display column and row, break to next row when adding the next word would exceed `width`; confirm T016 tests pass
- [X] T018 [US2] Write failing tests for `detail_url_regions(text: &str, available_width: u16, base_row: u16, base_col: u16) -> Vec<UrlHitRegion>` in `tests/url_tests.rs`: given detail text with a URL that does not wrap, assert correct single `UrlHitRegion`; given detail text where URL spans a line break, assert two `UrlHitRegion` entries (one per display row) with matching `url` string; confirm tests fail
- [X] T019 [US2] Implement `pub fn detail_url_regions(text: &str, available_width: u16, base_row: u16, base_col: u16) -> Vec<UrlHitRegion>` in `src/url.rs` using `simulate_wrap` and `extract_url_spans`; call it inside `render_detail_panel` in `src/ui.rs` and push results into `app.clickable_urls`; confirm T018 tests pass and `cargo test` is green

**Checkpoint**: User Stories 1 AND 2 are functional — URLs in both list titles and detail panel are clickable

---

## Phase 5: User Story 3 — Non-Disruptive Interaction (Priority: P3)

**Goal**: Clicking a URL does not alter the focused task, column, or any other navigation state.

**Independent Test**: Select a task; click a URL in a different task's title (in the list view); verify the originally selected task remains selected and the browser opens.

> **NOTE: Write tests FIRST (T020, T021), confirm they FAIL before implementing (T022)**

- [X] T020 [P] [US3] Write failing test in `tests/url_tests.rs`: construct `AppState` with `focused_col = 1`, `focused_card[1] = 2`, add a `UrlHitRegion` entry, call `handle_left_click` with matching coordinates, assert `focused_col` is still `1` and `focused_card[1]` is still `2` after the call; confirm test fails (or passes trivially if state already preserved — verify with an intentionally wrong assertion first)
- [X] T021 [P] [US3] Write failing test in `tests/url_tests.rs`: call `handle_left_click` with coordinates that do NOT match any `UrlHitRegion`; assert no `status_msg` is set and no panic occurs; confirm test fails
- [X] T022 [US3] Audit `handle_left_click` in `src/main.rs`: confirm it returns immediately after finding a URL hit without calling any navigation-mutating methods on `AppState`; add `return;` guard explicitly if not already present; add a `_ => {}` no-op for non-URL clicks to confirm non-matching clicks are silently ignored; confirm T020 and T021 tests pass

**Checkpoint**: All three user stories are fully functional and non-interfering

---

## Phase 6: Polish & Cross-Cutting Concerns

- [X] T023 [P] Run `cargo clippy -- -D warnings` on the full workspace; fix any warnings introduced by this feature (unused imports, missing docs on pub items if required by clippy, etc.)
- [X] T024 [P] Run `cargo fmt` on all modified files (`src/main.rs`, `src/app.rs`, `src/model.rs`, `src/ui.rs`, `src/url.rs`, `tests/url_tests.rs`, `tests/model_tests.rs`); confirm `cargo fmt --check` passes
- [X] T025 Run full `cargo test` and confirm every test passes with no failures, no warnings
- [X] T026 Manual end-to-end validation per `specs/004-url-click-open/quickstart.md`: add a task with a URL in the title, add a task with a URL in the detail, click each, verify browser opens; verify clicking non-URL text causes no browser launch

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies — start immediately
- **Foundational (Phase 2)**: Depends on Setup — **BLOCKS all user stories**
- **US1 (Phase 3)**: Depends on Foundational (T003–T008 complete)
- **US2 (Phase 4)**: Depends on Foundational + US1 (needs `render_column` integration as reference pattern)
- **US3 (Phase 5)**: Depends on US1 (`handle_left_click` must exist)
- **Polish (Phase 6)**: Depends on all user story phases complete

### Within Each Phase (strict ordering)

```
T003, T004 → can run in parallel (different files)
T005 → T006 (test then implement field)
T007 → T008 (test then implement extract_url_spans)
T009, T010 → can run in parallel (different test functions)
T011 → after T009
T012 → after T010
T013 → after T011, T012
T014 → after T013 (needs imports in place before event arm)
T015 → after T014
T016 → T017 (test then implement simulate_wrap)
T018 → T019 (test then implement detail regions)
T020, T021 → can run in parallel (different test functions)
T022 → after T020, T021
T023, T024 → can run in parallel
T025 → after T023, T024
T026 → after T025
```

---

## Parallel Execution Examples

### Phase 2 (Foundational)

```
Parallel: T003 (UrlHitRegion tests) || T007 (extract_url_spans tests)
Then:     T004 (UrlHitRegion impl)  || T008 (extract_url_spans impl)
Then:     T005 → T006 (AppState field)
```

### Phase 3 (US1)

```
Parallel: T009 (open_url tests) || T010 (list region tests)
Then:     T011 (open_url impl)  || T012 (list region impl)
Then:     T013 → T014 → T015
```

### Phase 4 (US2)

```
Sequential: T016 → T017 → T018 → T019
```

### Phase 5 (US3)

```
Parallel: T020 (no-state-change test) || T021 (non-URL click test)
Then:     T022 (implement guard)
```

---

## Implementation Strategy

### MVP (User Story 1 Only)

1. Complete Phase 1: Setup (T001–T002)
2. Complete Phase 2: Foundational (T003–T008) — **CRITICAL**
3. Complete Phase 3: User Story 1 (T009–T015)
4. **STOP and VALIDATE**: Click a URL in a list column title → browser opens
5. Ship if acceptable; continue to Phase 4 for detail panel support

### Incremental Delivery

1. Phase 1 + Phase 2 → Foundation ready
2. Phase 3 → MVP: click URLs in list titles → demo/ship
3. Phase 4 → Detail panel URLs clickable → demo/ship
4. Phase 5 → State preservation guaranteed → demo/ship
5. Phase 6 → Code quality gates → merge-ready

---

## Notes

- `[P]` tasks operate on different files or test functions and have no conflicting edits
- Each phase ends with a verifiable checkpoint — stop there to validate before proceeding
- The constitution (Principle I) mandates that every test task be committed and confirmed failing before its paired implementation task begins
- `simulate_wrap` is `fn` (crate-private) since it mirrors ratatui internals; it is tested via `#[cfg(test)]` visibility or by making it `pub(crate)`
- `handle_left_click` may be factored into `src/app.rs` instead of `src/main.rs` if that improves testability — the plan allows either location
