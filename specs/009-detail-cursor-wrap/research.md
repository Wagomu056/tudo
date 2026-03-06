# Research: Detail Field Cursor Movement and Unicode-Aware Text Wrapping

## R1: Cursor Position Representation in Rust Strings

**Decision**: Use byte offset (`usize`) as the cursor position, with helper methods that navigate by character (grapheme cluster) boundaries.

**Rationale**: Rust strings are UTF-8 encoded. Byte offsets are the native indexing unit (`String::insert`, `String::remove` operate on byte indices). Converting between character index and byte offset on every keystroke adds overhead and complexity. By storing the byte offset directly, insert/delete operations are O(1) lookups + O(n) shift. Character-boundary navigation uses `str::char_indices()` or iterates grapheme clusters.

**Alternatives considered**:
- Character index (count of chars): Requires scanning from start on every insert/delete to find byte position. Adds complexity without benefit since `String` methods need byte offsets.
- Grapheme cluster index: Same issue as character index, plus requires `unicode-segmentation` crate. For CJK text, char boundaries and grapheme boundaries are typically identical.

**Note**: For simplicity and correctness with CJK, navigating by `char` boundaries (not grapheme clusters) is sufficient. CJK characters are single `char` values. The `unicode-segmentation` crate would only be needed for complex scripts (Thai, Devanagari) or emoji sequences, which are not a primary use case.

## R2: Cursor Navigation Methods

**Decision**: Implement `move_left`, `move_right`, `move_home`, `move_end` methods on `InputState` that adjust the byte-offset cursor.

**Rationale**: These are simple operations that iterate `char_indices()` to find the previous/next char boundary. No external crate needed.

**Implementation sketch**:
- `move_left`: Find the char boundary before current cursor using `self.buffer[..self.cursor].char_indices().last()`
- `move_right`: Find the next char boundary after cursor using `self.buffer[self.cursor..].chars().next()` and add its `len_utf8()`
- `move_home`: Set cursor to 0
- `move_end`: Set cursor to `self.buffer.len()`

## R3: Positional Insert and Delete

**Decision**: Use `String::insert(byte_offset, char)` for insertion and `String::remove(byte_offset)` for backspace-delete.

**Rationale**: These are standard library methods that handle UTF-8 correctly. `insert` shifts bytes right; `remove` shifts bytes left and returns the removed char. Both panic if the index is not on a char boundary, which our cursor navigation guarantees.

**Alternatives considered**:
- Rope data structure: Overkill for detail text (typically < 1KB). Standard `String` operations are fast enough.
- Split buffer (gap buffer): Unnecessary complexity for the expected text sizes.

## R4: Unicode-Aware Wrapping for Detail Panel

**Decision**: Reuse the existing `wrap_str` function (already in `src/ui.rs`) for detail text wrapping. Apply it per-line (split by `\n` first, then wrap each line).

**Rationale**: `wrap_str` already uses `unicode_width::UnicodeWidthChar::width()` to correctly handle CJK full-width characters (2 columns) vs ASCII (1 column). It handles the edge case where a full-width character would be split at the boundary by moving it to the next line.

**Alternatives considered**:
- ratatui's built-in `Wrap { trim: false }`: Currently used, but does not guarantee correct CJK width handling in all ratatui versions. Pre-wrapping with our own `wrap_str` gives us full control and testability.
- textwrap crate: External dependency; our `wrap_str` already handles the use case.

## R5: Cursor Position on Wrapped Lines

**Decision**: Compute the cursor's visual (row, col) position by running `wrap_str` on the text up to the cursor position, then counting the resulting lines and the display width of the last line.

**Rationale**: This approach reuses existing wrapping logic and guarantees consistency between displayed text and cursor position. The cursor row determines vertical scroll offset; the cursor column determines the `set_cursor_position` call for the terminal.

**Implementation sketch**:
1. Split buffer by `\n` to get logical lines
2. For each logical line, use `wrap_str` to get visual lines
3. Track which visual line contains the cursor byte offset
4. The cursor column = display width of text on that visual line up to the cursor position

## R6: Vertical Scrolling

**Decision**: Track a `scroll_offset` (number of visual lines to skip) in UI rendering. Adjust when cursor moves above or below the visible area.

**Rationale**: Simple approach that keeps scroll state in the rendering function (or `AppState` if persistence across frames is needed). The visible height is `area.height - 2` (borders) minus title lines.

## R7: Cursor Rendering

**Decision**: Use ratatui's `Frame::set_cursor_position(x, y)` to display the terminal cursor at the computed position during `InputDetail` mode.

**Rationale**: This is the standard ratatui approach for showing an edit cursor. It leverages the terminal emulator's native cursor, which is already visible and blinking. No custom rendering needed.

**Note**: The existing `InputTitle` mode likely already uses this pattern (or the trailing `_` character approach seen in the codebase). We should align detail cursor rendering with the same approach for consistency.
