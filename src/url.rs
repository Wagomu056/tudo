use crate::model::UrlHitRegion;

// ── Tests (simulate_wrap is crate-private, tested here) ──────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // T016: simulate_wrap tests

    #[test]
    fn simulate_wrap_short_line_no_wrap() {
        // "hello world" at width 20 → all on row 0
        let result = simulate_wrap("hello world", 20);
        let rows: Vec<u16> = result.iter().map(|&(_, _, r)| r).collect();
        assert!(rows.iter().all(|&r| r == 0), "all chars on row 0");
    }

    #[test]
    fn simulate_wrap_line_exactly_at_width() {
        // "hello" (5 chars) at width 5 → fits, no wrap
        let result = simulate_wrap("hello", 5);
        let rows: Vec<u16> = result.iter().map(|&(_, _, r)| r).collect();
        assert!(rows.iter().all(|&r| r == 0));
    }

    #[test]
    fn simulate_wrap_wraps_at_word_boundary() {
        // "hello world" at width 8 → "hello" on row 0, "world" on row 1
        let result = simulate_wrap("hello world", 8);
        // "hello" = 5 chars → col 0-4, row 0
        // " world" → "world" wraps to row 1
        let hello_rows: Vec<u16> = result
            .iter()
            .filter(|&&(b, _, _)| b < 5)
            .map(|&(_, _, r)| r)
            .collect();
        let world_rows: Vec<u16> = result
            .iter()
            .filter(|&&(b, _, _)| b > 5)
            .map(|&(_, _, r)| r)
            .collect();
        assert!(hello_rows.iter().all(|&r| r == 0), "hello on row 0");
        assert!(world_rows.iter().all(|&r| r == 1), "world on row 1");
    }

    #[test]
    fn simulate_wrap_multi_word_wraps_twice() {
        // "aa bb cc dd" at width 5 → "aa bb" row 0, "cc dd" row 1
        let result = simulate_wrap("aa bb cc dd", 5);
        // "aa" at col 0-1 row 0
        let aa_rows: Vec<u16> = result
            .iter()
            .filter(|&&(b, _, _)| b < 2)
            .map(|&(_, _, r)| r)
            .collect();
        // "bb" bytes 3-4
        let bb_rows: Vec<u16> = result
            .iter()
            .filter(|&&(b, _, _)| b >= 3 && b < 5)
            .map(|&(_, _, r)| r)
            .collect();
        // "cc" bytes 6-7
        let cc_rows: Vec<u16> = result
            .iter()
            .filter(|&&(b, _, _)| b >= 6 && b < 8)
            .map(|&(_, _, r)| r)
            .collect();
        // "dd" bytes 9-10
        let dd_rows: Vec<u16> = result
            .iter()
            .filter(|&&(b, _, _)| b >= 9)
            .map(|&(_, _, r)| r)
            .collect();

        assert!(aa_rows.iter().all(|&r| r == 0), "aa on row 0");
        assert!(bb_rows.iter().all(|&r| r == 0), "bb on row 0");
        assert!(cc_rows.iter().all(|&r| r == 1), "cc on row 1");
        assert!(dd_rows.iter().all(|&r| r == 1), "dd on row 1");
    }
}

// ── URL extraction ────────────────────────────────────────────────────────────

/// Extract all `http://` and `https://` URL byte-offset spans from `text`.
/// Returns `(start, end)` pairs where `&text[start..end]` is the URL.
pub fn extract_url_spans(text: &str) -> Vec<(usize, usize)> {
    let mut results = Vec::new();
    let mut search_from = 0;

    while search_from < text.len() {
        let rest = &text[search_from..];

        let http_pos = rest.find("http://");
        let https_pos = rest.find("https://");

        let scheme_start_rel = match (http_pos, https_pos) {
            (None, None) => break,
            (Some(h), None) => h,
            (None, Some(s)) => s,
            (Some(h), Some(s)) => {
                // If https starts at the same offset as http ("https://"),
                // prefer https (longer scheme).
                if s <= h {
                    s
                } else {
                    h
                }
            }
        };

        let abs_start = search_from + scheme_start_rel;
        let after_scheme = if text[abs_start..].starts_with("https://") {
            abs_start + "https://".len()
        } else {
            abs_start + "http://".len()
        };

        if after_scheme >= text.len() {
            search_from = abs_start + 1;
            continue;
        }

        // Scan forward from after_scheme, tracking bracket depth.
        let mut last_safe_end = after_scheme;
        let mut depth: usize = 0;
        let mut final_end = after_scheme;

        for (byte_offset, ch) in text[after_scheme..].char_indices() {
            let pos = after_scheme + byte_offset;
            let next_pos = pos + ch.len_utf8();

            if ch.is_whitespace() || ch.is_control() {
                final_end = last_safe_end;
                break;
            } else if ch == '(' || ch == '[' {
                depth += 1;
                last_safe_end = next_pos;
                final_end = next_pos;
            } else if ch == ')' || ch == ']' {
                if depth > 0 {
                    depth -= 1;
                    last_safe_end = next_pos;
                    final_end = next_pos;
                } else {
                    final_end = last_safe_end;
                    break;
                }
            } else if ch == '.' || ch == ',' || ch == ';' || ch == '!' || ch == '?' {
                let next_ch = text[next_pos..].chars().next();
                match next_ch {
                    None | Some(' ') | Some('\t') | Some('\n') | Some('\r') => {
                        final_end = last_safe_end;
                        break;
                    }
                    Some(nc) if nc.is_whitespace() || nc.is_control() => {
                        final_end = last_safe_end;
                        break;
                    }
                    _ => {
                        last_safe_end = next_pos;
                        final_end = next_pos;
                    }
                }
            } else {
                last_safe_end = next_pos;
                final_end = next_pos;
            }
        }

        // If we fell off the end of the string without breaking, use last_safe_end.
        if final_end == after_scheme {
            final_end = last_safe_end;
        }

        if final_end > abs_start {
            results.push((abs_start, final_end));
        }

        search_from = if final_end > abs_start {
            final_end
        } else {
            abs_start + 1
        };
    }

    results
}

