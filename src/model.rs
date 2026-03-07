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

// ── Memo ─────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Memo {
    pub id: u64,
    pub title: String,
    pub detail: String,
}

impl Memo {
    pub fn new(id: u64, title: String) -> Self {
        Memo {
            id,
            title,
            detail: String::new(),
        }
    }
}

// ── FocusArea ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FocusArea {
    Kanban,
    Memo,
}

// ── StatusTaskMap ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Default)]
pub struct StatusTaskMap {
    pub todo: Vec<Task>,
    pub doing: Vec<Task>,
    pub checking: Vec<Task>,
    pub done: Vec<Task>,
}

impl StatusTaskMap {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn tasks_for(&self, status: Status) -> &Vec<Task> {
        match status {
            Status::Todo => &self.todo,
            Status::Doing => &self.doing,
            Status::Checking => &self.checking,
            Status::Done => &self.done,
        }
    }

    pub fn tasks_for_mut(&mut self, status: Status) -> &mut Vec<Task> {
        match status {
            Status::Todo => &mut self.todo,
            Status::Doing => &mut self.doing,
            Status::Checking => &mut self.checking,
            Status::Done => &mut self.done,
        }
    }

    pub fn insert_at_top(&mut self, status: Status, task: Task) {
        self.tasks_for_mut(status).insert(0, task);
    }

    pub fn remove_by_id(&mut self, status: Status, id: u64) -> Option<Task> {
        let list = self.tasks_for_mut(status);
        list.iter()
            .position(|t| t.id == id)
            .map(|pos| list.remove(pos))
    }

    pub fn all_tasks(&self) -> impl Iterator<Item = &Task> {
        self.todo
            .iter()
            .chain(self.doing.iter())
            .chain(self.checking.iter())
            .chain(self.done.iter())
    }

    pub fn from_flat(tasks: Vec<Task>) -> Self {
        let mut map = StatusTaskMap::new();
        for task in tasks {
            map.tasks_for_mut(task.status).push(task);
        }
        map
    }

    pub fn len(&self) -> usize {
        self.todo.len() + self.doing.len() + self.checking.len() + self.done.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

fn serialize_status_task_map<S>(map: &StatusTaskMap, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    let flat: Vec<&Task> = map.all_tasks().collect();
    flat.serialize(serializer)
}

fn deserialize_status_task_map<'de, D>(deserializer: D) -> Result<StatusTaskMap, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let flat = Vec::<Task>::deserialize(deserializer)?;
    Ok(StatusTaskMap::from_flat(flat))
}

// ── BoardState ───────────────────────────────────────────────────────────────

fn default_next_memo_id() -> u64 {
    1
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoardState {
    pub version: u32,
    pub next_id: u64,
    #[serde(
        serialize_with = "serialize_status_task_map",
        deserialize_with = "deserialize_status_task_map"
    )]
    pub tasks: StatusTaskMap,
    pub saved_at: DateTime<Local>,
    #[serde(default)]
    pub memos: Vec<Memo>,
    #[serde(default = "default_next_memo_id")]
    pub next_memo_id: u64,
}

impl Default for BoardState {
    fn default() -> Self {
        BoardState {
            version: 1,
            next_id: 1,
            tasks: StatusTaskMap::new(),
            saved_at: Local::now(),
            memos: Vec::new(),
            next_memo_id: 1,
        }
    }
}

impl BoardState {
    /// Construct a board with the given tasks (flat Vec) and next_id.
    /// Used in tests for building specific states.
    pub fn with_tasks(tasks: Vec<Task>, next_id: u64) -> Self {
        BoardState {
            version: 1,
            next_id,
            tasks: StatusTaskMap::from_flat(tasks),
            saved_at: Local::now(),
            memos: Vec::new(),
            next_memo_id: 1,
        }
    }

    /// Allocate a new unique task ID and advance the counter.
    pub fn alloc_id(&mut self) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        id
    }

    /// Allocate a new unique memo ID and advance the counter.
    pub fn alloc_memo_id(&mut self) -> u64 {
        let id = self.next_memo_id;
        self.next_memo_id += 1;
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

// ── UrlHitRegion ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub struct UrlHitRegion {
    pub row: u16,
    pub col_start: u16,
    pub col_end: u16,
    pub url: String,
}

