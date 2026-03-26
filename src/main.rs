use std::io;
use std::panic;

use ratatui::crossterm::{
    event::{
        self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyModifiers, MouseButton,
        MouseEvent, MouseEventKind,
    },
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};

use tudo::app::{handle_left_click, AppState};
use tudo::model::{AppMode, BoardState, FocusArea};
use tudo::storage;
use tudo::ui;

fn main() -> io::Result<()> {
    // ── Load board ────────────────────────────────────────────────────────
    let board = match storage::load_board() {
        Ok(b) => b,
        Err(e) => {
            eprintln!("Warning: could not load board — {e}. Starting empty.");
            BoardState::default()
        }
    };
    let mut app = AppState::new(board);
    app.apply_daily_filter();

    // ── Terminal setup ────────────────────────────────────────────────────
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // ── Panic hook — always restore terminal ─────────────────────────────
    let original_hook = panic::take_hook();
    panic::set_hook(Box::new(move |info| {
        let _ = disable_raw_mode();
        let _ = execute!(io::stdout(), LeaveAlternateScreen, DisableMouseCapture);
        original_hook(info);
    }));

    // ── Event loop ────────────────────────────────────────────────────────
    let result = run_app(&mut terminal, &mut app);

    // ── Cleanup ───────────────────────────────────────────────────────────
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(e) = result {
        eprintln!("Error: {e}");
    }
    Ok(())
}

fn run_app(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    app: &mut AppState,
) -> io::Result<()> {
    let mut reorder_save_pending = false;
    let mut last_reorder_at: Option<std::time::Instant> = None;

    loop {
        app.clickable_urls.clear();
        app.clickable_tasks.clear();
        app.clickable_memos.clear();
        terminal.draw(|frame| ui::render(frame, app))?;

        if !event::poll(std::time::Duration::from_millis(200))? {
            if reorder_save_pending
                && last_reorder_at
                    .map(|t| t.elapsed() >= std::time::Duration::from_secs(1))
                    .unwrap_or(false)
            {
                if let Err(e) = storage::save_board(&mut app.board) {
                    app.status_msg = Some(e.to_string());
                }
                reorder_save_pending = false;
            }
            continue;
        }

        match event::read()? {
            Event::Resize(_, _) => {
                // ratatui handles resize on the next draw; no action needed
            }
            Event::Key(key) => {
                // Quit shortcuts — flush pending reorder save before exit
                if key.code == KeyCode::Char('q') && app.mode == AppMode::Normal {
                    if reorder_save_pending {
                        let _ = storage::save_board(&mut app.board);
                    }
                    break;
                }
                if key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL) {
                    if reorder_save_pending {
                        let _ = storage::save_board(&mut app.board);
                    }
                    break;
                }

                let is_reorder = app.mode == AppMode::Normal
                    && app.focus_area == FocusArea::Kanban
                    && matches!(key.code, KeyCode::Char('J') | KeyCode::Char('K'));

                match app.mode {
                    AppMode::Normal => handle_normal_key(app, key.code),
                    AppMode::InputTitle | AppMode::InputDetail => {
                        handle_input_key(app, key.code, key.modifiers)
                    }
                }

                if is_reorder {
                    reorder_save_pending = true;
                    last_reorder_at = Some(std::time::Instant::now());
                } else {
                    // Cancel debounce; the upcoming save captures all reorder changes too
                    reorder_save_pending = false;
                    if let Err(e) = storage::save_board(&mut app.board) {
                        app.status_msg = Some(e.to_string());
                    }
                }
            }
            Event::Mouse(MouseEvent {
                kind: MouseEventKind::Down(MouseButton::Left),
                column,
                row,
                ..
            }) => {
                handle_left_click(app, column, row);
                if let Err(e) = storage::save_board(&mut app.board) {
                    app.status_msg = Some(e.to_string());
                }
            }
            _ => {}
        }
    }
    Ok(())
}

