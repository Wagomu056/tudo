# Developer Quickstart: Memo Panel

**Feature**: 007-memo-panel
**Branch**: `007-memo-panel`

## Prerequisites

- Rust stable ≥ 1.75 (via rustup)
- `cargo test` and `cargo clippy -- -D warnings` currently pass on `main`

## Overview of Changes

This feature touches 4 existing files. No new files are created. No new crate dependencies are required.

| File | Changes |
|------|---------|
| `src/model.rs` | Add `Memo` struct, `FocusArea` enum; extend `InputState` and `BoardState` |
| `src/app.rs` | Extend `AppState` with memo fields; add memo navigation and CRUD methods; update `confirm_input` |
| `src/ui.rs` | Add memo panel to kanban area layout; add `render_memo_panel`; update detail panel and status bar |
| `src/main.rs` | Route `h/j/k/l/a/e/E/D` to memo handlers when `FocusArea::Memo` |

Test files to create or extend:
- `tests/model_tests.rs` — add Memo serialization and backward-compat tests
- `tests/storage_tests.rs` — add round-trip test with memos
- `src/app.rs` (inline tests) — add memo navigation and CRUD tests

## Implementation Order (Test-First)

Follow Red-Green-Refactor strictly (Constitution Principle I).

### Step 1 — Data Model (`src/model.rs`)

**Tests first** (`tests/model_tests.rs`):
- `memo_new_has_correct_fields` — `Memo::new(1, "title")` → `id=1, title="title", detail=""`
- `board_state_with_memos_round_trips` — serialize and deserialize `BoardState` containing memos
- `board_state_missing_memos_field_deserializes_to_empty` — parse JSON without `memos`/`next_memo_id` fields → `memos=[]`, `next_memo_id=1`

**Then implement**:
1. Add `Memo` struct with `id: u64`, `title: String`, `detail: String`; derive `Debug, Clone, Serialize, Deserialize`
2. Add `Memo::new(id, title) -> Memo`
3. Add `FocusArea` enum with `Kanban` and `Memo` variants; derive `Debug, Clone, PartialEq, Eq`
4. Add `is_memo: bool` to `InputState` (default `false`)
5. Add `memos: Vec<Memo>` with `#[serde(default)]` and `next_memo_id: u64` with `#[serde(default = "default_next_memo_id")]` to `BoardState`; add free function `fn default_next_memo_id() -> u64 { 1 }`
6. Add `BoardState::alloc_memo_id(&mut self) -> u64`

### Step 2 — App Logic (`src/app.rs`)

**Tests first** (inline `#[cfg(test)]` module in `app.rs`):

Navigation tests:
- `memo_enter_from_kanban_bottom` — `move_down` at bottom → `focus_area = Memo`
- `memo_enter_from_empty_kanban_column` — `move_down` in empty column → `focus_area = Memo`
- `memo_exit_to_kanban_first_row` — `move_memo_up` at row 0 → `focus_area = Kanban`
- `memo_move_right` — `move_memo_right` → `focused_memo` increments
- `memo_move_right_boundary` — `move_memo_right` at last item → `focused_memo` unchanged
- `memo_move_left` — `move_memo_left` → `focused_memo` decrements
- `memo_move_left_boundary` — `move_memo_left` at 0 → `focused_memo` unchanged
- `memo_move_down_row` — `move_memo_down` advances by `memo_cols`
- `memo_move_down_last_row_noop` — `move_memo_down` at last row → no change
- `memo_move_up_row` — `move_memo_up` (not row 0) subtracts `memo_cols`

CRUD tests:
- `memo_create_adds_to_board` — `open_create_memo` + `confirm_input` → memo appended
- `memo_create_focuses_new_memo` — after create, `focused_memo` points to new item
- `memo_create_empty_title_rejected` — empty title → status_msg set, no memo added
- `memo_delete_removes_item` — `delete_focused_memo` removes correct item
- `memo_delete_clamps_focus` — after delete, `focused_memo` clamped
- `memo_delete_last_item_focus_zero` — deleting last → `focused_memo = 0`
- `memo_edit_title` — `open_edit_memo_title` + `confirm_input` → title updated
- `memo_edit_detail` — `open_edit_memo_detail` + `confirm_input` → detail updated

