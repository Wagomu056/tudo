# Implementation Plan: Status-Ordered Task Lists

**Branch**: `011-status-ordered-tasks` | **Date**: 2026-03-07 | **Spec**: [spec.md](spec.md)
**Input**: Feature specification from `/specs/011-status-ordered-tasks/spec.md`

## Summary

Replace the flat `Vec<Task>` in `BoardState` with per-status `Vec<Task>` collections so that each status column maintains its own ordered list. When a task is added to a status (via creation, advance, or retreat), it is inserted at index 0 (top) of that status's list. The most recently added task always appears first in each column. Backward-compatible deserialization handles legacy flat-list JSON files.

## Technical Context

**Language/Version**: Rust stable >= 1.75 (via rustup)
**Primary Dependencies**: ratatui 0.29 (crossterm 0.28.1), serde + serde_json 1.0, chrono 0.4, directories 5, unicode-width
**Storage**: JSON file (`current.log` in platform data directory)
**Testing**: `cargo test` (built-in test framework)
**Target Platform**: macOS, Linux, Windows (terminal)
**Project Type**: Desktop TUI application (CLI)
**Performance Goals**: 60 fps rendering (<=16ms per frame)
**Constraints**: Single-file data format, backward-compatible with existing `current.log`
**Scale/Scope**: Single-user local app, typically <100 tasks

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Status | Notes |
|-----------|--------|-------|
| I. Test-First | PASS | TDD explicitly required by user and spec (FR-008). Tests written before implementation. |
| II. Simplicity / YAGNI | PASS | Per-status Vec is the minimum structure change needed to fulfill the requirement. No unnecessary abstractions. |
| III. TUI-First | PASS | No TUI changes required beyond using the new data source for column rendering. Keyboard-driven workflow unchanged. |
| IV. Data Portability | PASS | JSON format preserved. Legacy flat-list format supported via custom deserialization. Version field maintained. |
| V. Correctness Over Performance | PASS | No `unwrap()` in production paths. Explicit error handling for deserialization. |

## Project Structure

### Documentation (this feature)

```text
specs/011-status-ordered-tasks/
├── plan.md              # This file
├── research.md          # Phase 0 output
├── data-model.md        # Phase 1 output
├── quickstart.md        # Phase 1 output
└── tasks.md             # Phase 2 output (via /speckit.tasks)
```

### Source Code (repository root)

```text
src/
├── model.rs             # BoardState: Vec<Task> -> per-status storage + custom Serialize/Deserialize
├── app.rs               # AppState: tasks_for_column(), advance/retreat/reorder/create/delete
├── ui.rs                # render_column: use updated tasks_for_column()
├── storage.rs           # No structural changes (save/load BoardState as-is)
├── input.rs             # No changes
├── main.rs              # No changes
├── lib.rs               # No changes
└── url.rs               # No changes

tests/
├── storage_tests.rs     # Add backward-compatibility round-trip tests
├── model_tests.rs       # No changes expected
└── url_tests.rs         # No changes
```

**Structure Decision**: Single-project Rust layout. Changes concentrated in `model.rs` (data structure) and `app.rs` (logic). No new files needed.

## Complexity Tracking

No violations to justify. The per-status Vec is the direct, minimum-complexity implementation of the requirement.
