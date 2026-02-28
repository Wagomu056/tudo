use chrono::Local;

use crate::model::{
    AppError, AppMode, BoardState, DoneEntry, InputState, Status, Task, UrlHitRegion,
};

pub const NUM_COLS: usize = 4;

// ── AppState ─────────────────────────────────────────────────────────────────

pub struct AppState {
    pub board: BoardState,
    /// Index of the focused column (0 = Todo … 3 = Done).
    pub focused_col: usize,
    /// Per-column card cursor — index into that column's task list.
    pub focused_card: [usize; NUM_COLS],
    pub mode: AppMode,
    pub input: InputState,
    /// Transient message shown in the status bar (errors, hints).
    pub status_msg: Option<String>,
    /// URL hit regions computed during each render frame; cleared at frame start.
    pub clickable_urls: Vec<UrlHitRegion>,
}

impl AppState {
    pub fn new(board: BoardState) -> Self {
        AppState {
            board,
            focused_col: 0,
            focused_card: [0; NUM_COLS],
            mode: AppMode::Normal,
            input: InputState::default(),
            status_msg: None,
            clickable_urls: Vec::new(),
        }
    }

    // ── Query helpers ──────────────────────────────────────────────────────

    /// Return references to tasks belonging to the given column status.
    pub fn tasks_for_column(&self, status: Status) -> Vec<&Task> {
        self.board
            .tasks
            .iter()
            .filter(|t| t.status == status)
            .collect()
    }

    /// Return a reference to the currently focused task, if any.
    pub fn focused_task(&self) -> Option<&Task> {
        let status = crate::model::ALL_STATUSES[self.focused_col];
        let col = self.tasks_for_column(status);
        col.get(self.focused_card[self.focused_col]).copied()
    }

    /// ID of the focused task, if any.
    fn focused_task_id(&self) -> Option<u64> {
        self.focused_task().map(|t| t.id)
    }

    // ── Navigation ────────────────────────────────────────────────────────

    pub fn move_left(&mut self) {
        if self.focused_col > 0 {
            self.focused_col -= 1;
        }
    }

    pub fn move_right(&mut self) {
        if self.focused_col < NUM_COLS - 1 {
            self.focused_col += 1;
        }
    }

    pub fn move_up(&mut self) {
        let idx = &mut self.focused_card[self.focused_col];
        if *idx > 0 {
            *idx -= 1;
        }
    }

    pub fn move_down(&mut self) {
        let status = crate::model::ALL_STATUSES[self.focused_col];
        let len = self.tasks_for_column(status).len();
        let idx = &mut self.focused_card[self.focused_col];
        if *idx + 1 < len {
            *idx += 1;
        }
    }

    // ── Status lifecycle ──────────────────────────────────────────────────

    /// Advance the focused card to the next status (Enter key).
    pub fn advance_status(&mut self) {
        let id = match self.focused_task_id() {
            Some(id) => id,
            None => return,
        };
        if let Some(task) = self.board.tasks.iter_mut().find(|t| t.id == id) {
            if let Some(next) = task.status.next() {
                task.status = next;
                if next == Status::Done {
                    task.done_at = Some(Local::now());
                }
            }
        }
        self.focus_task_by_id(id);
        self.clamp_focus();
    }

    /// Move the focused card back to the previous status (BackSpace key).
    pub fn retreat_status(&mut self) {
        let id = match self.focused_task_id() {
            Some(id) => id,
            None => return,
        };
        if let Some(task) = self.board.tasks.iter_mut().find(|t| t.id == id) {
            if let Some(prev) = task.status.prev() {
                task.status = prev;
                if prev != Status::Done {
                    task.done_at = None;
                }
            }
        }
        self.focus_task_by_id(id);
        self.clamp_focus();
    }

    /// Permanently delete the focused card (D key).
    pub fn delete_focused_card(&mut self) {
        let id = match self.focused_task_id() {
            Some(id) => id,
            None => return,
        };
        self.board.tasks.retain(|t| t.id != id);
        self.clamp_focus();
    }

    /// Move `focused_col` and `focused_card` to follow the task with the given
    /// id to its current (possibly new) column and position within that column.
    fn focus_task_by_id(&mut self, id: u64) {
        let status = match self.board.tasks.iter().find(|t| t.id == id) {
            Some(task) => task.status,
            None => return,
        };
        let col = status.col_index();
        let pos = {
            let col_tasks = self.tasks_for_column(status);
            col_tasks.iter().position(|t| t.id == id).unwrap_or(0)
        };
        self.focused_col = col;
        self.focused_card[col] = pos;
    }

    /// Clamp all per-column cursors so they remain within valid bounds.
    pub fn clamp_focus(&mut self) {
        for col in 0..NUM_COLS {
            let status = crate::model::ALL_STATUSES[col];
            let len = self.tasks_for_column(status).len();
            if len == 0 {
                self.focused_card[col] = 0;
            } else if self.focused_card[col] >= len {
                self.focused_card[col] = len - 1;
            }
        }
    }

    // ── Daily Done filter (applied at startup) ────────────────────────────