// ── TaskHitRegion ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub struct TaskHitRegion {
    pub row_start: u16,
    pub row_end: u16,
    pub col_start: u16,
    pub col_end: u16,
    pub column: usize,
    pub card_index: usize,
}

// ── MemoHitRegion ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub struct MemoHitRegion {
    pub row: u16,
    pub col_start: u16,
    pub col_end: u16,
    pub memo_index: usize,
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
    /// Cursor position as a byte offset into `buffer`.
    /// Always on a valid UTF-8 char boundary, within 0..=buffer.len().
    pub cursor: usize,
    pub is_create: bool,
    pub is_memo: bool,
}

impl InputState {
    /// Move cursor one character to the left (no-op at beginning).
    pub fn move_left(&mut self) {
        if self.cursor == 0 {
            return;
        }
        if let Some((idx, _)) = self.buffer[..self.cursor].char_indices().last() {
            self.cursor = idx;
        }
    }

    /// Move cursor one character to the right (no-op at end).
    pub fn move_right(&mut self) {
        if self.cursor >= self.buffer.len() {
            return;
        }
        if let Some(ch) = self.buffer[self.cursor..].chars().next() {
            self.cursor += ch.len_utf8();
        }
    }

    /// Move cursor to the beginning of the buffer.
    pub fn move_home(&mut self) {
        self.cursor = 0;
    }

    /// Move cursor to the end of the buffer.
    pub fn move_end(&mut self) {
        self.cursor = self.buffer.len();
    }

    /// Insert a character at the cursor position and advance the cursor.
    pub fn insert_char(&mut self, c: char) {
        self.buffer.insert(self.cursor, c);
        self.cursor += c.len_utf8();
    }

    /// Delete the character immediately before the cursor (backspace). No-op at start.
    pub fn delete_char_back(&mut self) {
        if self.cursor == 0 {
            return;
        }
        if let Some((idx, _)) = self.buffer[..self.cursor].char_indices().last() {
            self.buffer.remove(idx);
            self.cursor = idx;
        }
    }

    /// Set the buffer to `s` and move the cursor to the end.
    pub fn set_buffer(&mut self, s: String) {
        self.cursor = s.len();
        self.buffer = s;
    }

    /// Append a character to the buffer (does not update cursor).
    pub fn push_char(&mut self, c: char) {
        self.buffer.push(c);
    }

    /// Remove the last character from the buffer (does not update cursor; no-op if empty).
    pub fn pop_char(&mut self) {
        self.buffer.pop();
    }

    /// Return the current buffer contents.
    pub fn value(&self) -> &str {
        &self.buffer
    }

