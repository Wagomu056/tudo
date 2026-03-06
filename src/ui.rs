use ratatui::{
    layout::{Constraint, Direction, Layout, Position, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph},
    Frame,
};

use unicode_width::UnicodeWidthChar;

use crate::app::AppState;
use crate::model::{AppMode, FocusArea, Status, TaskHitRegion, MemoHitRegion, ALL_STATUSES};
use crate::url;

// ── Top-level render ─────────────────────────────────────────────────────────

pub fn render(frame: &mut Frame, app: &mut AppState) {
    let area = frame.area();

    // Guard: show warning if terminal is too small (memo panel needs extra height)
    if area.width < 40 || area.height < 12 {
        let warning = Paragraph::new("Terminal too small.\nMinimum: 40×12")
            .block(Block::default().borders(Borders::ALL))
            .style(Style::default().fg(Color::Yellow));
        frame.render_widget(warning, area);
        return;
    }

    // Vertical split: board rows + 1-line status bar
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(3), Constraint::Length(1)])
        .split(area);

    let board_area = rows[0];
    let status_area = rows[1];

    // Horizontal split: 75% kanban columns, 25% detail panel
    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(75), Constraint::Percentage(25)])
        .split(board_area);

    let kanban_area = cols[0];
    let detail_area = cols[1];

    // Vertical split of kanban_area: columns (80%) + memo panel (20%)
    let kanban_rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(80), Constraint::Percentage(20)])
        .split(kanban_area);

    let columns_area = kanban_rows[0];
    let memo_area = kanban_rows[1];

    // Equal 4-column split
    let columns = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Ratio(1, 4),
            Constraint::Ratio(1, 4),
            Constraint::Ratio(1, 4),
            Constraint::Ratio(1, 4),
        ])
        .split(columns_area);

    for (i, &status) in ALL_STATUSES.iter().enumerate() {
        render_column(
            frame,
            columns[i],
            app,
            status,
            i == app.focused_col && app.focus_area == FocusArea::Kanban,
        );
    }

    render_memo_panel(frame, memo_area, app);
    render_detail_panel(frame, detail_area, app);

    render_status_bar(frame, status_area, app);

    // Input popup overlay (drawn last so it appears on top)
    if app.mode != AppMode::Normal {
        render_input_popup(frame, area, app);
    }
}

// ── Column ────────────────────────────────────────────────────────────────────

fn render_column(
    frame: &mut Frame,
    area: Rect,
    app: &mut AppState,
    status: Status,
    is_focused_col: bool,
) {
    let focused_idx = app.focused_card[status.col_index()];

    // Compute URL and task hit regions.
    // Text starts at area.x + 1 (left border).
    let max_title_chars = area.width.saturating_sub(4) as usize;
    let text_x = area.x + 1;
    let col_index = status.col_index();
    let mut cumulative_rows: u16 = 0;
    let mut url_regions = Vec::new();
    let mut task_regions = Vec::new();
    {
        let tasks = app.tasks_for_column(status);
        for (vi, task) in tasks.iter().enumerate() {
            let wrapped = wrap_str(&task.title, max_title_chars);
            let item_row = area.y + 1 + cumulative_rows;
            let wrapped_len = wrapped.len() as u16;
            for (line_idx, line) in wrapped.iter().enumerate() {
                let row = item_row + line_idx as u16;
                let regions = url::list_item_url_regions(line, row, text_x);
                url_regions.extend(regions);
            }
            task_regions.push(TaskHitRegion {
                row_start: item_row,
                row_end: item_row + wrapped_len,
                col_start: area.x,
                col_end: area.x + area.width,
                column: col_index,
                card_index: vi,
            });
            cumulative_rows += wrapped_len;
            cumulative_rows += 1;
        }
    }
    app.clickable_urls.extend(url_regions);
    app.clickable_tasks.extend(task_regions);

    let tasks = app.tasks_for_column(status);
    let items: Vec<ListItem> = tasks
        .iter()
        .enumerate()
        .map(|(i, task)| {
            let wrapped = wrap_str(&task.title, max_title_chars);
            let mut lines: Vec<Line> = wrapped.into_iter().map(Line::raw).collect();
            lines.push(Line::raw(""));
            let text = Text::from(lines);
            let style = if is_focused_col && i == focused_idx {
                Style::default()
                    .bg(Color::Blue)
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };
            ListItem::new(text).style(style)
        })
        .collect();

    let border_style = if is_focused_col {
        Style::default().fg(Color::Blue)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let block = Block::default()
        .title(Span::styled(
            format!(" {} ({}) ", status.label(), tasks.len()),
            if is_focused_col {
                Style::default()
                    .fg(Color::Blue)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::Gray)
            },
        ))
        .borders(Borders::ALL)
        .border_style(border_style);

    let list = List::new(items).block(block);
    frame.render_widget(list, area);
}

