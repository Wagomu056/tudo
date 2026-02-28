# Implementation Plan: Task Reordering Within Columns

**Branch**: `006-task-reorder` | **Date**: 2026-02-28 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `specs/006-task-reorder/spec.md`

## Summary

Add J (Shift+J) and K (Shift+K) keybindings to move the focused task one
position down or up within its column. Task order is encoded as the element
order in the existing `BoardState.tasks: Vec<Task>` — no schema changes
required. Saves after reorder are debounced to 1 second using `std::time::Instant`
in the event loop; other save events and quit flush the pending save immediately.

## Technical Context

**Language/Version**: Rust stable ≥ 1.75 (via rustup)
**Primary Dependencies**: ratatui 0.29 (crossterm 0.28.1), serde_json 1.0, chrono 0.4, directories 5
**Storage**: JSON file (`current.log` in platform data directory)
**Testing**: `cargo test` (unit tests inline in `app.rs`, integration tests in `tests/`)
**Target Platform**: macOS, Linux, Windows terminal emulators
**Project Type**: CLI / TUI application
**Performance Goals**: ≤ 16 ms per frame (60 fps); reorder visual response instant (same frame)
**Constraints**: No UI thread blocking; no new crate dependencies; save I/O debounced to ≤ 1 per second during rapid reordering
**Scale/Scope**: Single-user local app; Vec size bounded by practical task count (< 1000)

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Status | Notes |
|-----------|--------|-------|
| I. Test-First | ✅ PASS | Tests written in `app.rs` before implementation; inline tests cover all new methods (reorder_task_down, reorder_task_up, boundary no-ops, focus tracking) |
| II. Simplicity / YAGNI | ✅ PASS | No new crates, no new abstractions; debounce uses two local variables in `run_app`; reorder uses `Vec::swap` from std; three similar test cases are acceptable without abstraction |
| III. TUI-First | ✅ PASS | J/K keybindings are keyboard-driven; debounce is checked in the existing 200 ms poll-timeout path, never blocks the UI thread |
| IV. Data Portability | ✅ PASS | No schema change: `Vec<Task>` serialization order already encodes column order in JSON; `version` field unchanged; human-readable before and after |
| V. Correctness | ✅ PASS | All new paths use `?`/`match`; no `unwrap()` in production code; no `unsafe`; explicit no-op on boundary (return without mutation) |

**Gate result: ALL PASS — proceed to Phase 0.**

## Project Structure

### Documentation (this feature)

```text
specs/006-task-reorder/
├── plan.md              ← this file
├── research.md          ← Phase 0 output
├── data-model.md        ← Phase 1 output
├── quickstart.md        ← Phase 1 output
├── contracts/
│   └── keybindings.md   ← Phase 1 output
└── tasks.md             ← Phase 2 output (/speckit.tasks)
```

### Source Code (affected files only)

```text
src/
├── app.rs       ← add reorder_task_down(), reorder_task_up() + inline tests
└── main.rs      ← add J/K key dispatch + debounce state in run_app()

tests/
└── (no new integration test file needed — pure unit tests suffice)
```

**Structure Decision**: Single-project layout (existing). All changes are
confined to two files. No new modules, no new files.

## Implementation Phases

### Phase 0 — Research (see research.md)

Unknowns to resolve:
1. Uppercase key detection in crossterm (`KeyCode::Char('J')` vs modifier flags)
2. `Vec::swap` semantics for in-place reorder
3. Debounce pattern in a synchronous 200 ms poll loop without threads

### Phase 1 — Design & Contracts (see data-model.md, contracts/, quickstart.md)

1. Task ordering data model (Vec order = column order, no schema change)
2. Keybinding contract document
3. Agent context update