fn handle_normal_key(app: &mut AppState, code: KeyCode) {
    // Focus-switch shortcuts work from any panel
    match code {
        KeyCode::Char('m') => {
            app.focus_area = FocusArea::Memo;
            return;
        }
        KeyCode::Char('t') => {
            app.focus_area = FocusArea::Kanban;
            return;
        }
        _ => {}
    }

    if app.focus_area == FocusArea::Memo {
        match code {
            KeyCode::Char('h') | KeyCode::Left => app.move_memo_left(),
            KeyCode::Char('l') | KeyCode::Right => app.move_memo_right(),
            KeyCode::Char('k') | KeyCode::Up => app.move_memo_up(),
            KeyCode::Char('j') | KeyCode::Down => app.move_memo_down(),
            KeyCode::Char('a') => app.open_create_memo(),
            KeyCode::Char('e') => app.open_edit_memo_title(),
            KeyCode::Char('E') => app.open_edit_memo_detail(),
            KeyCode::Char('D') => app.delete_focused_memo(),
            _ => {}
        }
        return;
    }

    // FocusArea::Kanban
    match code {
        KeyCode::Char('h') | KeyCode::Left => app.move_left(),
        KeyCode::Char('l') | KeyCode::Right => app.move_right(),
        KeyCode::Char('k') | KeyCode::Up => app.move_up(),
        KeyCode::Char('j') | KeyCode::Down => app.kanban_try_move_down(),
        KeyCode::Char('a') => app.open_create(),
        KeyCode::Char('e') => app.open_edit_title(),
        KeyCode::Char('J') => {
            app.reorder_task_down();
        }
        KeyCode::Char('K') => {
            app.reorder_task_up();
        }
        KeyCode::Char('E') => app.open_edit_detail(),
        KeyCode::Enter => {
            // Capture id before mutation
            let id = app.focused_task().map(|t| t.id);
            app.advance_status();
            // If the task just moved to Done, append to daily log
            if let Some(id) = id {
                if let Some(entry) = app.make_done_entry_for(id) {
                    if let Err(e) = tudo::storage::append_done_entry(&entry) {
                        app.status_msg = Some(e.to_string());
                    }
                }
            }
        }
        KeyCode::Backspace => app.retreat_status(),
        KeyCode::Char('D') => {
            if app.focused_task().is_some() {
                app.delete_focused_card();
                let col_status = tudo::model::ALL_STATUSES[app.focused_col];
                if app.tasks_for_column(col_status).is_empty() {
                    app.status_msg = Some("Column empty".to_string());
                }
            }
        }
        _ => {}
    }
}

fn handle_input_key(app: &mut AppState, code: KeyCode, modifiers: KeyModifiers) {
    match code {
        // Ctrl+J inserts a newline in detail mode only
        KeyCode::Char('j') if modifiers.contains(KeyModifiers::CONTROL) => {
            if app.mode == AppMode::InputDetail {
                app.input.insert_char('\n');
            }
        }
        KeyCode::Enter => app.confirm_input(),
        KeyCode::Esc => app.cancel_input(),
        KeyCode::Backspace => app.input.delete_char_back(),
        KeyCode::Char(c) => app.input.insert_char(c),
        KeyCode::Left => app.input.move_left(),
        KeyCode::Right => app.input.move_right(),
        KeyCode::Home => app.input.move_home(),
        KeyCode::End => app.input.move_end(),
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tudo::model::BoardState;

    fn make_app() -> AppState {
        AppState::new(BoardState::default())
    }

    // T002 [US1]: pressing 'm' from Kanban focus moves focus to Memo
    #[test]
    fn test_m_key_focuses_memo() {
        let mut app = make_app();
        app.focus_area = FocusArea::Kanban;
        handle_normal_key(&mut app, KeyCode::Char('m'));
        assert_eq!(app.focus_area, FocusArea::Memo);
    }

    // T003 [US1]: pressing 'm' when already in Memo focus is idempotent
    #[test]
    fn test_m_key_idempotent_when_memo_focused() {
        let mut app = make_app();
        app.focus_area = FocusArea::Memo;
        handle_normal_key(&mut app, KeyCode::Char('m'));
        assert_eq!(app.focus_area, FocusArea::Memo);
    }

    // T005 [US2]: pressing 't' from Memo focus moves focus to Kanban
    #[test]
    fn test_t_key_focuses_kanban() {
        let mut app = make_app();
        app.focus_area = FocusArea::Memo;
        handle_normal_key(&mut app, KeyCode::Char('t'));
        assert_eq!(app.focus_area, FocusArea::Kanban);
    }

    // T006 [US2]: pressing 't' when already in Kanban focus is idempotent
    #[test]
    fn test_t_key_idempotent_when_kanban_focused() {
        let mut app = make_app();
        app.focus_area = FocusArea::Kanban;
        handle_normal_key(&mut app, KeyCode::Char('t'));
        assert_eq!(app.focus_area, FocusArea::Kanban);
    }
}
