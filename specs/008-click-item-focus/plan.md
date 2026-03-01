# Implementation Plan: Click-to-Focus Items

**Branch**: `008-click-item-focus` | **Date**: 2026-03-01 | **Spec**: [spec.md](spec.md)
**Input**: Feature specification from `/specs/008-click-item-focus/spec.md`

## Summary

Add mouse click-to-focus for tasks and memos. When a user left-clicks on a task card in any kanban column, focus moves to that task. When a user clicks a memo item, focus switches to the memo panel and selects that memo. This extends the existing `clickable_urls` pattern by adding `clickable_tasks` and `clickable_memos` hit-region vectors that are rebuilt each render frame, then checked in `handle_left_click`.

## Technical Context

**Language/Version**: Rust stable ≥ 1.75 (via rustup)
**Primary Dependencies**: ratatui 0.29 (crossterm 0.28.1 backend), serde + serde_json 1.0, chrono 0.4, directories 5
**Storage**: JSON file (`current.log` in platform data directory) — unchanged
**Testing**: `cargo test` (built-in test framework); integration tests in `tests/`
**Target Platform**: macOS / Linux / Windows terminal emulators
**Project Type**: TUI desktop app (CLI)
**Performance Goals**: ≤ 16 ms per frame (60 fps rendering)
**Constraints**: Click response must be imperceptible; no new crate dependencies
**Scale/Scope**: Single-user local app; ~4 columns × ~20 tasks, ~20 memos typical

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Status | Notes |
|-----------|--------|-------|
| I. Test-First | PASS | Tests for hit-region construction and focus-change logic will be written before implementation |
| II. Simplicity / YAGNI | PASS | Extends existing `clickable_urls` pattern; no new abstractions — just two new Vec fields and simple hit-test structs |
| III. TUI-First | PASS | Mouse support is optional enhancement per constitution; keyboard navigation remains unaffected |
| IV. Data Portability | PASS | No storage format changes; purely UI-layer feature |
| V. Correctness Over Performance | PASS | No `unwrap()` in new code paths; hit-test uses explicit bounds checking |

**Post-Phase 1 re-check**: All gates still PASS. No new dependencies, no complexity violations.

## Project Structure

### Documentation (this feature)

```text
specs/008-click-item-focus/
├── plan.md              # This file
├── research.md          # Phase 0 output
├── data-model.md        # Phase 1 output
├── quickstart.md        # Phase 1 output
└── tasks.md             # Phase 2 output (created by /speckit.tasks)
```

### Source Code (repository root)

```text
src/
├── app.rs      # AppState + handle_left_click (modified)
├── model.rs    # TaskHitRegion, MemoHitRegion (new structs)
├── ui.rs       # render_column, render_memo_panel (modified to emit hit regions)
├── main.rs     # Frame-start clearing of hit regions (modified)
├── input.rs    # Unchanged
├── lib.rs      # Unchanged
├── storage.rs  # Unchanged
└── url.rs      # Unchanged

tests/
├── model_tests.rs    # Unchanged
├── storage_tests.rs  # Unchanged
└── url_tests.rs      # Unchanged
```

**Structure Decision**: Single-project layout. All changes are within `src/` — modifying 4 existing files and adding 2 small structs to `model.rs`. No new files needed.