    /// Reset the buffer, cursor, and flags to defaults.
    pub fn clear(&mut self) {
        self.buffer.clear();
        self.cursor = 0;
        self.is_memo = false;
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

// ── InputState tests ──────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // ── T002: StatusTaskMap construction and insert ───────────────────────

    #[test]
    fn status_task_map_new_returns_empty_lists() {
        let map = StatusTaskMap::new();
        assert!(map.tasks_for(Status::Todo).is_empty());
        assert!(map.tasks_for(Status::Doing).is_empty());
        assert!(map.tasks_for(Status::Checking).is_empty());
        assert!(map.tasks_for(Status::Done).is_empty());
    }

    #[test]
    fn status_task_map_insert_at_top_places_at_index_zero() {
        let mut map = StatusTaskMap::new();
        let t1 = Task::new(1, "first".to_string());
        let t2 = Task::new(2, "second".to_string());
        map.insert_at_top(Status::Todo, t1);
        map.insert_at_top(Status::Todo, t2);
        let tasks = map.tasks_for(Status::Todo);
        assert_eq!(tasks.len(), 2);
        assert_eq!(tasks[0].id, 2); // most recent at top
        assert_eq!(tasks[1].id, 1); // original pushed down
    }

    // ── T003: StatusTaskMap remove_by_id ─────────────────────────────────

    #[test]
    fn status_task_map_remove_by_id_removes_correct_task() {
        let mut map = StatusTaskMap::new();
        map.insert_at_top(Status::Todo, Task::new(1, "a".to_string()));
        map.insert_at_top(Status::Todo, Task::new(2, "b".to_string()));
        let removed = map.remove_by_id(Status::Todo, 1);
        assert!(removed.is_some());
        assert_eq!(removed.unwrap().id, 1);
        assert_eq!(map.tasks_for(Status::Todo).len(), 1);
        assert_eq!(map.tasks_for(Status::Todo)[0].id, 2);
    }

    #[test]
    fn status_task_map_remove_by_id_noop_when_not_found() {
        let mut map = StatusTaskMap::new();
        map.insert_at_top(Status::Todo, Task::new(1, "a".to_string()));
        let removed = map.remove_by_id(Status::Todo, 99);
        assert!(removed.is_none());
        assert_eq!(map.tasks_for(Status::Todo).len(), 1);
    }

    // ── T004: StatusTaskMap from_flat ─────────────────────────────────────

    #[test]
    fn status_task_map_from_flat_distributes_by_status() {
        let mut t1 = Task::new(1, "todo1".to_string());
        t1.status = Status::Todo;
        let mut t2 = Task::new(2, "doing1".to_string());
        t2.status = Status::Doing;
        let mut t3 = Task::new(3, "todo2".to_string());
        t3.status = Status::Todo;
        let map = StatusTaskMap::from_flat(vec![t1, t2, t3]);
        assert_eq!(map.tasks_for(Status::Todo).len(), 2);
        assert_eq!(map.tasks_for(Status::Doing).len(), 1);
        assert_eq!(map.tasks_for(Status::Checking).len(), 0);
        // preserves relative order within each status
        assert_eq!(map.tasks_for(Status::Todo)[0].id, 1);
        assert_eq!(map.tasks_for(Status::Todo)[1].id, 3);
    }

    // T001: cursor invariants

    #[test]
    fn cursor_starts_at_zero_on_default() {
        let state = InputState::default();
        assert_eq!(state.cursor, 0);
        assert!(state.cursor <= state.buffer.len());
        assert!(state.buffer.is_char_boundary(state.cursor));
    }

    #[test]
    fn cursor_stays_on_char_boundary_after_insert() {
        let mut state = InputState::default();
        state.insert_char('あ');
        assert!(state.buffer.is_char_boundary(state.cursor));
        assert!(state.cursor <= state.buffer.len());
    }

    #[test]
    fn cursor_stays_on_char_boundary_after_delete() {
        let mut state = InputState::default();
        state.insert_char('あ');
        state.delete_char_back();
        assert!(state.buffer.is_char_boundary(state.cursor));
        assert!(state.cursor <= state.buffer.len());
    }

    // T002/T003: move_left, move_right, move_home, move_end — ASCII

    #[test]
    fn move_left_at_start_is_noop() {
        let mut state = InputState::default();
        state.set_buffer("hello".to_string());
        state.move_home();
        state.move_left();
        assert_eq!(state.cursor, 0);
    }

    #[test]
    fn move_right_at_end_is_noop() {
        let mut state = InputState::default();
        state.set_buffer("hello".to_string());
        state.move_right();
        assert_eq!(state.cursor, 5);
    }

    #[test]
    fn move_left_ascii() {
        let mut state = InputState::default();
        state.set_buffer("hello".to_string()); // cursor at 5
        state.move_left();
        assert_eq!(state.cursor, 4);
        state.move_left();
        assert_eq!(state.cursor, 3);
    }

    #[test]
    fn move_right_ascii() {
        let mut state = InputState::default();
        state.set_buffer("hello".to_string());
        state.move_home();
        state.move_right();
        assert_eq!(state.cursor, 1);
        state.move_right();
        assert_eq!(state.cursor, 2);
    }

    #[test]
    fn move_home_goes_to_zero() {
        let mut state = InputState::default();
        state.set_buffer("hello".to_string());
        state.move_home();
        assert_eq!(state.cursor, 0);
    }

    #[test]
    fn move_end_goes_to_buffer_len() {
        let mut state = InputState::default();
        state.set_buffer("hello".to_string());
        state.move_home();
        state.move_end();
        assert_eq!(state.cursor, 5);
    }

    // T002/T003: CJK navigation

    #[test]
    fn move_left_cjk() {
        let mut state = InputState::default();
        state.set_buffer("あい".to_string()); // cursor at 6 (3+3 bytes)
        state.move_left();
        assert_eq!(state.cursor, 3);
        state.move_left();
        assert_eq!(state.cursor, 0);
    }

    #[test]
    fn move_right_cjk() {
        let mut state = InputState::default();
        state.set_buffer("あい".to_string());
        state.move_home();
        state.move_right();
        assert_eq!(state.cursor, 3); // after 'あ'
        state.move_right();
        assert_eq!(state.cursor, 6); // after 'い'
    }

    // T002/T003: mixed ASCII+CJK navigation

    #[test]
    fn move_through_mixed_text() {
        let mut state = InputState::default();
        // 'a'=1 byte, 'あ'=3 bytes, 'b'=1 byte → total 5 bytes
        state.set_buffer("aあb".to_string());
        state.move_home();
        state.move_right();
        assert_eq!(state.cursor, 1); // after 'a'
        state.move_right();
        assert_eq!(state.cursor, 4); // after 'あ'
        state.move_right();
        assert_eq!(state.cursor, 5); // after 'b'
        state.move_left();
        assert_eq!(state.cursor, 4);
        state.move_left();
        assert_eq!(state.cursor, 1);
    }

    // T004/T005: insert_char

    #[test]
    fn insert_char_at_end() {
        let mut state = InputState::default();
        state.insert_char('h');
        state.insert_char('i');
        assert_eq!(state.buffer, "hi");
        assert_eq!(state.cursor, 2);
    }

    #[test]
    fn insert_char_at_beginning() {
        let mut state = InputState::default();
        state.set_buffer("ello".to_string());
        state.move_home();
        state.insert_char('h');
        assert_eq!(state.buffer, "hello");
        assert_eq!(state.cursor, 1);
    }

    #[test]
    fn insert_char_in_middle() {
        let mut state = InputState::default();
        state.set_buffer("hllo".to_string());
        state.move_home();
        state.move_right(); // after 'h'
        state.insert_char('e');
        assert_eq!(state.buffer, "hello");
        assert_eq!(state.cursor, 2);
    }

    #[test]
    fn insert_cjk_char_advances_cursor_by_utf8_len() {
        let mut state = InputState::default();
        state.insert_char('あ');
        assert_eq!(state.buffer, "あ");
        assert_eq!(state.cursor, 3); // 'あ' is 3 bytes in UTF-8
    }

    // T004/T005: delete_char_back

    #[test]
    fn delete_char_back_at_end() {
        let mut state = InputState::default();
        state.set_buffer("hello".to_string());
        state.delete_char_back();
        assert_eq!(state.buffer, "hell");
        assert_eq!(state.cursor, 4);
    }

    #[test]
    fn delete_char_back_at_beginning_is_noop() {
        let mut state = InputState::default();
        state.set_buffer("hello".to_string());
        state.move_home();
        state.delete_char_back();
        assert_eq!(state.buffer, "hello");
        assert_eq!(state.cursor, 0);
    }

    #[test]
    fn delete_char_back_in_middle() {
        let mut state = InputState::default();
        state.set_buffer("hello".to_string());
        state.move_home();
        state.move_right(); // cursor at 1
        state.move_right(); // cursor at 2
        state.delete_char_back(); // delete 'e'
        assert_eq!(state.buffer, "hllo");
        assert_eq!(state.cursor, 1);
    }

    #[test]
    fn delete_cjk_char_back() {
        let mut state = InputState::default();
        state.set_buffer("あい".to_string());
        state.delete_char_back(); // delete 'い'
        assert_eq!(state.buffer, "あ");
        assert_eq!(state.cursor, 3);
    }

    // T006: set_buffer and clear

    #[test]
    fn set_buffer_moves_cursor_to_end() {
        let mut state = InputState::default();
        state.set_buffer("hello".to_string());
        assert_eq!(state.cursor, 5);
        assert_eq!(state.buffer, "hello");
    }

    #[test]
    fn set_buffer_cjk_cursor_at_end() {
        let mut state = InputState::default();
        state.set_buffer("あい".to_string());
        assert_eq!(state.cursor, 6); // 3+3 bytes
    }

    #[test]
    fn clear_resets_cursor_and_buffer() {
        let mut state = InputState::default();
        state.set_buffer("hello".to_string());
        state.clear();
        assert_eq!(state.cursor, 0);
        assert_eq!(state.buffer, "");
        assert!(!state.is_memo);
    }
}
