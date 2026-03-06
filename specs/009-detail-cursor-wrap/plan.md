# Implementation Plan: Detail Field Cursor Movement and Unicode-Aware Text Wrapping

**Branch**: `009-detail-cursor-wrap` | **Date**: 2026-03-06 | **Spec**: [spec.md](spec.md)
**Input**: Feature specification from `/specs/009-detail-cursor-wrap/spec.md`

## Summary

Add cursor-based text editing to the Detail input field (currently append-only) with left/right arrow, Home/End navigation, and positional insert/delete. Apply Unicode-aware text wrapping to the detail panel so that long text (including CJK full-width characters) wraps correctly at the panel boundary. Display a visible cursor at the correct position, including on wrapped lines.

## Technical Context

**Language/Version**: Rust stable >= 1.75 (via rustup)
**Primary Dependencies**: ratatui 0.29 (crossterm 0.28.1 backend), serde + serde_json 1.0, chrono 0.4, directories 5, unicode-width (already in use)
**Storage**: JSON file (`current.log` in platform data directory) — unchanged
**Testing**: `cargo test` (unit tests in-module, integration tests in `tests/`)
**Target Platform**: macOS / Linux / Windows terminal emulators (80+ columns)
**Project Type**: TUI desktop application
**Performance Goals**: <= 16 ms per frame (60 fps TUI rendering)
**Constraints**: Must render correctly at 80 columns minimum width
**Scale/Scope**: Single-user local application

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Status | Notes |
|-----------|--------|-------|
| I. Test-First | PASS | Tests for cursor movement, wrapping, and rendering will be written before implementation |
| II. Simplicity / YAGNI | PASS | Reuses existing `wrap_str` for detail wrapping; `InputState` extended minimally with a `cursor` field |
| III. TUI-First | PASS | Feature is entirely TUI-focused; keyboard-driven cursor navigation |
| IV. Data Portability | PASS | No storage format changes; cursor is UI-only state |
| V. Correctness Over Performance | PASS | Unicode grapheme/width handling ensures correctness for CJK; no `unwrap()` in new code |

## Project Structure

### Documentation (this feature)

```text
specs/009-detail-cursor-wrap/
├── plan.md              # This file
├── research.md          # Phase 0 output
├── data-model.md        # Phase 1 output
├── quickstart.md        # Phase 1 output
└── tasks.md             # Phase 2 output (/speckit.tasks)
```

### Source Code (repository root)

```text
src/
├── app.rs               # Input event handling — add cursor key handlers, positional insert/delete
├── model.rs             # Add cursor position field to InputState
├── ui.rs                # Update render_detail_panel for cursor display; apply wrap_str to detail text
├── input.rs             # Extend with cursor movement utility methods
└── lib.rs               # No changes expected

tests/
└── (existing test files) # Add tests for cursor movement and detail wrapping
```

**Structure Decision**: Single-project Rust binary. All changes are within the existing `src/` files. No new files or modules needed.
