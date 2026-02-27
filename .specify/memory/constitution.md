<!--
SYNC IMPACT REPORT
==================
Version change: (none) → 1.0.0
This is the initial ratification of the Tudo Constitution.

Modified principles: N/A (initial creation)

Added sections:
  - Core Principles (I–V): Test-First, Simplicity/YAGNI, TUI-First,
    Data Portability, Correctness Over Performance
  - Technology Stack
  - Development Workflow
  - Governance

Templates reviewed:
  - .specify/templates/plan-template.md  ✅ compatible
    (Constitution Check section is generic — works as-is)
  - .specify/templates/spec-template.md  ✅ compatible
    (no constitution-specific mandatory sections to add)
  - .specify/templates/tasks-template.md ✅ compatible
    (test-first task ordering matches Principle I)
  - .specify/templates/checklist-template.md ✅ compatible
    (generic structure, no updates needed)

Follow-up TODOs: none — all placeholders resolved.
-->

# Tudo Constitution

## Core Principles

### I. Test-First (NON-NEGOTIABLE)

Tests MUST be written before any implementation code. The Red-Green-Refactor
cycle is strictly enforced:

- Write a failing test that describes the desired behavior.
- Get user/reviewer approval on the test intent.
- Confirm the test fails (red).
- Implement the minimal code to make the test pass (green).
- Refactor while keeping tests green.

No feature or bug fix may be merged without accompanying tests that were
written before the implementation. This is a hard gate enforced at code review,
not a guideline.

### II. Simplicity / YAGNI

Every line of code MUST justify its existence against a current, concrete
requirement.

- Premature abstractions, unused helpers, and speculative "future-proofing"
  are forbidden.
- Three similar code blocks are preferable to a premature abstraction.
- Complexity MUST be justified in the `Complexity Tracking` table of the
  implementation plan before work begins.
- Prefer the standard library and existing crate dependencies before adding
  new ones.

### III. TUI-First

The primary user interface is a terminal UI powered by ratatui. The application
MUST:

- Be fully keyboard-driven; mouse support is optional and never required for
  core workflows.
- Render correctly in standard terminal emulators at 80 columns minimum width.
- Deliver responsive feedback for all user actions (target ≤ 16 ms per frame
  for 60 fps rendering).
- Never block the UI thread with I/O or long-running operations; offload to
  async tasks or threads.

A headless (non-TUI) mode MAY be provided for scripting, but the TUI is the
reference interface and MUST remain the design priority.

### IV. Data Portability

Task data MUST be stored in a human-readable, text-based format (e.g., TOML,
JSON, or plain-text Markdown). Binary primary storage formats are forbidden.

- Users MUST be able to read, edit, and back up task data without the
  application running.
- Schema changes MUST be backward-compatible within a MAJOR version.
- Data files MUST include a `version` field to enable forward migration tooling.

### V. Correctness Over Performance

Rust's type system and ownership model are leveraged to eliminate runtime
errors by construction.

- `unwrap()` and `expect()` on `Option`/`Result` are forbidden in production
  code paths; use explicit error propagation (`?` operator or `match`).
- `panic!` is reserved for unrecoverable programmer-error invariant violations
  only.
- `unsafe` blocks require an inline `// SAFETY:` comment and MUST be reviewed
  by a second developer before merge.
- Performance optimization is secondary to correctness; profile with real data
  before optimizing.

## Technology Stack

- **Language**: Rust (latest stable toolchain via `rustup`)
- **TUI Framework**: ratatui
- **Build Tool**: Cargo
- **Serialization**: serde (with an appropriate format feature: `serde_json`,
  `toml`, etc.)
- **Testing**: Rust built-in test framework (`cargo test`); integration tests
  in `tests/`
- **Linting**: `clippy` (deny warnings in CI); `rustfmt` for formatting
- **CI gates**: Every PR MUST pass `cargo test`, `cargo clippy -- -D warnings`,
  and `cargo fmt --check` before merge.

New crate dependencies MUST be evaluated for maintenance status,
compile-time impact, and license compatibility (MIT or Apache-2.0 preferred).

## Development Workflow

- All features MUST originate from a spec in `specs/[###-feature-name]/spec.md`.
- Implementation plans (`plan.md`) MUST include a Constitution Check section
  verifying adherence to all five Core Principles before Phase 0 research
  begins.
- PRs are blocked if the Constitution Check has unresolved violations.
- Code review MUST verify: test-first evidence (failing tests visible in
  earlier commits), no `unwrap()` in production paths, and ratatui frame
  budget not exceeded.
- Any developer may raise a Constitution violation; flagged items MUST be
  resolved before the PR merges.

## Governance

This Constitution supersedes all other development guidelines and practices.
Amendments require:

1. A written proposal describing the change and its rationale.
2. Agreement from all active contributors.
3. A migration plan for existing code if the amendment is backward-incompatible
   with current practices.
4. A version increment following the policy below.

**Versioning policy**:

- MAJOR: Principle removals, redefinitions that break existing practices, or
  backward-incompatible governance changes.
- MINOR: New principle or section added, or material expansion of existing
  guidance.
- PATCH: Clarifications, wording improvements, or typo fixes.

**Version**: 1.0.0 | **Ratified**: 2026-02-27 | **Last Amended**: 2026-02-27
