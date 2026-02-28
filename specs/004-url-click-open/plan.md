# Implementation Plan: Clickable URLs in TUI

**Branch**: `004-url-click-open` | **Date**: 2026-02-28 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `/specs/004-url-click-open/spec.md`

## Summary

Enable left-click on any `http://` or `https://` URL visible in tudo's kanban column titles or detail panel to open that URL in the system default browser. Mouse capture is already enabled at the crossterm layer but all mouse events are currently discarded. The implementation adds: (1) a URL extraction function using stdlib string scanning, (2) a non-blocking browser launcher using `std::process::Command`, (3) a `UrlHitRegion` struct and `clickable_urls` vec in `AppState` populated during each render frame, and (4) a `Event::Mouse(Down(Left))` arm in the event loop that looks up the clicked region and launches the browser.

No new crate dependencies are required.

## Technical Context

**Language/Version**: Rust stable ≥ 1.75 (via rustup)
**Primary Dependencies**: ratatui 0.29 (crossterm 0.28.1 backend), serde + serde_json 1.0, chrono 0.4, directories 5
**Storage**: JSON files (`current.log`, `YYYYMMDD.log`) — unchanged by this feature
**Testing**: `cargo test` (built-in Rust test framework); integration tests in `tests/`
**Target Platform**: macOS, Linux, Windows (standard terminal emulators with mouse reporting)
**Project Type**: TUI desktop application
**Performance Goals**: ≤ 16 ms per frame (60 fps); URL region scan per frame is O(total rendered characters) — negligible
**Constraints**: No new crate dependencies; `unwrap()`/`expect()` forbidden in production paths
**Scale/Scope**: Single user, local files; feature scope is entirely in-process

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Status | Notes |
|-----------|--------|-------|
| **I. Test-First** | ✅ PASS | All tasks below write failing tests before implementation code |
| **II. Simplicity / YAGNI** | ✅ PASS | No new crates; `UrlHitRegion` is minimal; URL detection is a single focused module |
| **III. TUI-First** | ✅ PASS | Mouse support is optional per constitution; keyboard workflows unaffected; URL click offloads to `spawn()` (non-blocking) |
| **IV. Data Portability** | ✅ PASS | No storage format changes; `UrlHitRegion` is transient in-memory state only |
| **V. Correctness Over Performance** | ✅ PASS | Error from `open_url` propagated via `Result`; no `unwrap()` in production paths; `spawn()` used (non-blocking, avoids UI freeze) |

**Post-design re-check**: No violations found. `clickable_urls` field is transient and reset each frame — consistent with Principle IV. The `simulate_wrap` helper duplicates ratatui's layout logic for coordinate mapping; justified because ratatui exposes no rendered-position API (Complexity Tracking below).

## Project Structure

### Documentation (this feature)

```text
specs/004-url-click-open/
├── plan.md              # This file
├── research.md          # Phase 0 output
├── data-model.md        # Phase 1 output
├── quickstart.md        # Phase 1 output
└── tasks.md             # Phase 2 output (/speckit.tasks command)
```

### Source Code Changes

```text
src/
├── main.rs      # Add Event::Mouse arm; clear clickable_urls before draw; import MouseEvent types
├── app.rs       # Add clickable_urls: Vec<UrlHitRegion> field to AppState
├── model.rs     # Add UrlHitRegion struct
├── ui.rs        # Populate clickable_urls in render_column and render_detail_panel
└── url.rs       # NEW: extract_url_spans, open_url, simulate_wrap (internal helper)

tests/
└── url_tests.rs # NEW: unit tests for extract_url_spans, open_url, simulate_wrap, hit-region logic
```

**Structure Decision**: Single-project layout (Option 1). All changes are additive to the existing `src/` structure. A new `src/url.rs` module keeps URL logic isolated and independently testable per Principle I.

## Complexity Tracking

| Situation | Why Needed | Simpler Alternative Rejected Because |
|-----------|------------|--------------------------------------|
| `simulate_wrap` helper (mirrors ratatui word-wrap) | Screen coordinates of wrapped detail text are needed to build `UrlHitRegion` entries; ratatui exposes no rendered-position API | Computing positions lazily (at click time) requires the same wrap simulation but duplicates the layout `Rect` data that is only available inside the `draw` callback |

---

## Implementation Phases

### Phase A: URL Extraction Module (`src/url.rs`)

**New file** containing two public functions. Written test-first.

#### `extract_url_spans(text: &str) -> Vec<(usize, usize)>`

Scans `text` for `http://` and `https://` URLs and returns byte-offset spans `(start, end)` such that `&text[start..end]` is the URL.

