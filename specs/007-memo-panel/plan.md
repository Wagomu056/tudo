# Implementation Plan: Memo Panel

**Branch**: `007-memo-panel` | **Date**: 2026-03-01 | **Spec**: [spec.md](spec.md)
**Input**: Feature specification from `/specs/007-memo-panel/spec.md`

## Summary

Add a memo panel zone below the 4 kanban columns (not under the Detail panel) occupying ~20% of screen height. Memo items (title + detail, no status) are displayed left-to-right in a fixed-width tile grid. Navigation to/from the panel uses the existing hjkl keys as a natural extension. Memos are persisted in `current.log` alongside tasks. Four existing source files are modified; no new files or crate dependencies are required.

## Technical Context

**Language/Version**: Rust stable ≥ 1.75 (via rustup)
**Primary Dependencies**: ratatui 0.29 (crossterm 0.28.1 backend), serde + serde_json 1.0, chrono 0.4, directories 5
**Storage**: JSON file — `current.log` in platform data directory (XDG/AppSupport/LOCALAPPDATA)
**Testing**: Rust built-in test framework (`cargo test`); inline `#[cfg(test)]` modules in `app.rs`; integration tests in `tests/`
**Target Platform**: macOS, Linux, Windows terminal emulators (80-column minimum)
**Project Type**: TUI CLI application
**Performance Goals**: ≤ 16 ms per frame (60 fps) — memo panel adds one `Layout` split and one widget render per frame; well within budget
**Constraints**: No new crate dependencies; backward-compatible `current.log` schema; no `unwrap()`/`expect()` in production paths
**Scale/Scope**: Single-user; memo count expected in single digits to low dozens; no pagination required (no-scroll design per spec)

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Status | Evidence |
|-----------|--------|----------|
| I. Test-First | ✅ PASS | Implementation order in quickstart.md mandates failing tests before each implementation step |
| II. Simplicity / YAGNI | ✅ PASS | No new files, no new dependencies, no new abstractions beyond what the feature requires; `is_memo` flag reuses existing `InputState`/`AppMode` |
| III. TUI-First | ✅ PASS | Fully keyboard-driven; memo panel renders as ratatui widget; no blocking I/O introduced |
| IV. Data Portability | ✅ PASS | `#[serde(default)]` on new `BoardState` fields ensures backward compatibility within version 1; JSON remains human-readable |
| V. Correctness Over Performance | ✅ PASS | All `Option` returns from memo access are handled with `if let`/`match`; no `unwrap()` introduced |

**Gate result**: All five principles satisfied. Proceeding to Phase 1 design.

## Project Structure

### Documentation (this feature)

```text
specs/007-memo-panel/
├── plan.md              # This file (/speckit.plan command output)
├── research.md          # Phase 0 output — design decisions
├── data-model.md        # Phase 1 output — entities and state
├── quickstart.md        # Phase 1 output — developer guide
├── contracts/
│   └── keybindings.md  # Phase 1 output — key binding contract
└── tasks.md             # Phase 2 output (/speckit.tasks command)
```

### Source Code (repository root)

```text
src/
├── model.rs     ← Add Memo struct, FocusArea enum; extend InputState, BoardState
├── app.rs       ← Extend AppState; add memo navigation + CRUD methods
├── ui.rs        ← Add memo panel layout + render_memo_panel; update detail + status bar
├── main.rs      ← Route hjkl/a/e/E/D to memo handlers based on focus_area
├── storage.rs   ← No changes (BoardState serialization handles it automatically)
├── input.rs     ← No changes
└── url.rs       ← No changes

tests/
├── model_tests.rs    ← Add Memo serialization + backward-compat tests
├── storage_tests.rs  ← Add round-trip test with memos
└── url_tests.rs      ← No changes
```

**Structure Decision**: Single-project flat layout unchanged. All memo-related logic is co-located with the corresponding existing modules (model → model.rs, app logic → app.rs, rendering → ui.rs, input routing → main.rs).

## Phase 0: Research — Completed

See [research.md](research.md) for full decision records. Summary:

| Decision | Choice |
|----------|--------|
| Layout | Two-step split: kanban_area → [columns_area, memo_area] vertical; detail unchanged |
| Items-per-row | `MEMO_ITEM_WIDTH = 24`; `memo_cols = panel_width / 24` cached in `AppState.memo_cols` |
| Cross-boundary nav | `j` at kanban bottom enters memo; `k` at memo top row returns to kanban |
| Storage | `#[serde(default)]` on new fields; version 1 unchanged |
| Input mode | Reuse `InputTitle`/`InputDetail` with `InputState.is_memo: bool` flag |
| New dependencies | None |

## Phase 1: Design — Completed

### Data Model

See [data-model.md](data-model.md) for full entity specifications.

**New types**:
- `Memo { id: u64, title: String, detail: String }` — persisted in `board.memos`
- `FocusArea { Kanban, Memo }` — runtime-only, not persisted

**Extended types**:
- `InputState` + `is_memo: bool`
- `BoardState` + `memos: Vec<Memo>` + `next_memo_id: u64` (both `#[serde(default)]`)
- `AppState` + `focus_area: FocusArea` + `focused_memo: usize` + `memo_cols: usize`

### Key Binding Contract

See [contracts/keybindings.md](contracts/keybindings.md) for the full contract.

**Critical new bindings (Memo focus)**:

| Key | Action |
|-----|--------|
| `j` (Kanban, bottom) | Enter Memo focus |
| `k` (Memo, row 0) | Return to Kanban focus |
| `h/l` | Move left/right within memo row |
| `j/k` | Move down/up between memo rows |
| `a` | Create new memo |
| `e/E` | Edit memo title / detail |
| `D` | Delete focused memo |

### Developer Guide

See [quickstart.md](quickstart.md) for step-by-step implementation guide with test-first ordering.

## Complexity Tracking

No constitution violations. No complexity justification required.

## Post-Design Constitution Re-check

| Principle | Status | Notes |
|-----------|--------|-------|
| I. Test-First | ✅ PASS | quickstart.md orders tests before every implementation step |
| II. Simplicity / YAGNI | ✅ PASS | 4 files modified, 0 new files, 0 new dependencies, 3 new fields in AppState |
| III. TUI-First | ✅ PASS | One additional `Layout` split + one new `Paragraph`/widget render per frame |
| IV. Data Portability | ✅ PASS | `#[serde(default)]` confirmed; version 1 retained |
| V. Correctness Over Performance | ✅ PASS | All memo access via `Option`-returning helpers; no forced unwraps |

**All gates pass. Ready for `/speckit.tasks`.**