// ── Memo panel ────────────────────────────────────────────────────────────────

const MEMO_ITEM_WIDTH: u16 = 24;

fn render_memo_panel(frame: &mut Frame, area: Rect, app: &mut AppState) {
    // Compute items-per-row and cache in AppState for navigation
    let items_per_row = (area.width / MEMO_ITEM_WIDTH).max(1);
    app.memo_cols = items_per_row as usize;

    let is_focused = app.focus_area == FocusArea::Memo;
    let border_style = if is_focused {
        Style::default().fg(Color::Blue)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let block = Block::default()
        .title(Span::styled(
            " Memo ",
            if is_focused {
                Style::default()
                    .fg(Color::Blue)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::Gray)
            },
        ))
        .borders(Borders::ALL)
        .border_style(border_style);

    // Inner area for item rendering
    let inner = block.inner(area);
    frame.render_widget(block, area);

    if inner.height == 0 || inner.width == 0 {
        return;
    }

    let item_w = MEMO_ITEM_WIDTH;
    let item_h: u16 = 1;

    for (idx, memo) in app.board.memos.iter().enumerate() {
        let row = (idx as u16) / items_per_row;
        let col = (idx as u16) % items_per_row;

        let y = inner.y + row * item_h;
        let x = inner.x + col * item_w;

        // Stop rendering if beyond panel height
        if y >= inner.y + inner.height {
            break;
        }

        let cell_width = item_w.min(inner.x + inner.width - x);
        let cell_rect = Rect {
            x,
            y,
            width: cell_width,
            height: item_h,
        };

        app.clickable_memos.push(MemoHitRegion {
            row: y,
            col_start: x,
            col_end: x + cell_width,
            memo_index: idx,
        });

        let style = if is_focused && idx == app.focused_memo {
            Style::default()
                .bg(Color::Blue)
                .fg(Color::White)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default()
        };

        let title = truncate_str(&memo.title, (item_w.saturating_sub(1)) as usize);
        let widget = Paragraph::new(title).style(style);
        frame.render_widget(widget, cell_rect);
    }
}

// ── Detail panel ──────────────────────────────────────────────────────────────

