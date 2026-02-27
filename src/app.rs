use chrono::Local;

use crate::model::{AppError, AppMode, BoardState, DoneEntry, InputState, Status, Task};

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
