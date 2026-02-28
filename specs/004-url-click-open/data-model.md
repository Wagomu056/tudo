# Data Model: Clickable URLs in TUI

**Feature**: 004-url-click-open
**Date**: 2026-02-28

---

## New Entities

### UrlHitRegion

Represents a screen region occupied by a URL that was rendered in the current frame. Populated during each render pass; consumed by the mouse click handler.

**Location**: `src/model.rs`

```
UrlHitRegion {
    row:       u16    // Screen row (0-based absolute terminal row)
    col_start: u16    // Leftmost screen column of the URL text (inclusive)
    col_end:   u16    // One past the rightmost column (exclusive)
    url:       String // The full URL string (http:// or https://)
}
```

**Validation rules**:
- `col_end > col_start` (non-empty region).
- `url` starts with `"http://"` or `"https://"`.
- Not persisted; rebuilt every render frame.

**State transitions**: No transitions — value is created during render and consumed (or discarded) during event handling.

---

## Modified Entities

### AppState (src/app.rs)

One field added:

```
AppState {
    // ... existing fields unchanged ...
    clickable_urls: Vec<UrlHitRegion>   // Rebuilt each frame by render functions
}
```

**Lifecycle**:
1. Initialized to `Vec::new()` in `AppState::new()` (or default).
2. Cleared (`clickable_urls.clear()`) at the start of each `terminal.draw` callback, before any widget rendering.
3. Populated by `render_column` and `render_detail_panel` as each URL span is encountered.
4. Read by the `Event::Mouse(Down(Left))` handler to find which URL (if any) was clicked.
5. Never serialized to disk.

---

## URL Span (Internal, not a struct)

Used as the return type of `extract_url_spans`:

```
(usize, usize)   // (start_byte_offset, end_byte_offset) into the source &str
```

Both offsets are byte indices into the source string (compatible with `&str[start..end]`). The caller is responsible for converting to display column offsets when appending to `clickable_urls`.

---

## No Storage Changes

`UrlHitRegion` and `clickable_urls` are transient in-memory state. No changes to `BoardState`, `Task`, `DoneEntry`, or any JSON file format. `BoardState.version` is unchanged.
