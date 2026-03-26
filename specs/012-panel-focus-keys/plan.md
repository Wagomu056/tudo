# Implementation Plan: Panel Focus Keyboard Shortcuts

**Branch**: `012-panel-focus-keys` | **Date**: 2026-03-26 | **Spec**: [spec.md](spec.md)
**Input**: Feature specification from `/specs/012-panel-focus-keys/spec.md`

## Summary

Add `m` and `t` keyboard shortcuts in Normal mode to directly focus the memo panel and kanban (todo) panel respectively. Implementation requires two new match arms in `handle_normal_key()` in `src/main.rs` and four unit tests in `src/app.rs`. No new dependencies, no data model changes, no storage changes.

## Technical Context

**Language/Version**: Rust stable ≥ 1.75 (via rustup)
**Primary Dependencies**: ratatui 0.29 (crossterm 0.28.1 backend), serde + serde_json 1.0, chrono 0.4, directories 5, unicode-width
**Storage**: N/A — focus state is transient, not persisted
**Testing**: Rust built-in test framework (`cargo test`); inline `#[cfg(test)]` unit tests in `src/app.rs`
**Target Platform**: macOS / Linux terminal (crossterm backend)
**Project Type**: TUI desktop application
**Performance Goals**: ≤ 16 ms per frame (60 fps) — key handlers are O(1) assignments, no impact
**Constraints**: Normal-mode only; must not intercept 'm'/'t' during text input
**Scale/Scope**: 2 new match arms, 4 new unit tests; no architectural changes

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Status | Notes |
|-----------|--------|-------|
| I. Test-First | PASS | Unit tests written before implementation code (see quickstart.md Step 1) |
| II. Simplicity / YAGNI | PASS | Two match arms; no new abstractions, no new crate dependencies |
| III. TUI-First | PASS | Keyboard-driven navigation; O(1) state assignment does not approach 16 ms budget |
| IV. Data Portability | PASS | No storage changes; `FocusArea` is transient UI state |
| V. Correctness Over Performance | PASS | No `unwrap()`/`expect()` needed; pure state assignment |

All five principles satisfied. No violations to justify.

## Project Structure

### Documentation (this feature)

```text
specs/012-panel-focus-keys/
├── plan.md           # This file
├── research.md       # Phase 0 output
├── data-model.md     # Phase 1 output
├── quickstart.md     # Phase 1 output
└── tasks.md          # Phase 2 output (/speckit.tasks — not yet created)
```

### Source Code (changes limited to)

```text
src/
├── main.rs     # Add 'm' and 't' arms in handle_normal_key()
└── app.rs      # Add 4 unit tests in #[cfg(test)] module
```

No other files require modification.

**Structure Decision**: Single-project layout unchanged. Feature touches only two existing files; no new modules or files are created.

## Implementation Steps

### Phase 0 (Complete)

- [x] research.md — all decisions resolved, no NEEDS CLARIFICATION remaining

### Phase 1 (Complete)

- [x] data-model.md — documents `FocusArea` state transitions; confirms no new fields or storage
- [x] quickstart.md — step-by-step implementation guide with test-first workflow
- [x] No contracts/ needed — this is a purely internal TUI state change with no external interface

### Phase 2 (Next: `/speckit.tasks`)

Tasks to generate:
1. Write 4 failing unit tests for 'm'/'t' key focus transitions
2. Implement `Char('t')` arm in `handle_normal_key()` (make `test_t_*` tests green)
3. Implement `Char('m')` arm in `handle_normal_key()` (make `test_m_*` tests green)
4. Run `cargo clippy -- -D warnings` and `cargo fmt` — fix any issues
5. Manual smoke test in terminal

## Risks & Mitigations

| Risk | Likelihood | Mitigation |
|------|-----------|------------|
| 'm' or 't' added elsewhere before merge | Low | Grep check during PR review |
| Input mode not guarding these keys | Low | Existing dispatch already routes input modes to `handle_input_key()`; covered by smoke test |
