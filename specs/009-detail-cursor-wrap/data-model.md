# Data Model: Detail Field Cursor Movement and Unicode-Aware Text Wrapping

## Modified Entities

### InputState (src/model.rs)

Current fields:
- `buffer: String` — raw text being edited
- `is_create: bool` — whether creating new item or editing existing
- `is_memo: bool` — whether editing a memo or task

**New field**:
- `cursor: usize` — byte offset into `buffer` indicating the cursor position. Always on a valid UTF-8 char boundary. Range: `0..=buffer.len()`.

**Invariants**:
- `cursor` MUST always be `<= buffer.len()`
- `cursor` MUST always be on a char boundary (`buffer.is_char_boundary(cursor)` is true)
- When `buffer` is cleared, `cursor` resets to 0
- When `buffer` is set (e.g., loading existing detail), `cursor` is set to `buffer.len()` (end of text)

### InputState Methods (new)

| Method | Signature | Behavior |
|--------|-----------|----------|
| `move_left` | `&mut self` | Move cursor to previous char boundary; no-op if at 0 |
| `move_right` | `&mut self` | Move cursor to next char boundary; no-op if at end |
| `move_home` | `&mut self` | Set cursor to 0 |
| `move_end` | `&mut self` | Set cursor to `buffer.len()` |
| `insert_char` | `&mut self, c: char` | Insert char at cursor, advance cursor by `c.len_utf8()` |
| `delete_char_back` | `&mut self` | Remove char before cursor, move cursor left; no-op if at 0 |
| `clear` | `&mut self` | Reset buffer to empty, cursor to 0 |
| `set_buffer` | `&mut self, s: String` | Set buffer contents, cursor to end |

## Unchanged Entities

### Task / Memo (src/model.rs)
- `detail: String` — no changes to storage format or structure

### BoardState (src/model.rs)
- No changes

### AppState (src/model.rs)
- `input: InputState` — existing field, type gains new `cursor` field (backward-compatible Default)

## No New Persistent Entities

Cursor position is transient UI state. No changes to `current.log` or any stored data format.
