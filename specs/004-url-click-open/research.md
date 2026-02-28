# Research: Clickable URLs in TUI

**Feature**: 004-url-click-open
**Date**: 2026-02-28

---

## 1. Mouse Event API (crossterm 0.28.1)

**Decision**: Use `Event::Mouse(MouseEvent)` arm in the existing event-loop match.

**Rationale**: Mouse capture is already enabled at startup (`EnableMouseCapture`) and disabled at shutdown (`DisableMouseCapture`). The only missing piece is handling the event instead of discarding it in the `_ => {}` wildcard. No new dependencies needed.

**Key types** (imported via `ratatui::crossterm::event`):

```rust
use ratatui::crossterm::event::{Event, MouseEvent, MouseEventKind, MouseButton};
```

`MouseEvent` fields:
- `column: u16` — screen column of the click
- `row: u16` — screen row of the click
- `kind: MouseEventKind` — `Down(MouseButton)`, `Up(MouseButton)`, `Drag(MouseButton)`, `Moved`, `ScrollDown`, `ScrollUp`
- `modifiers: KeyModifiers`

Left-click pattern:
```rust
Event::Mouse(MouseEvent { kind: MouseEventKind::Down(MouseButton::Left), column, row, .. })
```

**Alternatives considered**: None — crossterm is the existing backend; there is no alternative event source.

---

## 2. Opening URLs in the System Browser

**Decision**: Use `std::process::Command` with `#[cfg(target_os)]` conditionals. No new crate.

**Rationale**: The constitution requires preferring stdlib over adding new crate dependencies. Opening a URL in the default browser needs ~15 lines of code covering macOS (`open`), Linux (`xdg-open`), and Windows (`start`). The `open` crate (MIT, zero deps) provides the same abstraction but adds a dependency that is not justified by a 15-line implementation.

**Implementation sketch**:
```rust
pub fn open_url(url: &str) -> std::io::Result<()> {
    #[cfg(target_os = "macos")]
    std::process::Command::new("open").arg(url).spawn()?;
    #[cfg(target_os = "linux")]
    std::process::Command::new("xdg-open").arg(url).spawn()?;
    #[cfg(target_os = "windows")]
    std::process::Command::new("cmd").args(["/C", "start", "", url]).spawn()?;
    Ok(())
}
```

`spawn()` (not `status()`) is used so the UI is not blocked waiting for the browser process to exit.

**Alternatives considered**:
- `open` crate (v5.3.3, MIT): Well-maintained, zero deps, identical internal implementation. Rejected because YAGNI — stdlib suffices for 3 platforms.
- `webbrowser` crate: More features (output suppression, hardened mode). Rejected as over-engineered for this use case.

---

## 3. URL Detection from Text (stdlib only)

**Decision**: Implement `extract_url_spans(text: &str) -> Vec<(usize, usize)>` using `str::find` and `char_indices` iteration. No regex crate.

**Rationale**: The regex crate is a significant compile-time addition and not currently in the dependency tree. URL detection for `http://` and `https://` schemes only is straightforward with a two-step algorithm: (1) find scheme start with `str::find`, (2) scan forward consuming valid URL characters with bracket-balance tracking.

**Algorithm**:
1. Find next `http://` or `https://` occurrence (prefer the earlier of the two).
2. From the `://` end, iterate characters tracking:
   - Hard terminators (whitespace, control chars): stop immediately.
   - Open brackets `([`: increment depth, continue.
   - Close brackets `)]`: if depth > 0 decrement; else stop (unmatched closer → URL ends before it).
   - Soft terminators (`. , ; ! ?`): stop only if the next character is a hard terminator or end-of-string; otherwise continue (e.g., `example.com/path.html` keeps the dot).
   - All other RFC 3986 valid characters: continue, update `last_safe_end`.
3. Record `(scheme_start, last_safe_end)`.
4. Advance search position to `last_safe_end` and repeat.

**Return type**: `Vec<(usize, usize)>` — byte offsets into the original `&str` (start inclusive, end exclusive), compatible with `&text[start..end]`.

**Alternatives considered**:
- `linkify` crate: Handles more schemes and heuristics. Rejected; http/https-only scope makes it unnecessary.
- `url` crate: Parses and validates URLs, but does not extract spans from free text. Rejected for wrong problem.

---

## 4. Mapping Screen Coordinates to URL Positions

**Decision**: Populate a `Vec<UrlHitRegion>` in `AppState` during each render frame. Check this vec on mouse click events.

**Rationale**: ratatui renders each frame from scratch; the render function has exact knowledge of each widget's bounding `Rect`. By computing URL hit regions during rendering — when coordinate data is readily available — we avoid duplicating the layout calculation logic in the event handler.

The flow:
1. `AppState.clickable_urls` is cleared at the start of each `draw` callback.
2. `render_column` and `render_detail_panel` compute screen positions for each URL they render and append `UrlHitRegion` entries to `app.clickable_urls`.
3. On `Event::Mouse(Down(Left))`, the event handler iterates `app.clickable_urls` to find the first region containing `(column, row)` and calls `open_url`.

**Coordinate computation for list items** (render_column):
- Each visible task title occupies one screen row.
- Item at visual index `vi` (0-based, after scroll offset) → `row = area.y + 1 + vi` (inside 1-cell border).
- Text starts at `area.x + 1` (left border).
- `extract_url_spans` on the (possibly truncated) title text gives character offsets; since the text is ASCII-compatible and single-line, byte offset ≈ display column offset.

**Coordinate computation for detail panel** (render_detail_panel):
- The detail `Paragraph` uses word-wrap with `available_width = area.width - 2`.
- A helper `simulate_wrap(text, width)` computes `(char_pos → (display_col, display_row))` by iterating words and line-breaking at width, mirroring ratatui's Word wrap algorithm.
- URLs in the detail text are located in the wrapped layout and recorded with their screen coordinates.

**Alternatives considered**:
- Computing URL positions only when a click occurs (lazy): Would require re-running both layout and wrap logic in the event handler. Rejected; duplicates render-side code.
- Annotating ratatui `Span` metadata: ratatui does not expose rendered position metadata. Not applicable.