fn render_detail_panel(frame: &mut Frame, area: Rect, app: &mut AppState) {
    // Text content area: inside 1-cell border on each side.
    let available_width = area.width.saturating_sub(2);
    let text_base_col = area.x + 1;

    let content = if app.focus_area == FocusArea::Memo {
        match app.focused_memo_item() {
            Some(memo) => {
                // Compute detail URL regions if detail is non-empty.
                if !memo.detail.is_empty() {
                    let detail_base_row = area.y + 1 + 2; // title + blank
                    let detail_text = memo.detail.clone();
                    let regions = url::detail_url_regions(
                        &detail_text,
                        available_width,
                        detail_base_row,
                        text_base_col,
                    );
                    app.clickable_urls.extend(regions);
                }

                let memo = app.focused_memo_item().unwrap();
                let title_line = Line::from(vec![
                    Span::styled("Title: ", Style::default().add_modifier(Modifier::BOLD)),
                    Span::raw(memo.title.clone()),
                ]);
                let lines = if memo.detail.is_empty() {
                    vec![
                        title_line,
                        Line::raw(""),
                        Line::styled("(no detail)", Style::default().fg(Color::DarkGray)),
                    ]
                } else {
                    let mut lines = vec![title_line, Line::raw("")];
                    for dl in memo.detail.split('\n') {
                        for wrapped in wrap_str(dl, available_width as usize) {
                            lines.push(Line::raw(wrapped));
                        }
                    }
                    lines
                };
                Text::from(lines)
            }
            None => Text::styled("No memo selected", Style::default().fg(Color::DarkGray)),
        }
    } else {
        match app.focused_task() {
            Some(task) => {
                // Compute detail URL regions if detail is non-empty.
                if !task.detail.is_empty() {
                    let detail_base_row = area.y + 1 + 2; // title + blank
                    let detail_text = task.detail.clone();
                    let regions = url::detail_url_regions(
                        &detail_text,
                        available_width,
                        detail_base_row,
                        text_base_col,
                    );
                    app.clickable_urls.extend(regions);
                }

                let task = app.focused_task().unwrap();
                let title_line = Line::from(vec![
                    Span::styled("Title: ", Style::default().add_modifier(Modifier::BOLD)),
                    Span::raw(&task.title),
                ]);
                let detail_text = if task.detail.is_empty() {
                    vec![
                        title_line,
                        Line::raw(""),
                        Line::styled("(no detail)", Style::default().fg(Color::DarkGray)),
                    ]
                } else {
                    let mut lines = vec![title_line, Line::raw("")];
                    for dl in task.detail.split('\n') {
                        for wrapped in wrap_str(dl, available_width as usize) {
                            lines.push(Line::raw(wrapped));
                        }
                    }
                    lines
                };
                Text::from(detail_text)
            }
            None => Text::styled("No task selected", Style::default().fg(Color::DarkGray)),
        }
    };

    let panel = Paragraph::new(content)
        .block(
            Block::default()
                .title(" Detail ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::DarkGray)),
        );

    frame.render_widget(panel, area);
}

// ── Status bar ────────────────────────────────────────────────────────────────

fn render_status_bar(frame: &mut Frame, area: Rect, app: &mut AppState) {
    let default_hint = if app.focus_area == FocusArea::Memo {
        "a:add  e:title  E:detail  D:del  hjkl:nav  k:back  q:quit"
    } else {
        "a:add  e:title  E:detail  Enter:→  BS:←  D:del  J/K:move  j:memo  q:quit"
    };

    let msg = app.status_msg.as_deref().unwrap_or(default_hint);

    let style = if app.status_msg.is_some() {
        Style::default().fg(Color::Red)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let bar = Paragraph::new(msg).style(style);
    frame.render_widget(bar, area);
}

// ── Input popup ───────────────────────────────────────────────────────────────

pub fn render_input_popup(frame: &mut Frame, area: Rect, app: &mut AppState) {
    let popup_area = centered_rect(60, 20, area);

    // Clear the background
    frame.render_widget(Clear, popup_area);

    let title = match (app.mode.clone(), app.input.is_memo, app.input.is_create) {
        (AppMode::InputTitle, true, true) => " Add Memo ",
        (AppMode::InputTitle, true, false) => " Edit Memo Title ",
        (AppMode::InputDetail, true, _) => " Edit Memo Detail ",
        (AppMode::InputTitle, false, true) => " Add Task ",
        (AppMode::InputTitle, false, false) => " Edit Title ",
        (AppMode::InputDetail, false, _) => " Edit Detail ",
        (AppMode::Normal, _, _) => unreachable!(),
    };

    let available_width = popup_area.width.saturating_sub(2) as usize;
    let available_height = popup_area.height.saturating_sub(2) as usize;
    let content_x = popup_area.x + 1;
    let content_y = popup_area.y + 1;

    let buffer = app.input.value();
    let cursor = app.input.cursor;

    // Pre-wrap each logical line for correct CJK display
    let wrapped_lines = wrap_lines(buffer, available_width);

    // Compute visual cursor position
    let (cursor_row, cursor_col) = cursor_visual_position(buffer, cursor, available_width);

    // Scroll offset: keep cursor visible within the popup
    let scroll_offset: usize = if available_height > 0 && cursor_row >= available_height {
        cursor_row - available_height + 1
    } else {
        0
    };

    // Render only the visible slice of wrapped lines
    let visible_lines: Vec<Line> = wrapped_lines
        .iter()
        .skip(scroll_offset)
        .take(available_height)
        .map(|l| Line::raw(l.clone()))
        .collect();

    let popup = Paragraph::new(Text::from(visible_lines))
        .block(
            Block::default()
                .title(title)
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Yellow)),
        )
        .style(Style::default().fg(Color::White));

    frame.render_widget(popup, popup_area);

    // Set terminal cursor at the computed position
    let screen_row = cursor_row.saturating_sub(scroll_offset);
    if screen_row < available_height {
        frame.set_cursor_position(Position {
            x: content_x + cursor_col as u16,
            y: content_y + screen_row as u16,
        });
    }
}