**Then implement** in `AppState`:
1. Add `focus_area: FocusArea`, `focused_memo: usize`, `memo_cols: usize` (default 4) fields
2. Add `focused_memo_item(&self) -> Option<&Memo>` helper
3. Add `kanban_try_move_down(&mut self)` — moves down in kanban column; if at bottom (or empty), switches to `FocusArea::Memo`, sets `focused_memo = 0`
4. Add `move_memo_left/right/up/down(&mut self)` methods (up from row 0 → back to kanban)
5. Add `open_create_memo(&mut self)` — sets `input.is_create = true`, `input.is_memo = true`, mode = `InputTitle`
6. Add `open_edit_memo_title(&mut self)` and `open_edit_memo_detail(&mut self)`
7. Add `delete_focused_memo(&mut self)` and `clamp_memo_focus(&mut self)`
8. Update `confirm_input` to branch on `input.is_memo` → create/edit memo instead of task

### Step 3 — Storage (`tests/storage_tests.rs`)

**Tests first**:
- `save_and_load_board_with_memos` — round-trip `BoardState` containing `Vec<Memo>`
- `load_board_legacy_json_has_empty_memos` — load JSON string without `memos` field → `board.memos.is_empty()`

No code changes to `storage.rs` needed — `BoardState` serialization is handled automatically once model changes are done.

### Step 4 — UI (`src/ui.rs`)

**Tests**: UI rendering is not unit-tested directly; coverage from manual testing and the existing terminal size guard.

**Implement**:
1. Modify `render()`: replace `kanban_area` direct column layout with a two-step split:
   - `kanban_area` → vertical split: `[columns_area (80%), memo_area (20%)]`
   - `columns_area` → existing 4-column horizontal split
2. Add `MEMO_ITEM_WIDTH: u16 = 24` constant
3. Add `render_memo_panel(frame, memo_area, app)`:
   - Compute `items_per_row = max(1, memo_area.width / MEMO_ITEM_WIDTH)`
   - Write `app.memo_cols = items_per_row as usize`
   - Render outer block with title `" Memo "` and border
   - Render each memo item as a positioned inline widget at its grid position
   - Highlight focused item in `FocusArea::Memo` with blue/bold style
4. Modify `render_detail_panel`: when `focus_area == Memo`, read from `app.focused_memo_item()` instead of `app.focused_task()`
5. Update `render_status_bar`: show memo-specific hint when `focus_area == Memo`
6. Update popup title in `render_input_popup` for `is_memo = true` cases

### Step 5 — Key Routing (`src/main.rs`)

**Tests**: Covered by existing integration via manual testing; app.rs unit tests cover logic.

**Implement**:
1. Replace `KeyCode::Char('j')` handler in `handle_normal_key` with a call to `app.kanban_try_move_down()` when `focus_area == Kanban`.
2. When `focus_area == Memo`, dispatch `h/l/k/j/a/e/E/D` to memo methods.
3. Update `is_reorder` flag: reorder only applies in `FocusArea::Kanban`.
4. Save after every memo mutation (same pattern as existing task mutations).

## Running Tests

```bash
cargo test
cargo clippy -- -D warnings
cargo fmt --check
```

## Key Invariants to Maintain

1. `focused_memo < board.memos.len()` when memos non-empty (enforced by `clamp_memo_focus`)
2. `memo_cols >= 1` (enforced in `render_memo_panel`)
3. Memo mutations never touch `tasks`, and task mutations never touch `memos`
4. `append_done_entry` is never called for memo operations
5. No `unwrap()`/`expect()` on `Option`/`Result` in production paths (Constitution Principle V)
