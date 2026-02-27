# Developer Quickstart: 002-xdg-log-dirs

**Goal**: Verify that the platform data directory feature is working correctly in your local environment.

---

## Prerequisites

- Rust stable toolchain (≥ 1.75) installed via `rustup`
- `cargo` available on PATH

---

## Run Tests

```bash
# All unit + integration tests
cargo test

# Only storage-related tests
cargo test --test storage_tests

# Lint
cargo clippy -- -D warnings
```

---

## Verify Behavior Manually

1. **Build and run** from a scratch directory:
   ```bash
   mkdir /tmp/tudo-test-run
   cd /tmp/tudo-test-run
   cargo run --manifest-path /path/to/tudo/Cargo.toml
   ```

2. **Interact**: Create a task (press `a`), add a title, move it to Done (press `Enter` repeatedly).

3. **Quit** (press `q`).

4. **Verify no log files** in `/tmp/tudo-test-run`:
   ```bash
   ls /tmp/tudo-test-run   # should show nothing tudo-related
   ```

5. **Verify files in platform data dir**:
   ```bash
   # macOS
   ls ~/Library/Application\ Support/tudo/

   # Linux
   ls ~/.local/share/tudo/
   ```
   You should see `current.log` and a `YYYYMMDD.log` file.

---

## What the Tests Cover

| Test | Description |
|------|-------------|
| `resolve_data_dir_returns_a_path` | `resolve_data_dir()` returns `Ok(PathBuf)` in a normal environment |
| `resolve_data_dir_creates_directory` | The returned directory exists on disk after calling `resolve_data_dir()` |
| `resolve_data_dir_fallback` | When `HOME` env var is overridden to a temp dir, the fallback path resolves correctly |
| `load_board_uses_data_dir` | `load_board()` reads from the data directory, not the CWD |
| `save_board_uses_data_dir` | `save_board()` writes to the data directory, not the CWD |
| `append_done_entry_uses_data_dir` | `append_done_entry()` writes to the data directory, not the CWD |

---

## Files Changed

| File | Change |
|------|--------|
| `Cargo.toml` | Add `directories = "5"` dependency |
| `src/storage.rs` | Add `resolve_data_dir()`; update `load_board()`, `save_board()`, `append_done_entry()` |
| `tests/storage_tests.rs` | Add tests for `resolve_data_dir()` and dir-aware convenience functions |