// ── Helpers ───────────────────────────────────────────────────────────────────

/// Create a centred rect of the given percentage dimensions.
pub fn centered_rect(pct_x: u16, pct_y: u16, area: Rect) -> Rect {
    let vert = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - pct_y) / 2),
            Constraint::Percentage(pct_y),
            Constraint::Percentage((100 - pct_y) / 2),
        ])
        .split(area);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - pct_x) / 2),
            Constraint::Percentage(pct_x),
            Constraint::Percentage((100 - pct_x) / 2),
        ])
        .split(vert[1])[1]
}

/// Wrap multi-line text: split by `\n`, then wrap each logical line at `max_width`.
fn wrap_lines(s: &str, max_width: usize) -> Vec<String> {
    s.split('\n')
        .flat_map(|line| wrap_str(line, max_width))
        .collect()
}

/// Compute the visual `(row, col)` of `cursor` (a byte offset in `buffer`) when
/// the text is displayed with lines pre-wrapped at `wrap_width` display columns.
/// Both row and col are 0-indexed.
fn cursor_visual_position(buffer: &str, cursor: usize, wrap_width: usize) -> (usize, usize) {
    let cursor = cursor.min(buffer.len());
    // Work only with the text before the cursor position
    let text_before = &buffer[..cursor];
    let logical_lines: Vec<&str> = text_before.split('\n').collect();
    let n = logical_lines.len();
    let mut visual_row = 0usize;

    for (i, &line) in logical_lines.iter().enumerate() {
        let wrapped = wrap_str(line, wrap_width);
        if i < n - 1 {
            // Full logical line — count all its visual rows
            visual_row += wrapped.len();
        } else {
            // Last segment — cursor is at the end of this text
            visual_row += wrapped.len().saturating_sub(1);
            let last_visual = wrapped.last().map(String::as_str).unwrap_or("");
            let col: usize = last_visual.chars().map(|c| c.width().unwrap_or(0)).sum();
            return (visual_row, col);
        }
    }
    (0, 0)
}

/// Wrap a string into lines whose display width is at most `max_width` columns.
/// CJK / full-width characters count as 2 columns.
fn wrap_str(s: &str, max_width: usize) -> Vec<String> {
    if max_width == 0 {
        return vec![s.to_string()];
    }
    let mut lines = Vec::new();
    let mut line = String::new();
    let mut width = 0usize;
    for ch in s.chars() {
        let cw = ch.width().unwrap_or(0);
        if width + cw > max_width && !line.is_empty() {
            lines.push(std::mem::take(&mut line));
            width = 0;
        }
        line.push(ch);
        width += cw;
    }
    if !line.is_empty() {
        lines.push(line);
    }
    if lines.is_empty() {
        lines.push(String::new());
    }
    lines
}