**Algorithm** (see research.md §3 for full details):
1. Find next scheme start with `str::find("http://")` and `str::find("https://")`, take minimum.
2. From `://` end, iterate `char_indices()` maintaining:
   - `bracket_depth: i32` for `(`, `)`, `[`, `]`
   - `last_safe_end: usize` for the last confirmed valid URL end
3. Stop on whitespace/control (hard terminator) or unmatched close bracket.
4. On soft terminators (`. , ; ! ?`), stop only if next char is a hard terminator or end-of-string.
5. Advance search past `last_safe_end` and repeat.

#### `open_url(url: &str) -> std::io::Result<()>`

Launches the URL in the system default browser without blocking. Uses `spawn()` (not `status()`).

```rust
#[cfg(target_os = "macos")]   // open <url>
#[cfg(target_os = "linux")]   // xdg-open <url>
#[cfg(target_os = "windows")] // cmd /C start "" <url>
```

Returns `Ok(())` immediately after spawning. If `spawn()` fails, returns `Err(io::Error)`.

#### `simulate_wrap(text: &str, width: u16) -> Vec<(usize, u16, u16)>`

Internal helper (not `pub`). Returns `(byte_offset, display_col, display_row)` for the start of each character, mirroring ratatui's `Wrap { trim: true }` word-wrap algorithm. Used by the render module to compute screen coordinates for URLs in the detail panel.

---

### Phase B: Data Model (`src/model.rs` + `src/app.rs`)

**`UrlHitRegion`** (new struct in `model.rs`):

```rust
pub struct UrlHitRegion {
    pub row:       u16,
    pub col_start: u16,
    pub col_end:   u16,
    pub url:       String,
}
```

**`AppState`** (modified in `app.rs`):
- Add field: `pub clickable_urls: Vec<UrlHitRegion>`
- Initialize to `Vec::new()` in `AppState::new()`

---

### Phase C: Render Integration (`src/ui.rs`)

Modify the `render` function to clear `app.clickable_urls` before delegating to sub-renderers.

**`render_column` changes**:
- For each visible task item at visual index `vi`:
  - Compute `item_row = area.y + 1 + vi as u16`
  - Call `extract_url_spans` on the truncated title string
  - For each span `(start, end)`:
    - `col_start = area.x + 1 + char_count(&title[..start]) as u16`
    - `col_end   = area.x + 1 + char_count(&title[..end])   as u16`
    - Push `UrlHitRegion { row: item_row, col_start, col_end, url: title[start..end].to_string() }`

**`render_detail_panel` changes**:
- Compute `text_width = area.width - 2` (inside border)
- Call `simulate_wrap(detail_text, text_width)` → char position map
- Call `extract_url_spans(detail_text)` → spans
- For each span, look up start/end in the char position map → `(display_col, display_row)`
- `row = area.y + 1 + display_row + title_line_offset`
- Push `UrlHitRegion` entries for each display row the URL occupies (URLs spanning a line break get two entries pointing to the same `url`)

---

### Phase D: Event Loop Integration (`src/main.rs`)

Add `MouseEvent`, `MouseEventKind`, `MouseButton` to the existing `use ratatui::crossterm::event::` import.

In `run_app`, before `terminal.draw(...)`:
```rust
app.clickable_urls.clear();
```

Add `Event::Mouse` arm to the event-loop match:
```rust
Event::Mouse(mouse_event) => {
    if let MouseEventKind::Down(MouseButton::Left) = mouse_event.kind {
        handle_left_click(app, mouse_event.column, mouse_event.row);
    }
}
```

**`handle_left_click(app, col, row)`** (new function in `main.rs` or factored into `app.rs`):
```
for region in &app.clickable_urls {
    if region.row == row && col >= region.col_start && col < region.col_end {
        if let Err(e) = open_url(&region.url) {
            app.status_msg = Some(format!("Cannot open URL: {}", e));
        }
        return;
    }
}
```

---

## Task Ordering (test-first, dependency order)

1. Write failing tests for `extract_url_spans` → implement → green
2. Write failing tests for `open_url` (mock / `#[cfg(test)]` shim) → implement → green
3. Write failing tests for `simulate_wrap` → implement → green
4. Add `UrlHitRegion` to `model.rs` + `clickable_urls` to `app.rs` → update existing model tests
5. Write failing tests for render-side `UrlHitRegion` population helpers → implement render changes → green
6. Write failing tests for `handle_left_click` logic → implement event-loop changes → green
7. `cargo clippy -- -D warnings` and `cargo fmt` pass
8. Manual end-to-end verification (see quickstart.md)
