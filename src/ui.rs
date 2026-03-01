use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Wrap},
    Frame,
};

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
            let lines: Vec<Line> = wrapped.into_iter().map(Line::raw).collect();
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
                        lines.push(Line::raw(dl.to_string()));
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
                        lines.push(Line::raw(dl.to_string()));
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
        )
        .wrap(Wrap { trim: false });

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

    let display = format!("{}_", app.input.value()); // trailing _ simulates cursor
    let popup = Paragraph::new(display)
        .block(
            Block::default()
                .title(title)
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Yellow)),
        )
        .style(Style::default().fg(Color::White));

    frame.render_widget(popup, popup_area);
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

/// Wrap a string into lines of at most `max_chars` characters (Unicode-aware).
fn wrap_str(s: &str, max_chars: usize) -> Vec<String> {
    if max_chars == 0 {
        return vec![s.to_string()];
    }
    let chars: Vec<char> = s.chars().collect();
    if chars.len() <= max_chars {
        return vec![s.to_string()];
    }
    chars.chunks(max_chars).map(|c| c.iter().collect()).collect()
}

/// Truncate a string to at most `max_chars` characters (Unicode-aware).
fn truncate_str(s: &str, max_chars: usize) -> String {
    if s.chars().count() <= max_chars {
        s.to_string()
    } else {
        let mut out: String = s.chars().take(max_chars.saturating_sub(1)).collect();
        out.push('…');
        out
    }
}