/// Truncate a string so its display width is at most `max_width` columns.
/// CJK / full-width characters count as 2 columns.
fn truncate_str(s: &str, max_width: usize) -> String {
    let mut width = 0usize;
    let mut out = String::new();
    for ch in s.chars() {
        let cw = ch.width().unwrap_or(0);
        if width + cw > max_width {
            if max_width >= 1 {
                out.push('…');
            }
            return out;
        }
        out.push(ch);
        width += cw;
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── wrap_str ──────────────────────────────────────────────────────────

    #[test]
    fn wrap_ascii_short_no_wrap() {
        assert_eq!(wrap_str("hello", 10), vec!["hello"]);
    }

    #[test]
    fn wrap_ascii_exact_width() {
        assert_eq!(wrap_str("abcde", 5), vec!["abcde"]);
    }

    #[test]
    fn wrap_ascii_splits() {
        assert_eq!(wrap_str("abcdefgh", 5), vec!["abcde", "fgh"]);
    }

    #[test]
    fn wrap_cjk_respects_double_width() {
        // Each CJK char = 2 cols. With max_width=6, only 3 chars fit per line.
        let s = "あいうえお"; // 5 chars, 10 cols
        let result = wrap_str(s, 6);
        assert_eq!(result, vec!["あいう", "えお"]);
    }

    #[test]
    fn wrap_cjk_exact_width() {
        // 3 chars = 6 cols, max_width=6 → no wrap
        assert_eq!(wrap_str("あいう", 6), vec!["あいう"]);
    }

    #[test]
    fn wrap_mixed_ascii_and_cjk() {
        // "aあb" = 1+2+1 = 4 cols → fits in width 4
        // "cう"  = 1+2   = 3 cols
        let s = "aあbcう";
        let result = wrap_str(s, 4);
        assert_eq!(result, vec!["aあb", "cう"]);
    }

    #[test]
    fn wrap_cjk_boundary_breaks_before_overflow() {
        // max_width=5: "あい" = 4 cols, next "う" would be 6 → wrap
        let s = "あいう";
        let result = wrap_str(s, 5);
        assert_eq!(result, vec!["あい", "う"]);
    }

    #[test]
    fn wrap_zero_width_returns_whole_string() {
        assert_eq!(wrap_str("test", 0), vec!["test"]);
    }

    #[test]
    fn wrap_empty_string() {
        assert_eq!(wrap_str("", 10), vec![""]);
    }

    // ── truncate_str ──────────────────────────────────────────────────────

    #[test]
    fn truncate_ascii_short_no_truncation() {
        assert_eq!(truncate_str("hello", 10), "hello");
    }

    #[test]
    fn truncate_ascii_exact_width() {
        assert_eq!(truncate_str("abcde", 5), "abcde");
    }

    #[test]
    fn truncate_ascii_adds_ellipsis() {
        assert_eq!(truncate_str("abcdefgh", 5), "abcde…");
    }

    #[test]
    fn truncate_cjk_respects_double_width() {
        // "あいう" = 6 cols, max_width=5 → "あい" (4 cols) + "…"
        assert_eq!(truncate_str("あいうえお", 5), "あい…");
    }

    #[test]
    fn truncate_cjk_exact_fit() {
        // "あいう" = 6 cols, max_width=6 → fits exactly
        assert_eq!(truncate_str("あいう", 6), "あいう");
    }

    #[test]
    fn truncate_mixed_ascii_and_cjk() {
        // "aあbc" = 1+2+1+1 = 5 cols, max_width=4 → "aあb" (4 cols) + "…"
        assert_eq!(truncate_str("aあbc", 4), "aあb…");
    }

    #[test]
    fn truncate_empty_string() {
        assert_eq!(truncate_str("", 10), "");
    }

    // ── T014: wrap_lines (multi-line detail) ──────────────────────────────

    #[test]
    fn wrap_lines_ascii_multiline() {
        // Two logical lines, each wrapped independently
        let result = wrap_lines("hello\nworld", 4);
        assert_eq!(result, vec!["hell", "o", "worl", "d"]);
    }

    #[test]
    fn wrap_lines_cjk_multiline() {
        // CJK lines: each fits exactly at width 4 (2 chars × 2 cols)
        let result = wrap_lines("あい\nうえ", 4);
        assert_eq!(result, vec!["あい", "うえ"]);
    }

    #[test]
    fn wrap_lines_mixed_multiline() {
        // "aあ" = 1+2=3 cols fits at width 3; "bい" = 1+2=3 fits at width 3
        let result = wrap_lines("aあ\nbい", 3);
        assert_eq!(result, vec!["aあ", "bい"]);
    }

    #[test]
    fn wrap_lines_single_line_unchanged() {
        let result = wrap_lines("hello", 10);
        assert_eq!(result, vec!["hello"]);
    }

    // T015: full-width char at wrap boundary moves to next line

    #[test]
    fn wrap_lines_cjk_at_boundary_1col_remaining_wraps() {
        // "ab" = 2 cols, then 'あ' = 2 cols; only 1 col remains → 'あ' wraps
        let result = wrap_lines("abあ", 3);
        assert_eq!(result, vec!["ab", "あ"]);
    }

    #[test]
    fn wrap_lines_cjk_exactly_fills_width_no_extra_wrap() {
        // "aあ" = 1+2 = 3 cols at width 3 → fits on one line
        let result = wrap_lines("aあ", 3);
        assert_eq!(result, vec!["aあ"]);
    }

    // ── T018: cursor_visual_position ──────────────────────────────────────

    #[test]
    fn cursor_visual_pos_empty_buffer() {
        assert_eq!(cursor_visual_position("", 0, 10), (0, 0));
    }

    #[test]
    fn cursor_visual_pos_ascii_no_wrap() {
        // "hello" at width 10, cursor at byte 3 → row 0, col 3
        assert_eq!(cursor_visual_position("hello", 3, 10), (0, 3));
    }

    #[test]
    fn cursor_visual_pos_ascii_at_end() {
        // cursor at end of "hello" → row 0, col 5
        assert_eq!(cursor_visual_position("hello", 5, 10), (0, 5));
    }

    #[test]
    fn cursor_visual_pos_wraps_to_next_row() {
        // "hello world" at width 5:
        //   wrap_str("hello world", 5) → ["hello", " worl", "d"]
        // cursor = 8 → text_before = "hello wo"
        //   wrap_str("hello wo", 5) → ["hello", " wo"]  → row 1, col 3
        assert_eq!(cursor_visual_position("hello world", 8, 5), (1, 3));
    }

    #[test]
    fn cursor_visual_pos_with_newline() {
        // "hi\nworld", cursor at byte 5 (= "hi\nwo" → after 'o' in "wo")
        // text_before = "hi\nwo" → split → ["hi", "wo"]
        // "hi" → 1 visual row → visual_row = 1
        // "wo" last → col = 2
        assert_eq!(cursor_visual_position("hi\nworld", 5, 10), (1, 2));
    }

    #[test]
    fn cursor_visual_pos_cjk() {
        // "あいう" at width 4: wrap → ["あい", "う"]
        // cursor at byte 9 (after all 3 chars) → text_before = "あいう"
        // wrap_str("あいう", 4) → ["あい", "う"] → row 1, col 2
        assert_eq!(cursor_visual_position("あいう", 9, 4), (1, 2));
    }

    #[test]
    fn cursor_visual_pos_cjk_mid() {
        // "あいう" cursor at byte 3 (after 'あ') → text_before = "あ"
        // wrap_str("あ", 4) → ["あ"] → row 0, col 2
        assert_eq!(cursor_visual_position("あいう", 3, 4), (0, 2));
    }

    #[test]
    fn cursor_visual_pos_at_newline_char() {
        // "hello\nworld", cursor at byte 5 (before '\n') → text_before = "hello"
        // split('\n') → ["hello"] → wrap_str("hello", 10) → ["hello"]
        // row 0, col 5
        assert_eq!(cursor_visual_position("hello\nworld", 5, 10), (0, 5));
    }

    #[test]
    fn cursor_visual_pos_start_of_second_line() {
        // "hello\nworld", cursor at byte 6 (after '\n') → text_before = "hello\n"
        // split('\n') → ["hello", ""] → "hello" → 1 row, "" last → row 1, col 0
        assert_eq!(cursor_visual_position("hello\nworld", 6, 10), (1, 0));
    }
}
