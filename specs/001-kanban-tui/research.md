# Research: tudo Kanban TUI

**Feature**: 001-kanban-tui
**Date**: 2026-02-27
**Status**: Complete — all decisions resolved

---

## Decision 1: TUI Layout Strategy

**Decision**: Split terminal horizontally into a 75% kanban area (4 equal
columns) and a 25% right detail panel. Each column uses `Constraint::Ratio(1,4)`.

```rust
// Top-level split: columns area + detail panel
let main = Layout::default()
    .direction(Direction::Horizontal)
    .constraints([Constraint::Percentage(75), Constraint::Percentage(25)])
    .split(area);

// Equal 4-column split
let cols = Layout::default()
    .direction(Direction::Horizontal)
    .constraints([
        Constraint::Ratio(1, 4), Constraint::Ratio(1, 4),
        Constraint::Ratio(1, 4), Constraint::Ratio(1, 4),
    ])
    .split(main[0]);
```

**Rationale**: `Constraint::Ratio` produces pixel-perfect equal widths that
adapt to terminal resize. The 75/25 split reads comfortably at 80 columns
(60 chars for board, 20 for detail) and scales well on wider terminals.

**Alternatives considered**:
- Fixed character widths: breaks on terminal resize.
- `Percentage(25)` × 4 for columns: functionally identical to Ratio(1,4) but
  semantically less clear.

---

## Decision 2: Focused Card Highlight

**Decision**: Apply `Style::default().bg(Color::Blue).fg(Color::White)` to the
`ListItem` at the focused index within the focused column. Non-focused columns
render with a dimmer block border to indicate inactive state.

```rust
let items: Vec<ListItem> = tasks.iter().enumerate().map(|(i, t)| {
    let style = if col_focused && i == selected_idx {
        Style::default().bg(Color::Blue).fg(Color::White)
    } else {
        Style::default()
    };
    ListItem::new(t.title.as_str()).style(style)
}).collect();
```

**Rationale**: `REVERSED` modifier works on all terminal color schemes;
explicit blue background also works and is more visible. Either is acceptable;
explicit color chosen for clarity.

---

## Decision 3: Text Input Popup (no extra crates)

**Decision**: Implement a minimal manual string buffer for title/detail
editing. Render as a centered `Block` overlay using ratatui's `Clear` widget.
Support: char insert at end, `Backspace` to delete last char, `Enter` to
confirm, `Esc` to cancel. No cursor-position navigation (cursor always at end).

```rust
fn centered_rect(pct_x: u16, pct_y: u16, area: Rect) -> Rect {
    let vert = Layout::default().direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - pct_y) / 2),
            Constraint::Percentage(pct_y),
            Constraint::Percentage((100 - pct_y) / 2),
        ]).split(area);
    Layout::default().direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - pct_x) / 2),
            Constraint::Percentage(pct_x),
            Constraint::Percentage((100 - pct_x) / 2),
        ]).split(vert[1])[1]
}

// In render():
f.render_widget(Clear, popup_area);
f.render_widget(
    Paragraph::new(input_state.buffer.as_str())
        .block(Block::default().title("Title").borders(Borders::ALL)),
    popup_area,
);
```

**Rationale**: Manual buffer handling is ~30 lines of code vs. adding a
`tui-textarea` dependency. For single-line title input, append-only editing
covers 95% of use cases (Simplicity/YAGNI). Multi-line detail editing can
be done by editing `current.log` directly (Data Portability principle).

**Alternatives considered**:
- `tui-textarea` crate: full multi-line editor with cursor navigation;
  rejected for v1 as over-engineered for a single-user local MVP. Deferred
  as a minor-version enhancement.
- `tui-input` crate: single-line only; similar complexity to manual buffer
  without meaningful benefit over a raw `String` buffer.

---

## Decision 4: Storage Format

**Decision**:
- `current.log`: a single JSON file (pretty-printed) containing the full
  `BoardState` struct. Overwritten atomically on every board mutation.
- `YYYYMMDD.log`: append-only JSON Lines (one compact JSON object per line).
  Each line is a `DoneEntry` with title, detail, and completion timestamp.

**Rationale**: `serde_json` is the de-facto Rust JSON crate (1 billion+
downloads). Pretty-printed `current.log` is human-readable without any tools
(SC-006). JSON Lines format for `YYYYMMDD.log` allows simple `append(true)`
writes without re-parsing the file.

**Alternatives considered**:
- TOML: excellent for config files but TOML arrays of tables are verbose and
  the format is not append-friendly for log entries.
- Plain text: insufficient structure for reliable round-trip (de)serialization.
- SQLite: binary format, violates Data Portability principle (IV).

---

## Decision 5: Date & Time Handling

**Decision**: Use the `chrono` crate (version `0.4`, `serde` feature enabled).

Key operations:
- Current timestamp: `Local::now()`
- Today's date: `Local::now().date_naive()`
- Same-day comparison: `task.done_at.date_naive() == Local::now().date_naive()`
- Log filename: `Local::now().format("%Y%m%d").to_string()` + `.log`

```toml
chrono = { version = "0.4", features = ["serde"] }
```

**Rationale**: `chrono` is the standard Rust date/time crate (350M+ downloads),
provides `NaiveDate` comparison, RFC 3339 serialization via serde, and date
formatting — all without an async runtime.

**Alternatives considered**:
- `std::time::SystemTime`: UNIX timestamps only; no date arithmetic or
  human-readable formatting without significant custom code.
- `time` crate: modern alternative; fewer downloads; chrono chosen for
  broader ecosystem familiarity.

---

## Decision 6: Task Identity

**Decision**: Sequential `u64` ID stored in `BoardState.next_id`. No external
UUID crate.

**Rationale**: IDs only need to be unique within a single user's local data
file. A monotonic counter satisfies this requirement with zero dependencies
and produces human-readable IDs in log files.

**Alternatives considered**:
- `uuid` crate: unnecessary for a single-user local app.
- Vec index: fragile — indices shift on deletion, causing bugs.

---

## Decision 7: File Storage Location

**Decision**: Store `current.log` and `YYYYMMDD.log` in the current working
directory (`./`) where `tudo` is launched.

**Rationale**: Zero-configuration discovery — users see their data files in
the same directory they run the app from, directly satisfying SC-006. No
platform-specific path lookup code or extra crate (`dirs`) required.

**Alternatives considered**:
- `~/.local/share/tudo/` (XDG base dir): Linux standard; requires `dirs`
  crate or manual `$HOME` lookup. Adds complexity for v1.
- Configurable via CLI flag: YAGNI for v1.

---

## Crate Dependency List

```toml
[dependencies]
ratatui  = { version = "0.28", features = ["crossterm"] }
# crossterm is re-exported by ratatui; explicit dep only if direct use needed
serde      = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
chrono     = { version = "0.4", features = ["serde"] }
```

**Total new crates**: 4 (ratatui, serde, serde_json, chrono) — all major,
widely-used crates with MIT/Apache-2.0 licenses. crossterm is an implicit
dependency of ratatui and need not be listed separately unless directly
imported.
