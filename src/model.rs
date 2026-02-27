use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};
use std::fmt;

// ── Status ───────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Status {
    Todo,
    Doing,
    Checking,
    Done,
}

impl Status {
    /// Returns the next status in the workflow, or None if already Done.
    pub fn next(self) -> Option<Status> {
        match self {
            Status::Todo => Some(Status::Doing),
            Status::Doing => Some(Status::Checking),
            Status::Checking => Some(Status::Done),
            Status::Done => None,
        }
    }

    /// Returns the previous status in the workflow, or None if already Todo.
    pub fn prev(self) -> Option<Status> {
        match self {
            Status::Todo => None,
            Status::Doing => Some(Status::Todo),
            Status::Checking => Some(Status::Doing),
            Status::Done => Some(Status::Checking),
        }
    }

    /// Index of the status (0 = Todo, 1 = Doing, 2 = Checking, 3 = Done).
    pub fn col_index(self) -> usize {
        match self {
            Status::Todo => 0,
            Status::Doing => 1,
            Status::Checking => 2,
            Status::Done => 3,
        }
    }

    /// Display label for the column header.
    pub fn label(self) -> &'static str {
        match self {
            Status::Todo => "Todo",
            Status::Doing => "Doing",
            Status::Checking => "Checking",
            Status::Done => "Done",
        }
    }
}

pub const ALL_STATUSES: [Status; 4] = [Status::Todo, Status::Doing, Status::Checking, Status::Done];

// ── Task ─────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: u64,
    pub title: String,
    pub detail: String,
    pub status: Status,
    pub created_at: DateTime<Local>,
    pub done_at: Option<DateTime<Local>>,
}

/// Error returned when task construction fails validation.
#[derive(Debug)]
pub struct TaskError(pub String);

impl fmt::Display for TaskError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "invalid task: {}", self.0)
    }
}

impl Task {
    /// Create a new Todo task with the given id and title (no validation).
    /// Used internally when constructing from trusted sources (e.g. deserialized data).
    pub fn new(id: u64, title: String) -> Self {
        Task {
            id,
            title,
            detail: String::new(),
            status: Status::Todo,
            created_at: Local::now(),
            done_at: None,
        }
    }

    /// Create a new Todo task, validating that the title is non-empty after trimming.
    pub fn try_new(id: u64, title: String) -> Result<Self, TaskError> {
        let trimmed = title.trim().to_string();
        if trimmed.is_empty() {
            return Err(TaskError("title must not be empty".to_string()));
        }
        Ok(Task {
            id,
            title: trimmed,
            detail: String::new(),
            status: Status::Todo,
            created_at: Local::now(),
            done_at: None,
        })
    }
}

// ── BoardState ───────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoardState {
    pub version: u32,
    pub next_id: u64,
    pub tasks: Vec<Task>,
    pub saved_at: DateTime<Local>,
}

impl Default for BoardState {
    fn default() -> Self {
        BoardState {
            version: 1,
            next_id: 1,
            tasks: Vec::new(),
            saved_at: Local::now(),
        }
    }
}

impl BoardState {
    /// Construct a board with the given tasks and next_id.
    /// Used in tests for building specific states.
    pub fn with_tasks(tasks: Vec<Task>, next_id: u64) -> Self {
        BoardState {
            version: 1,
            next_id,
            tasks,
            saved_at: Local::now(),
        }
    }

    /// Allocate a new unique task ID and advance the counter.
    pub fn alloc_id(&mut self) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        id
    }
}

// ── DoneEntry ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DoneEntry {
    pub title: String,
    pub detail: String,
    pub completed_at: DateTime<Local>,
}

impl DoneEntry {
    pub fn from_task(task: &Task) -> Self {
        DoneEntry {
            title: task.title.clone(),
            detail: task.detail.clone(),
            completed_at: task.done_at.unwrap_or_else(Local::now),
        }
    }
}

// ── AppMode ──────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AppMode {
    Normal,
    InputTitle,
    InputDetail,
}

// ── InputState ───────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Default)]
pub struct InputState {
    pub buffer: String,
    pub is_create: bool,
}

impl InputState {
    /// Append a character to the buffer.
    pub fn push_char(&mut self, c: char) {
        self.buffer.push(c);
    }

    /// Remove the last character from the buffer (no-op if empty).
    pub fn pop_char(&mut self) {
        self.buffer.pop();
    }

    /// Return the current buffer contents.
    pub fn value(&self) -> &str {
        &self.buffer
    }

    /// Reset the buffer to empty.
    pub fn clear(&mut self) {
        self.buffer.clear();
    }
}

// ── AppError ─────────────────────────────────────────────────────────────────

#[derive(Debug)]
pub enum AppError {
    Io(std::io::Error),
    Json(serde_json::Error),
    VersionMismatch(u32),
    Other(String),
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::Io(e) => write!(f, "I/O error: {}", e),
            AppError::Json(e) => write!(f, "JSON error: {}", e),
            AppError::VersionMismatch(v) => write!(f, "unsupported data version: {}", v),
            AppError::Other(s) => write!(f, "{}", s),
        }
    }
}

impl From<std::io::Error> for AppError {
    fn from(e: std::io::Error) -> Self {
        AppError::Io(e)
    }
}

impl From<serde_json::Error> for AppError {
    fn from(e: serde_json::Error) -> Self {
        AppError::Json(e)
    }
}