    /// Discard Done tasks whose `done_at` date is not today.
    pub fn apply_daily_filter(&mut self) {
        let today = Local::now().date_naive();
        self.board.tasks.retain(|t| {
            t.status != Status::Done || t.done_at.map(|d| d.date_naive() == today).unwrap_or(false)
        });
        self.clamp_focus();
    }

    // ── Input mode ────────────────────────────────────────────────────────

    /// Open the title-input popup for creating a new task (a key).
    pub fn open_create(&mut self) {
        self.input.clear();
        self.input.is_create = true;
        self.mode = AppMode::InputTitle;
        self.status_msg = None;
    }

    /// Open the title-edit popup for the focused card (e key).
    pub fn open_edit_title(&mut self) {
        let title = match self.focused_task() {
            Some(t) => t.title.clone(),
            None => return,
        };
        self.input.clear();
        self.input.buffer = title;
        self.input.is_create = false;
        self.mode = AppMode::InputTitle;
        self.status_msg = None;
    }

    /// Open the detail-edit popup for the focused card (E key).
    pub fn open_edit_detail(&mut self) {
        let detail = match self.focused_task() {
            Some(t) => t.detail.clone(),
            None => return,
        };
        self.input.clear();
        self.input.buffer = detail;
        self.input.is_create = false;
        self.mode = AppMode::InputDetail;
        self.status_msg = None;
    }

    /// Confirm the input popup (Enter key in Input mode).
    pub fn confirm_input(&mut self) {
        let value = self.input.value().trim().to_string();
        if value.is_empty() {
            self.status_msg = Some("Title cannot be empty".to_string());
            self.cancel_input();
            return;
        }

        if self.input.is_create {
            let id = self.board.alloc_id();
            let task = Task::new(id, value);
            self.board.tasks.push(task);
            self.focus_task_by_id(id);
        } else {
            let id = match self.focused_task_id() {
                Some(id) => id,
                None => {
                    self.cancel_input();
                    return;
                }
            };
            let is_detail = self.mode == AppMode::InputDetail;
            if let Some(task) = self.board.tasks.iter_mut().find(|t| t.id == id) {
                if is_detail {
                    task.detail = value;
                } else {
                    task.title = value;
                }
            }
        }

        self.input.clear();
        self.mode = AppMode::Normal;
    }

    /// Cancel the input popup (Esc key in Input mode).
    pub fn cancel_input(&mut self) {
        self.input.clear();
        self.mode = AppMode::Normal;
    }

    // ── Done entry helper ─────────────────────────────────────────────────

    /// Collect a DoneEntry if the last advance moved the focused task to Done.
    /// Call after `advance_status` when the status became Done.
    pub fn make_done_entry_for(&self, id: u64) -> Option<DoneEntry> {
        self.board
            .tasks
            .iter()
            .find(|t| t.id == id && t.status == Status::Done)
            .map(DoneEntry::from_task)
    }

    // ── Error display ─────────────────────────────────────────────────────

    pub fn set_error(&mut self, err: AppError) {
        self.status_msg = Some(err.to_string());
    }
}

// ── Mouse click handling ──────────────────────────────────────────────────────

