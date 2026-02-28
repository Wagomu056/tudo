# tudo Development Guidelines

Auto-generated from all feature plans. Last updated: 2026-02-27

## Active Technologies
- Rust stable ≥ 1.75 (via rustup) + ratatui 0.29 (crossterm), serde + serde_json 1.0, chrono 0.4; adding `directories = "5"` (002-xdg-log-dirs)
- Local files — `current.log` (JSON), `YYYYMMDD.log` (JSON Lines) (002-xdg-log-dirs)
- Rust stable ≥ 1.75 (via rustup) + ratatui 0.29 (crossterm backend), serde + serde_json 1.0, chrono 0.4 (003-keep-task-focus)
- JSON files (`current.log`, `YYYYMMDD.log`) — unchanged by this feature (003-keep-task-focus)

- Rust stable (≥ 1.75 via rustup) + ratatui 0.28 (crossterm backend), serde + serde_json 1.0, chrono 0.4 (001-kanban-tui)

## Project Structure

```text
src/
tests/
```

## Commands

cargo test [ONLY COMMANDS FOR ACTIVE TECHNOLOGIES][ONLY COMMANDS FOR ACTIVE TECHNOLOGIES] cargo clippy

## Code Style

Rust stable (≥ 1.75 via rustup): Follow standard conventions

## Recent Changes
- 003-keep-task-focus: Added Rust stable ≥ 1.75 (via rustup) + ratatui 0.29 (crossterm backend), serde + serde_json 1.0, chrono 0.4
- 002-xdg-log-dirs: Added Rust stable ≥ 1.75 (via rustup) + ratatui 0.29 (crossterm), serde + serde_json 1.0, chrono 0.4; adding `directories = "5"`

- 001-kanban-tui: Added Rust stable (≥ 1.75 via rustup) + ratatui 0.28 (crossterm backend), serde + serde_json 1.0, chrono 0.4

<!-- MANUAL ADDITIONS START -->
<!-- MANUAL ADDITIONS END -->
