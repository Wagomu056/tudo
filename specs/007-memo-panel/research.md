# Research: Memo Panel

**Feature**: 007-memo-panel
**Date**: 2026-03-01

## Decision 1: Layout — Splitting the Kanban Area

**Decision**: Add a vertical split inside `kanban_area` only. The existing horizontal split (75% kanban / 25% detail) is unchanged. Inside `kanban_area`, a new vertical split creates the 4 columns (~80% height) above the memo panel (~20% height).

**Rationale**: The ratatui `Layout` API composes cleanly — replace the flat `columns` layout on `kanban_area` with a two-step layout: vertical split of `kanban_area` into `columns_area` + `memo_area`, then horizontal split of `columns_area` into the 4 columns. Zero impact on the detail panel.

**Alternatives considered**:
- Splitting the entire `board_area` vertically (rejected: would shrink the detail panel height, contradicting the spec requirement to leave detail unchanged).
- A separate top-level vertical section (same as above; rejected for same reason).

## Decision 2: Items-Per-Row Cache

**Decision**: Store `memo_cols: usize` in `AppState` (default = 4). The `render_memo_panel` function computes the actual items-per-row from the panel width at render time and writes it into `app.memo_cols`. Navigation methods (`move_memo_left/right/up/down`) read `app.memo_cols`.

**Rationale**: Navigation key handling runs outside the render cycle and cannot compute layout geometry. Caching the last computed value in `AppState` is the minimal approach — one field, no new abstractions. The cached value is always fresh (set before any navigation can use it, since rendering happens every event loop iteration).

**Alternatives considered**:
- Fixed constant per row (rejected: would break on narrow terminals or when panel width changes).
- Passing `memo_cols` as a parameter to navigation methods (rejected: pollutes `AppState` API surface with rendering concerns).

**Item width**: Each memo tile uses a fixed `MEMO_ITEM_WIDTH = 24` characters (title truncated to 20 + 2 border chars + 2 padding). `memo_cols = panel_width / MEMO_ITEM_WIDTH`. Minimum 1.

## Decision 3: Cross-Boundary Navigation

**Decision**:
- Kanban → Memo: `move_down` in Normal mode checks if already at the bottom of the column (or column is empty). If so, it switches `focus_area` to `FocusArea::Memo` and clamps `focused_memo` to 0. If the memo panel has no items, this still switches focus (allowing `a` to be used immediately).
- Memo → Kanban: `move_memo_up` checks if `focused_memo < memo_cols` (i.e., currently on the first row). If so, it switches `focus_area` to `FocusArea::Kanban`. Focus returns to the previously focused column and card.
- Kanban `h/l` only moves between kanban columns; no lateral boundary crosses into memo (memo is below, not beside).
- Memo `h/l/j` respect boundaries: `h` at index 0 is a no-op; `l` at last item is a no-op; `j` at last row is a no-op.

**Rationale**: Mirrors the existing kanban boundary behaviour (`move_down` already does nothing at the last item). Extending it to cross into a new focus area is the natural extension the spec requires.

**Alternatives considered**:
- Tab key for focus switching (rejected: spec explicitly requires hjkl extension).
- Wrap-around navigation (rejected: not mentioned in spec, inconsistent with existing kanban boundary behaviour).

## Decision 4: Backward-Compatible Storage

**Decision**: Add `#[serde(default)]` to `memos: Vec<Memo>` and `next_memo_id: u64` in `BoardState`. The version field remains `1` — memos are additive; no migration needed.

**Rationale**: An existing `current.log` without a `memos` field deserializes cleanly with an empty `Vec` and `next_memo_id = 1`. No version bump, no migration tooling, no backward-incompatible schema change. This satisfies Constitution Principle IV (Data Portability, backward-compatible within MAJOR version).

**Alternatives considered**:
- Version bump to `2` (rejected: unnecessary — serde defaults handle the gap cleanly).
- Separate `memos.log` file (rejected: adds storage complexity; spec says memos live in `current.log`).

## Decision 5: AppMode Reuse for Memo Input

**Decision**: Reuse `AppMode::InputTitle` and `AppMode::InputDetail` for memo input. Add `is_memo: bool` to `InputState` to distinguish memo editing from task editing in `confirm_input`.

**Rationale**: The input popup UI is identical for tasks and memos (a text box with a title). No new render path needed. The `confirm_input` dispatch just branches on `input.is_memo`. Follows Constitution Principle II (no premature abstractions — three similar code blocks are preferable).

**Alternatives considered**:
- New `AppMode::InputMemoTitle` / `AppMode::InputMemoDetail` variants (rejected: unnecessary duplication of render logic; `is_memo` flag is sufficient disambiguation).

## Decision 6: Memo Item Display Format

**Decision**: Each memo item is rendered as a bordered box with the title text inside, truncated to fit. The focused item uses the same highlight style as focused tasks (blue background, white text, bold). Non-focused items use default style. Borders use `DarkGray` for non-focused, `Blue` for focused.

**Rationale**: Consistent with existing kanban column and card styling. Minimal new styling decisions.

## Decision 7: `move_down` Refinement

**Decision**: The existing `move_down` on `AppState` is modified to cross into memo panel. To keep the API clean, the method now returns a `MoveResult` enum (`Moved`, `EnteredMemo`, `AtBoundary`) — or more simply, we keep the existing `move_down` for kanban-only movement and add a new `try_move_down` for the cross-boundary version used by the key handler.

**Simplified decision**: Modify `handle_normal_key` in `main.rs` to check focus area and dispatch accordingly. `AppState::move_down` remains kanban-only. A new `AppState::kanban_move_down_or_enter_memo` method handles the cross-boundary logic. This keeps `move_down` testable in isolation.

## Summary: No New Dependencies

All design decisions use existing crates (`ratatui`, `serde`, `serde_json`, `chrono`). No new `Cargo.toml` changes needed.