/// Handle a left mouse click at terminal position `(col, row)`.
/// If the click falls within a URL hit region, opens the URL in the browser.
/// Navigation state (`focused_col`, `focused_card`) is never mutated here.
pub fn handle_left_click(app: &mut AppState, col: u16, row: u16) {
    for region in &app.clickable_urls {
        if row == region.row && col >= region.col_start && col < region.col_end {
            if let Err(e) = crate::url::open_url(&region.url) {
                app.status_msg = Some(format!("Cannot open URL: {e}"));
            }
            return;
        }
    }
    // Non-URL click: silently ignored.
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{BoardState, Status, Task};

    // ── T001: Test helper ─────────────────────────────────────────────────

    /// Build an AppState with the given tasks placed at the specified statuses.
    /// `focused_col` and `focused_card` are set to column 0, index 0 by default;
    /// tests adjust them as needed.
    fn make_app_with_tasks(specs: &[(u64, &str, Status)]) -> AppState {
        let tasks: Vec<Task> = specs
            .iter()
            .map(|&(id, title, status)| {
                let mut t = Task::new(id, title.to_string());
                t.status = status;
                t
            })
            .collect();
        let next_id = specs.iter().map(|s| s.0).max().unwrap_or(0) + 1;
        AppState::new(BoardState::with_tasks(tasks, next_id))
    }

    // ── T003: US1 — advance keeps focus on task ───────────────────────────

    #[test]
    fn test_advance_status_keeps_focus_on_task() {
        let mut app = make_app_with_tasks(&[(1, "task", Status::Doing)]);
        app.focused_col = Status::Doing.col_index();
        app.focused_card[Status::Doing.col_index()] = 0;

        app.advance_status();

        assert_eq!(app.focused_col, Status::Checking.col_index());
        assert_eq!(app.focused_card[Status::Checking.col_index()], 0);
    }

    // ── T004: US1 — advance tracks focus through all statuses ────────────

    #[test]
    fn test_advance_moves_focus_across_all_statuses() {
        let mut app = make_app_with_tasks(&[(1, "task", Status::Todo)]);
        app.focused_col = Status::Todo.col_index();
        app.focused_card[Status::Todo.col_index()] = 0;

        // Todo → Doing
        app.advance_status();
        assert_eq!(app.focused_col, Status::Doing.col_index());
        assert_eq!(app.focused_card[Status::Doing.col_index()], 0);

        // Doing → Checking
        app.advance_status();
        assert_eq!(app.focused_col, Status::Checking.col_index());
        assert_eq!(app.focused_card[Status::Checking.col_index()], 0);

        // Checking → Done
        app.advance_status();
        assert_eq!(app.focused_col, Status::Done.col_index());
        assert_eq!(app.focused_card[Status::Done.col_index()], 0);
    }

    // ── T005: US1 — boundary at Done preserves focus (no-op) ─────────────

    #[test]
    fn test_advance_at_done_boundary_preserves_focus() {
        let mut app = make_app_with_tasks(&[(1, "task", Status::Done)]);
        app.focused_col = Status::Done.col_index();
        app.focused_card[Status::Done.col_index()] = 0;

        app.advance_status();

        assert_eq!(app.focused_col, Status::Done.col_index());
        assert_eq!(app.focused_card[Status::Done.col_index()], 0);
    }

    // ── T009: US2 — retreat keeps focus on task ───────────────────────────

    #[test]
    fn test_retreat_status_keeps_focus_on_task() {
        let mut app = make_app_with_tasks(&[(1, "task", Status::Doing)]);
        app.focused_col = Status::Doing.col_index();
        app.focused_card[Status::Doing.col_index()] = 0;

        app.retreat_status();

        assert_eq!(app.focused_col, Status::Todo.col_index());
        assert_eq!(app.focused_card[Status::Todo.col_index()], 0);
    }

    // ── T010: US2 — retreat tracks focus through all statuses ────────────

    #[test]
    fn test_retreat_moves_focus_across_all_statuses() {
        let mut app = make_app_with_tasks(&[(1, "task", Status::Done)]);
        app.focused_col = Status::Done.col_index();
        app.focused_card[Status::Done.col_index()] = 0;

        // Done → Checking
        app.retreat_status();
        assert_eq!(app.focused_col, Status::Checking.col_index());
        assert_eq!(app.focused_card[Status::Checking.col_index()], 0);

        // Checking → Doing
        app.retreat_status();
        assert_eq!(app.focused_col, Status::Doing.col_index());
        assert_eq!(app.focused_card[Status::Doing.col_index()], 0);

        // Doing → Todo
        app.retreat_status();
        assert_eq!(app.focused_col, Status::Todo.col_index());
        assert_eq!(app.focused_card[Status::Todo.col_index()], 0);
    }

    // ── T011: US2 — boundary at Todo preserves focus (no-op) ─────────────

    #[test]
    fn test_retreat_at_todo_boundary_preserves_focus() {
        let mut app = make_app_with_tasks(&[(1, "task", Status::Todo)]);
        app.focused_col = Status::Todo.col_index();
        app.focused_card[Status::Todo.col_index()] = 0;

        app.retreat_status();

        assert_eq!(app.focused_col, Status::Todo.col_index());
        assert_eq!(app.focused_card[Status::Todo.col_index()], 0);
    }

    // ── T015: Edge case — source column cursor clamped when emptied ───────

    #[test]
    fn test_source_column_clamped_when_last_task_moves_out() {
        // T2 is inserted first → index 0 in Checking.
        // T1 is in Doing → after advancing, T1 lands at index 1 in Checking.
        let mut app =
            make_app_with_tasks(&[(2, "other", Status::Checking), (1, "task", Status::Doing)]);
        app.focused_col = Status::Doing.col_index();
        app.focused_card[Status::Doing.col_index()] = 0;

        // Advance T1: Doing → Checking
        app.advance_status();

        // Focus follows T1 into Checking at its insertion position (index 1)
        assert_eq!(app.focused_col, Status::Checking.col_index());
        assert_eq!(app.focused_card[Status::Checking.col_index()], 1);
        // Source column (Doing) is now empty; cursor clamped to 0
        assert_eq!(app.focused_card[Status::Doing.col_index()], 0);
    }

    // ── Create task focuses new task ─────────────────────────────────────

    #[test]
    fn test_create_task_focuses_new_task() {
        let mut app = make_app_with_tasks(&[]);
        app.focused_col = 2; // start on a different column
        app.open_create();
        app.input.buffer = "new task".to_string();
        app.confirm_input();

        assert_eq!(app.focused_col, Status::Todo.col_index());
        assert_eq!(app.focused_card[Status::Todo.col_index()], 0);
    }

    #[test]
    fn test_create_task_focuses_new_task_at_end_of_column() {
        let mut app = make_app_with_tasks(&[(1, "existing", Status::Todo)]);
        app.open_create();
        app.input.buffer = "second task".to_string();
        app.confirm_input();

        assert_eq!(app.focused_col, Status::Todo.col_index());
        assert_eq!(app.focused_card[Status::Todo.col_index()], 1);
    }
}