// ── URL opening ───────────────────────────────────────────────────────────────

/// Open `url` in the system default browser using a non-blocking `spawn()`.
pub fn open_url(url: &str) -> std::io::Result<()> {
    #[cfg(target_os = "macos")]
    std::process::Command::new("open").arg(url).spawn()?;
    #[cfg(target_os = "linux")]
    std::process::Command::new("xdg-open").arg(url).spawn()?;
    #[cfg(target_os = "windows")]
    std::process::Command::new("cmd")
        .args(["/C", "start", "", url])
        .spawn()?;
    Ok(())
}

// ── List item URL regions ──────────────────────────────────────────────────────

/// Compute `UrlHitRegion` entries for a single-line task title in the kanban list.
/// `item_row` is the absolute terminal row; `text_x` is the leftmost text column.
pub fn list_item_url_regions(title: &str, item_row: u16, text_x: u16) -> Vec<UrlHitRegion> {
    extract_url_spans(title)
        .into_iter()
        .map(|(start, end)| {
            let col_start = text_x + title[..start].chars().count() as u16;
            let col_end = text_x + title[..end].chars().count() as u16;
            UrlHitRegion {
                row: item_row,
                col_start,
                col_end,
                url: title[start..end].to_string(),
            }
        })
        .collect()
}

// ── Word-wrap simulation ──────────────────────────────────────────────────────

/// Simulate ratatui's word-wrap layout for `text` at `width` columns.
/// Returns `(byte_offset, display_col, display_row)` for each character position.
pub(crate) fn simulate_wrap(text: &str, width: u16) -> Vec<(usize, u16, u16)> {
    let width = width as usize;
    let mut result = Vec::new();

    // Collect (byte_start, leading_spaces, word_str) entries.
    let mut entries: Vec<(usize, usize, &str)> = Vec::new();
    let mut pos = 0;
    while pos < text.len() {
        let space_start = pos;
        let mut spaces = 0;
        while pos < text.len() && text.as_bytes()[pos] == b' ' {
            pos += 1;
            spaces += 1;
        }
        if pos >= text.len() {
            if spaces > 0 {
                entries.push((space_start, spaces, ""));
            }
            break;
        }
        let word_start = pos;
        while pos < text.len() && text.as_bytes()[pos] != b' ' {
            let ch = text[pos..].chars().next().unwrap();
            pos += ch.len_utf8();
        }
        entries.push((space_start, spaces, &text[word_start..pos]));
    }

    let mut cur_col: usize = 0;
    let mut cur_row: usize = 0;

    for (byte_start, leading_spaces, word) in &entries {
        let word_char_len = word.chars().count();

        if cur_col > 0 && cur_col + leading_spaces + word_char_len > width {
            // Wrap: move to next line, drop leading spaces.
            cur_col = 0;
            cur_row += 1;
            let mut bpos = byte_start + leading_spaces;
            for ch in word.chars() {
                result.push((bpos, cur_col as u16, cur_row as u16));
                bpos += ch.len_utf8();
                cur_col += 1;
            }
        } else {
            // Emit leading spaces.
            let mut bpos = *byte_start;
            for _ in 0..*leading_spaces {
                result.push((bpos, cur_col as u16, cur_row as u16));
                bpos += 1;
                cur_col += 1;
            }
            // Emit word characters.
            for ch in word.chars() {
                result.push((bpos, cur_col as u16, cur_row as u16));
                bpos += ch.len_utf8();
                cur_col += 1;
            }
        }
    }

    result
}

// ── Detail panel URL regions ──────────────────────────────────────────────────

/// Compute `UrlHitRegion` entries for text in the detail panel with word-wrap.
/// `base_row` and `base_col` are the top-left of the text area (inside border).
pub fn detail_url_regions(
    text: &str,
    available_width: u16,
    base_row: u16,
    base_col: u16,
) -> Vec<UrlHitRegion> {
    let char_map = simulate_wrap(text, available_width);
    let url_spans = extract_url_spans(text);
    let mut regions = Vec::new();

    for (url_start, url_end) in url_spans {
        let url_str = text[url_start..url_end].to_string();
        let mut row_groups: std::collections::BTreeMap<u16, (u16, u16)> =
            std::collections::BTreeMap::new();

        for &(byte_off, dcol, drow) in &char_map {
            if byte_off >= url_start && byte_off < url_end {
                let abs_row = base_row + drow;
                let abs_col = base_col + dcol;
                let entry = row_groups.entry(abs_row).or_insert((abs_col, abs_col + 1));
                if abs_col < entry.0 {
                    entry.0 = abs_col;
                }
                if abs_col + 1 > entry.1 {
                    entry.1 = abs_col + 1;
                }
            }
        }

        for (row, (col_start, col_end)) in row_groups {
            regions.push(UrlHitRegion {
                row,
                col_start,
                col_end,
                url: url_str.clone(),
            });
        }
    }

    regions
}
