use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Wrap},
    Frame,
};

use crate::app::AppState;
use crate::model::{AppMode, Status, ALL_STATUSES};
use crate::url;

// ── Top-level render ─────────────────────────────────────────────────────────

pub fn render(frame: &mut Frame, app: &mut AppState) {
    let area = frame.area();

    // Guard: show warning if terminal is too small
    if area.width < 40 || area.height < 10 {
        let warning = Paragraph::new("Terminal too small.\nMinimum: 40×10")
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

    // Equal 4-column split
    let columns = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Ratio(1, 4),
            Constraint::Ratio(1, 4),
            Constraint::Ratio(1, 4),
            Constraint::Ratio(1, 4),
        ])
        .split(kanban_area);

    for (i, &status) in ALL_STATUSES.iter().enumerate() {
        render_column(frame, columns[i], app, status, i == app.focused_col);
    }

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
    let tasks = app.tasks_for_column(status);
    let focused_idx = app.focused_card[status.col_index()];

    // Compute URL hit regions for each visible item before building ListItems.
    // Item row: area.y + 1 (top border) + visual_index.
    // Text starts at area.x + 1 (left border).
    let max_title_chars = area.width.saturating_sub(4) as usize;
    let text_x = area.x + 1;
    let mut url_regions = Vec::new();
    for (vi, task) in tasks.iter().enumerate() {
        let title = truncate_str(&task.title, max_title_chars);
        let item_row = area.y + 1 + vi as u16;
        let regions = url::list_item_url_regions(&title, item_row, text_x);
        url_regions.extend(regions);
    }
    app.clickable_urls.extend(url_regions);

    let tasks = app.tasks_for_column(status);
    let items: Vec<ListItem> = tasks
        .iter()
        .enumerate()
        .map(|(i, task)| {
            let title = truncate_str(&task.title, max_title_chars);
            let style = if is_focused_col && i == focused_idx {
                Style::default()
                    .bg(Color::Blue)
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };
            ListItem::new(title).style(style)
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

// ── Detail panel ──────────────────────────────────────────────────────────────

fn render_detail_panel(frame: &mut Frame, area: Rect, app: &mut AppState) {
    // Text content area: inside 1-cell border on each side.
    let available_width = area.width.saturating_sub(2);
    let text_base_col = area.x + 1;
    // Row 0 of text = area.y + 1 (top border).
    // We'll compute base_row per logical line below.

    let content = match app.focused_task() {
        Some(task) => {
            // Compute detail URL regions if detail is non-empty.
            // The detail text starts at text row 2 (title_line + blank line).
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
    let msg = app
        .status_msg
        .as_deref()
        .unwrap_or("a:add  e:title  E:detail  Enter:→  BS:←  D:del  J/K:move  q:quit");

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

    let title = match app.mode {
        AppMode::InputTitle if app.input.is_create => " Add Task ",
        AppMode::InputTitle => " Edit Title ",
        AppMode::InputDetail => " Edit Detail ",
        AppMode::Normal => unreachable!(),
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
