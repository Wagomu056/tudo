use chrono::Local;

use crate::model::{
    AppError, AppMode, BoardState, DoneEntry, FocusArea, InputState, Memo, Status, Task,
    UrlHitRegion,
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
    /// Whether keyboard focus is in the kanban columns or the memo panel.
    pub focus_area: FocusArea,
    /// Flat index into `board.memos` for the focused memo item.
    pub focused_memo: usize,
    /// Cached items-per-row from the last render frame (default 4).
    pub memo_cols: usize,
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
            focus_area: FocusArea::Kanban,
            focused_memo: 0,
            memo_cols: 4,
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

    // ── Memo helpers ──────────────────────────────────────────────────────

    /// Return a reference to the currently focused memo item, if any.
    pub fn focused_memo_item(&self) -> Option<&Memo> {
        self.board.memos.get(self.focused_memo)
    }

    /// Clamp `focused_memo` so it stays within valid bounds.
    pub fn clamp_memo_focus(&mut self) {
        let len = self.board.memos.len();
        if len == 0 {
            self.focused_memo = 0;
        } else if self.focused_memo >= len {
            self.focused_memo = len - 1;
        }
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

    /// Open the title-input popup for creating a new memo (a key in Memo focus).
    pub fn open_create_memo(&mut self) {
        self.input.clear();
        self.input.is_create = true;
        self.input.is_memo = true;
        self.mode = AppMode::InputTitle;
        self.status_msg = None;
    }

    /// Open the title-edit popup for the focused memo (e key in Memo focus).
    pub fn open_edit_memo_title(&mut self) {
        let title = match self.focused_memo_item() {
            Some(m) => m.title.clone(),
            None => return,
        };
        self.input.clear();
        self.input.buffer = title;
        self.input.is_create = false;
        self.input.is_memo = true;
        self.mode = AppMode::InputTitle;
        self.status_msg = None;
    }

    /// Open the detail-edit popup for the focused memo (E key in Memo focus).
    pub fn open_edit_memo_detail(&mut self) {
        let detail = match self.focused_memo_item() {
            Some(m) => m.detail.clone(),
            None => return,
        };
        self.input.clear();
        self.input.buffer = detail;
        self.input.is_create = false;
        self.input.is_memo = true;
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

        if self.input.is_memo {
            if self.input.is_create {
                let id = self.board.alloc_memo_id();
                let memo = Memo::new(id, value);
                self.board.memos.push(memo);
                self.focused_memo = self.board.memos.len() - 1;
                self.focus_area = FocusArea::Memo;
            } else {
                let idx = self.focused_memo;
                let is_detail = self.mode == AppMode::InputDetail;
                if let Some(memo) = self.board.memos.get_mut(idx) {
                    if is_detail {
                        memo.detail = value;
                    } else {
                        memo.title = value;
                    }
                }
            }
        } else if self.input.is_create {
            let id = self.board.alloc_id();
            let task = Task::new(id, value);
            let insert_pos = self
                .board
                .tasks
                .iter()
                .position(|t| t.status == Status::Todo)
                .unwrap_or(0);
            self.board.tasks.insert(insert_pos, task);
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

    // ── Reorder within column ─────────────────────────────────────────────

    // ── Memo navigation ───────────────────────────────────────────────────

    /// Move down within the kanban column; if already at the bottom (or column
    /// is empty), switch focus to the memo panel.
    pub fn kanban_try_move_down(&mut self) {
        let status = crate::model::ALL_STATUSES[self.focused_col];
        let len = self.tasks_for_column(status).len();
        let at_bottom = len == 0 || self.focused_card[self.focused_col] + 1 >= len;
        if at_bottom {
            self.focus_area = FocusArea::Memo;
            self.clamp_memo_focus();
        } else {
            self.move_down();
        }
    }

    /// Move memo focus one item to the left (no-op at index 0).
    pub fn move_memo_left(&mut self) {
        if self.focused_memo > 0 {
            self.focused_memo -= 1;
        }
    }

    /// Move memo focus one item to the right (no-op at last item).
    pub fn move_memo_right(&mut self) {
        if !self.board.memos.is_empty() && self.focused_memo + 1 < self.board.memos.len() {
            self.focused_memo += 1;
        }
    }

    /// Move memo focus up by one row. If already on the first row, return to
    /// kanban focus.
    pub fn move_memo_up(&mut self) {
        if self.focused_memo < self.memo_cols {
            self.focus_area = FocusArea::Kanban;
        } else {
            self.focused_memo -= self.memo_cols;
        }
    }

    /// Move memo focus down by one row (no-op if already on the last row).
    pub fn move_memo_down(&mut self) {
        let next = self.focused_memo + self.memo_cols;
        if next < self.board.memos.len() {
            self.focused_memo = next;
        }
    }

    // ── Memo CRUD ─────────────────────────────────────────────────────────

    /// Delete the focused memo (no-op if list is empty), then clamp focus.
    pub fn delete_focused_memo(&mut self) {
        if self.board.memos.is_empty() {
            return;
        }
        self.board.memos.remove(self.focused_memo);
        self.clamp_memo_focus();
    }

    // ── Reorder within column ─────────────────────────────────────────────

    /// Move the focused task one position down within its column (J key).
    /// Returns true if a swap occurred, false if already last (no-op).
    pub fn reorder_task_down(&mut self) -> bool {
        let col = self.focused_col;
        let status = crate::model::ALL_STATUSES[col];
        let cursor = self.focused_card[col];

        let col_indices: Vec<usize> = self
            .board
            .tasks
            .iter()
            .enumerate()
            .filter(|(_, t)| t.status == status)
            .map(|(i, _)| i)
            .collect();

        if cursor + 1 >= col_indices.len() {
            return false;
        }

        self.board
            .tasks
            .swap(col_indices[cursor], col_indices[cursor + 1]);
        self.focused_card[col] = cursor + 1;
        true
    }

    /// Move the focused task one position up within its column (K key).
    /// Returns true if a swap occurred, false if already first (no-op).
    pub fn reorder_task_up(&mut self) -> bool {
        let col = self.focused_col;
        let status = crate::model::ALL_STATUSES[col];
        let cursor = self.focused_card[col];

        if cursor == 0 {
            return false;
        }

        let col_indices: Vec<usize> = self
            .board
            .tasks
            .iter()
            .enumerate()
            .filter(|(_, t)| t.status == status)
            .map(|(i, _)| i)
            .collect();

        self.board
            .tasks
            .swap(col_indices[cursor - 1], col_indices[cursor]);
        self.focused_card[col] = cursor - 1;
        true
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
    use crate::model::{BoardState, FocusArea, Memo, Status, Task};

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

    // ── T002/T007: Reorder tests ──────────────────────────────────────────

    // Reorder down: normal swap
    #[test]
    fn test_reorder_task_down_swaps_and_follows_focus() {
        let mut app =
            make_app_with_tasks(&[(1, "first", Status::Todo), (2, "second", Status::Todo)]);
        app.focused_col = Status::Todo.col_index();
        app.focused_card[Status::Todo.col_index()] = 0; // focused on "first"

        let moved = app.reorder_task_down();

        assert!(moved);
        // Focus cursor now points to position 1 (where "first" landed)
        assert_eq!(app.focused_card[Status::Todo.col_index()], 1);
        // "second" is now first in the column
        let col = app.tasks_for_column(Status::Todo);
        assert_eq!(col[0].id, 2);
        assert_eq!(col[1].id, 1);
    }

    // Reorder down: boundary no-op
    #[test]
    fn test_reorder_task_down_at_last_is_noop() {
        let mut app = make_app_with_tasks(&[(1, "only", Status::Todo)]);
        app.focused_col = Status::Todo.col_index();
        app.focused_card[Status::Todo.col_index()] = 0;

        let moved = app.reorder_task_down();

        assert!(!moved);
        assert_eq!(app.focused_card[Status::Todo.col_index()], 0);
    }

    // Reorder only affects the focused column, not others
    #[test]
    fn test_reorder_does_not_affect_other_columns() {
        let mut app = make_app_with_tasks(&[
            (1, "todo-a", Status::Todo),
            (2, "todo-b", Status::Todo),
            (3, "doing-a", Status::Doing),
        ]);
        app.focused_col = Status::Todo.col_index();
        app.focused_card[Status::Todo.col_index()] = 0;
        app.focused_card[Status::Doing.col_index()] = 0;

        app.reorder_task_down();

        // Doing column is unaffected
        let doing = app.tasks_for_column(Status::Doing);
        assert_eq!(doing.len(), 1);
        assert_eq!(doing[0].id, 3);
    }

    // Reorder does not change task status
    #[test]
    fn test_reorder_preserves_task_status() {
        let mut app =
            make_app_with_tasks(&[(1, "first", Status::Doing), (2, "second", Status::Doing)]);
        app.focused_col = Status::Doing.col_index();
        app.focused_card[Status::Doing.col_index()] = 0;

        app.reorder_task_down();

        for task in &app.board.tasks {
            assert_eq!(task.status, Status::Doing);
        }
    }

    // Reorder up: normal swap
    #[test]
    fn test_reorder_task_up_swaps_and_follows_focus() {
        let mut app =
            make_app_with_tasks(&[(1, "first", Status::Todo), (2, "second", Status::Todo)]);
        app.focused_col = Status::Todo.col_index();
        app.focused_card[Status::Todo.col_index()] = 1; // focused on "second"

        let moved = app.reorder_task_up();

        assert!(moved);
        assert_eq!(app.focused_card[Status::Todo.col_index()], 0);
        let col = app.tasks_for_column(Status::Todo);
        assert_eq!(col[0].id, 2);
        assert_eq!(col[1].id, 1);
    }

    // Reorder up: boundary no-op
    #[test]
    fn test_reorder_task_up_at_first_is_noop() {
        let mut app = make_app_with_tasks(&[(1, "only", Status::Todo)]);
        app.focused_col = Status::Todo.col_index();
        app.focused_card[Status::Todo.col_index()] = 0;

        let moved = app.reorder_task_up();

        assert!(!moved);
        assert_eq!(app.focused_card[Status::Todo.col_index()], 0);
    }

    // ── US1 tests: Memo create/edit ──────────────────────────────────────

    // T012: memo_create_adds_to_board
    #[test]
    fn memo_create_adds_to_board() {
        let mut app = make_app_with_tasks(&[]);
        app.open_create_memo();
        app.input.buffer = "Buy milk".to_string();
        app.confirm_input();
        assert_eq!(app.board.memos.len(), 1);
        assert_eq!(app.board.memos[0].title, "Buy milk");
    }

    // T013: memo_create_focuses_new_memo
    #[test]
    fn memo_create_focuses_new_memo() {
        let mut app = make_app_with_tasks(&[]);
        app.open_create_memo();
        app.input.buffer = "Buy milk".to_string();
        app.confirm_input();
        assert_eq!(app.focus_area, FocusArea::Memo);
        assert_eq!(app.focused_memo, 0);
    }

    // T014: memo_create_empty_title_rejected
    #[test]
    fn memo_create_empty_title_rejected() {
        let mut app = make_app_with_tasks(&[]);
        app.open_create_memo();
        app.input.buffer = "  ".to_string();
        app.confirm_input();
        assert!(app.board.memos.is_empty());
        assert!(app.status_msg.is_some());
    }

    // T015: memo_edit_title
    #[test]
    fn memo_edit_title() {
        let mut app = make_app_with_tasks(&[]);
        app.board.memos.push(Memo::new(1, "Original".to_string()));
        app.board.next_memo_id = 2;
        app.focus_area = FocusArea::Memo;
        app.focused_memo = 0;
        app.open_edit_memo_title();
        app.input.buffer = "Updated".to_string();
        app.confirm_input();
        assert_eq!(app.board.memos[0].title, "Updated");
    }

    // T016: memo_edit_detail
    #[test]
    fn memo_edit_detail() {
        let mut app = make_app_with_tasks(&[]);
        app.board.memos.push(Memo::new(1, "A memo".to_string()));
        app.board.next_memo_id = 2;
        app.focus_area = FocusArea::Memo;
        app.focused_memo = 0;
        app.open_edit_memo_detail();
        app.input.buffer = "Some detail".to_string();
        app.confirm_input();
        assert_eq!(app.board.memos[0].detail, "Some detail");
    }

    // ── US2 tests: Navigation ─────────────────────────────────────────────

    fn make_app_with_memos(titles: &[&str]) -> AppState {
        let mut app = AppState::new(BoardState::default());
        for (i, &title) in titles.iter().enumerate() {
            app.board
                .memos
                .push(Memo::new(i as u64 + 1, title.to_string()));
        }
        app.board.next_memo_id = titles.len() as u64 + 1;
        app
    }

    // T022: memo_enter_from_kanban_bottom
    #[test]
    fn memo_enter_from_kanban_bottom() {
        let mut app = make_app_with_tasks(&[(1, "task", Status::Todo)]);
        app.focused_col = 0;
        app.focused_card[0] = 0;
        app.kanban_try_move_down();
        assert_eq!(app.focus_area, FocusArea::Memo);
        assert_eq!(app.focused_memo, 0);
    }

    // T023: memo_enter_from_empty_kanban_column
    #[test]
    fn memo_enter_from_empty_kanban_column() {
        let mut app = make_app_with_tasks(&[]);
        app.focused_col = 0;
        app.kanban_try_move_down();
        assert_eq!(app.focus_area, FocusArea::Memo);
    }

    // T024: memo_exit_to_kanban_from_first_row
    #[test]
    fn memo_exit_to_kanban_from_first_row() {
        let mut app = make_app_with_memos(&["A"]);
        app.focus_area = FocusArea::Memo;
        app.focused_memo = 0;
        app.memo_cols = 4;
        app.move_memo_up();
        assert_eq!(app.focus_area, FocusArea::Kanban);
    }

    // T025: memo_move_right_advances_index and boundary
    #[test]
    fn memo_move_right_advances_index() {
        let mut app = make_app_with_memos(&["A", "B", "C"]);
        app.focus_area = FocusArea::Memo;
        app.focused_memo = 0;
        app.move_memo_right();
        assert_eq!(app.focused_memo, 1);
    }

    #[test]
    fn memo_move_right_boundary_noop() {
        let mut app = make_app_with_memos(&["A", "B"]);
        app.focus_area = FocusArea::Memo;
        app.focused_memo = 1;
        app.move_memo_right();
        assert_eq!(app.focused_memo, 1);
    }

    // T026: memo_move_left_decrements_index and boundary
    #[test]
    fn memo_move_left_decrements_index() {
        let mut app = make_app_with_memos(&["A", "B"]);
        app.focus_area = FocusArea::Memo;
        app.focused_memo = 1;
        app.move_memo_left();
        assert_eq!(app.focused_memo, 0);
    }

    #[test]
    fn memo_move_left_boundary_noop() {
        let mut app = make_app_with_memos(&["A", "B"]);
        app.focus_area = FocusArea::Memo;
        app.focused_memo = 0;
        app.move_memo_left();
        assert_eq!(app.focused_memo, 0);
    }

    // T027: memo_move_down_advances_by_memo_cols and last_row_noop
    #[test]
    fn memo_move_down_advances_by_memo_cols() {
        let mut app = make_app_with_memos(&["A", "B", "C", "D"]);
        app.focus_area = FocusArea::Memo;
        app.focused_memo = 0;
        app.memo_cols = 3;
        app.move_memo_down();
        assert_eq!(app.focused_memo, 3);
    }

    #[test]
    fn memo_move_down_last_row_noop() {
        let mut app = make_app_with_memos(&["A", "B", "C"]);
        app.focus_area = FocusArea::Memo;
        app.focused_memo = 0;
        app.memo_cols = 3;
        app.move_memo_down(); // index 0 + 3 = 3, but len=3, so noop
        assert_eq!(app.focused_memo, 0);
    }

    // T028: memo_move_up_row_subtracts_memo_cols
    #[test]
    fn memo_move_up_row_subtracts_memo_cols() {
        let mut app = make_app_with_memos(&["A", "B", "C", "D"]);
        app.focus_area = FocusArea::Memo;
        app.focused_memo = 3;
        app.memo_cols = 3;
        app.move_memo_up();
        assert_eq!(app.focused_memo, 0);
    }

    // ── US3 tests: Delete ─────────────────────────────────────────────────

    // T033: memo_delete_removes_correct_item
    #[test]
    fn memo_delete_removes_correct_item() {
        let mut app = make_app_with_memos(&["A", "B"]);
        app.focus_area = FocusArea::Memo;
        app.focused_memo = 0;
        app.delete_focused_memo();
        assert_eq!(app.board.memos.len(), 1);
        assert_eq!(app.board.memos[0].id, 2);
    }

    // T034: memo_delete_clamps_focus
    #[test]
    fn memo_delete_clamps_focus() {
        let mut app = make_app_with_memos(&["A", "B"]);
        app.focus_area = FocusArea::Memo;
        app.focused_memo = 1;
        app.delete_focused_memo();
        assert_eq!(app.focused_memo, 0);
    }

    // T035: memo_delete_last_item_leaves_empty
    #[test]
    fn memo_delete_last_item_leaves_empty() {
        let mut app = make_app_with_memos(&["A"]);
        app.focus_area = FocusArea::Memo;
        app.focused_memo = 0;
        app.delete_focused_memo();
        assert!(app.board.memos.is_empty());
        assert_eq!(app.focused_memo, 0);
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
    fn test_create_task_focuses_new_task_at_top_of_column() {
        let mut app = make_app_with_tasks(&[(1, "existing", Status::Todo)]);
        app.open_create();
        app.input.buffer = "new task".to_string();
        app.confirm_input();

        // 新規タスクはカラムの先頭 (index 0) に配置される
        assert_eq!(app.focused_col, Status::Todo.col_index());
        assert_eq!(app.focused_card[Status::Todo.col_index()], 0);
    }

    #[test]
    fn test_create_task_placed_at_top_of_column_order() {
        let mut app = make_app_with_tasks(&[(1, "existing", Status::Todo)]);
        app.open_create();
        app.input.buffer = "new task".to_string();
        app.confirm_input();

        // カラム内の順序を確認: 新規タスクが先頭
        let col = app.tasks_for_column(Status::Todo);
        assert_eq!(col[0].title, "new task");
        assert_eq!(col[1].title, "existing");
    }

    #[test]
    fn test_create_task_at_top_with_mixed_statuses() {
        // Doing タスクが存在しても Todo カラムの先頭に挿入される
        let mut app = make_app_with_tasks(&[
            (1, "existing-todo", Status::Todo),
            (2, "doing-task", Status::Doing),
        ]);
        app.open_create();
        app.input.buffer = "newest".to_string();
        app.confirm_input();

        let col = app.tasks_for_column(Status::Todo);
        assert_eq!(col[0].title, "newest");
        assert_eq!(col[1].title, "existing-todo");
        // Doing カラムは影響なし
        let doing_col = app.tasks_for_column(Status::Doing);
        assert_eq!(doing_col.len(), 1);
        assert_eq!(doing_col[0].id, 2);
    }
}
