# Research: Panel Focus Keyboard Shortcuts

**Feature**: 012-panel-focus-keys
**Date**: 2026-03-26

---

## Decision 1: Key binding location

**Decision**: Add `Char('m')` and `Char('t')` match arms inside `handle_normal_key()` in `src/main.rs`.

**Rationale**: All keyboard shortcuts in normal (non-editing) mode are dispatched from `handle_normal_key()`. This function matches on `KeyCode` and delegates to state mutations in `AppState`. There is no separate keymap table — bindings live directly in match arms.

**Alternatives considered**: A separate keymap configuration struct was considered, but Constitution Principle II (YAGNI) forbids such premature abstraction for a two-key feature.

---

## Decision 2: State mutation for focus switch

**Decision**: Setting `app.focus_area` is sufficient to switch panel focus. No additional state changes are required.

**Rationale**: `FocusArea` (defined in `src/model.rs`) is the single source of truth for which panel is active. All rendering (`ui.rs`) and navigation (`app.rs`) already branch on `app.focus_area`. Setting it to `FocusArea::Memo` or `FocusArea::Kanban` immediately changes which panel responds to subsequent keypresses and which panel is highlighted.

**Alternatives considered**: Also resetting `focused_memo` or `focused_col` on switch — not needed since those fields are already preserved between focus changes.

---

## Decision 3: Guard condition (normal mode only)

**Decision**: The 'm' and 't' shortcuts are added to the top-level match in `handle_normal_key()`, which is only called when `app.mode == AppMode::Normal`. The existing dispatch in `main.rs` already excludes `InputTitle` and `InputDetail` modes from reaching `handle_normal_key()`.

**Rationale**: The event loop calls `handle_normal_key()` only when `app.mode` is `Normal`; input modes route to `handle_input_key()` instead. Therefore no additional guard is needed inside the new match arms.

**Alternatives considered**: Adding an explicit `if app.mode == AppMode::Normal` guard inside the arms — redundant given existing dispatch logic.

---

## Decision 4: No key conflicts

**Decision**: Neither `Char('m')` nor `Char('t')` is currently bound anywhere in the codebase. Safe to add without displacing any existing functionality.

**Rationale**: Grep across `src/main.rs` and `src/app.rs` confirms no existing bindings for these keys.

**Alternatives considered**: N/A — no conflicts exist.

---

## Decision 5: No-op when already focused

**Decision**: If the user presses 'm' while `focus_area == FocusArea::Memo` (or 't' while `focus_area == FocusArea::Kanban`), simply assign the same value. Rust assignment is a no-op semantically and requires no special branching.

**Rationale**: Simplest correct implementation. No visible state change occurs; the UI re-renders with the same focus highlight.

**Alternatives considered**: An explicit `if focus_area != target { ... }` guard — unnecessary complexity for a hot path that renders at 60 fps regardless.

---

## Decision 6: Test strategy

**Decision**: Write unit tests in `src/app.rs` (inline `#[cfg(test)]` module) that construct a minimal `AppState`, call the key handler, and assert `focus_area` changes as expected. No TUI rendering required.

**Rationale**: Constitution Principle I requires failing tests before implementation. The state mutation (`focus_area` assignment) is pure and testable without a terminal. Existing test patterns in the codebase use inline unit tests.

**Alternatives considered**: Integration tests in `tests/` — overkill for a pure state mutation; inline unit tests match existing project conventions.
