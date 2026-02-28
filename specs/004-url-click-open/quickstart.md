# Quickstart: Clickable URLs in TUI

**Feature**: 004-url-click-open

---

## What This Feature Does

Any `http://` or `https://` URL visible in a task title (kanban column list) or task detail panel can be opened in the system default browser by left-clicking it with the mouse.

The terminal emulator must support mouse event reporting (most modern emulators do — iTerm2, Alacritty, WezTerm, GNOME Terminal, etc.). If mouse reporting is unsupported, the feature is silently unavailable; keyboard-only workflows are unaffected.

---

## Using the Feature

1. Run tudo as usual.
2. Navigate to any task that contains a URL in its title or detail.
3. Left-click the URL text.
4. The default browser opens the page.

No configuration needed. No keyboard shortcut required.

---

## Developer Guide

### New Module: `src/url.rs`

Contains two public functions:

```rust
/// Extract all http/https URL byte-offset spans from a string.
/// Returns (start, end) pairs suitable for &text[start..end].
pub fn extract_url_spans(text: &str) -> Vec<(usize, usize)>

/// Open a URL in the system default browser.
/// Uses spawn() so the UI thread is not blocked.
pub fn open_url(url: &str) -> std::io::Result<()>
```

### Adding URL Click Support to a New Widget

If you add a new widget that renders text containing URLs, follow these steps to make them clickable:

1. After computing the widget's layout `Rect`, call `extract_url_spans` on the text you will render.
2. For each `(start, end)` span, convert byte offsets to display column positions (character count from `text[..start].chars().count()`).
3. Compute the absolute screen row(s) the text occupies.
4. Push a `UrlHitRegion { row, col_start, col_end, url }` into `app.clickable_urls`.

### Testing Locally

Run unit tests:
```
cargo test --test url_tests
```

Run all tests:
```
cargo test
```

Run clippy:
```
cargo clippy -- -D warnings
```

### Verifying the Feature

1. Add a task with a URL in the title: `a` → type `See https://example.com` → Enter.
2. Start tudo.
3. Left-click the URL text in the kanban column.
4. Confirm browser opens `https://example.com`.

---

## Platform Notes

| OS | Browser Launch Command |
|----|------------------------|
| macOS | `open <url>` |
| Linux | `xdg-open <url>` |
| Windows | `cmd /C start "" <url>` |

If the launch command is unavailable (e.g., no browser installed), `open_url` returns an `Err`. The error is displayed briefly in the TUI status bar and then cleared.

---

## Known Limitations

- Only `http://` and `https://` schemes are recognized. `mailto:`, `ftp://`, etc. are not clickable.
- URLs that wrap across multiple terminal lines in the detail panel are fully supported, but each display segment is a separate hit region; clicking any segment opens the full URL.
- Internationalized domain names (IDN) may not be recognized if the URL text contains non-ASCII characters before `://`.
