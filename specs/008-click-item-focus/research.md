# Research: Click-to-Focus Items

## R1: Hit-region tracking for task and memo items

**Decision**: Store layout rectangles computed during `render()` in `AppState`, similar to the existing `clickable_urls` pattern. Each frame, clear and recompute `clickable_tasks` and `clickable_memos` vectors that map screen regions to (column_index, task_index) or memo_index.

**Rationale**: The codebase already follows this exact pattern for URL click regions — `app.clickable_urls` is cleared every frame and rebuilt during rendering. Extending this approach to task/memo items keeps the architecture consistent and avoids introducing new rendering abstractions.

**Alternatives considered**:
- **Recalculate layout on click**: Would duplicate layout logic from `ui.rs` into `app.rs`. Fragile — any layout change would need to be mirrored.
- **Store Rect areas from ratatui**: Could store the `Rect` for each column/memo cell, but a simple struct with (row_start, row_end, col_start, col_end) + target index is more explicit and matches the existing `UrlHitRegion` style.

## R2: Handling wrapped task titles (multi-line items)

**Decision**: Track the cumulative row offset per task item during rendering (already computed for URL regions in `render_column`). Each task's hit region spans from its start row to start_row + wrapped_line_count.

**Rationale**: The `render_column` function already calculates `cumulative_rows` for URL hit region computation. Extending this to produce task hit regions requires minimal additional code.

**Alternatives considered**:
- **Fixed single-line height**: Would miss clicks on wrapped items. Incorrect for the current wrapping implementation.

## R3: Input mode guard

**Decision**: Check `app.mode != AppMode::Normal` at the top of `handle_left_click` and return early (after URL check). This prevents focus changes during editing while preserving URL clicks.

**Rationale**: Simple guard clause, consistent with how keyboard input is routed by mode in `main.rs`.

**Alternatives considered**:
- **Disable all clicks in input mode**: Would break URL clicks in the detail panel during editing. Not desirable.
- **Guard in main.rs before calling handle_left_click**: Would prevent URL clicks too, since they share the same handler.
