use std::fs;
use std::path::PathBuf;

use tudo::model::{BoardState, DoneEntry, Status, Task};
use tudo::storage;

// ── Helper ────────────────────────────────────────────────────────────────────

fn temp_path(name: &str) -> PathBuf {
    let dir = tempfile::tempdir().expect("create temp dir");
    let path = dir.path().join(name);
    // Keep the dir alive by leaking it (acceptable in tests — OS cleans up)
    std::mem::forget(dir);
    path
}

// ── save_board / load_board round-trip (T033) ─────────────────────────────────

#[test]
fn save_and_load_board_round_trip() {
    let path = temp_path("current.log");
    let path_str = path.to_str().unwrap();

    let mut board = BoardState::default();
    board.tasks.push(Task::new(1, "Alpha".to_string()));
    let mut task2 = Task::new(2, "Beta".to_string());
    task2.status = Status::Doing;
    task2.detail = "Some detail".to_string();
    board.tasks.push(task2);
    board.next_id = 3;

    storage::save_board_to(&mut board, path_str).expect("save board");

    let loaded = storage::load_board_from(path_str).expect("load board");
    assert_eq!(loaded.version, 1);
    assert_eq!(loaded.tasks.len(), 2);
    assert_eq!(loaded.tasks[0].title, "Alpha");
    assert_eq!(loaded.tasks[1].title, "Beta");
    assert_eq!(loaded.tasks[1].status, Status::Doing);
    assert_eq!(loaded.tasks[1].detail, "Some detail");
    assert_eq!(loaded.next_id, 3);
}

#[test]
fn load_board_returns_default_when_file_missing() {
    let path = temp_path("nonexistent_current.log");
    let board = storage::load_board_from(path.to_str().unwrap()).expect("load missing board");
    assert_eq!(board.version, 1);
    assert!(board.tasks.is_empty());
}

#[test]
fn load_board_errors_on_corrupt_json() {
    let path = temp_path("corrupt.log");
    fs::write(&path, "not valid json").expect("write corrupt file");
    let result = storage::load_board_from(path.to_str().unwrap());
    assert!(result.is_err());
}

// ── append_done_entry (T034) ──────────────────────────────────────────────────

#[test]
fn append_done_entry_writes_two_json_lines() {
    let log_path = temp_path("20260228.log");

    let entry1 = DoneEntry {
        title: "First".to_string(),
        detail: "".to_string(),
        completed_at: chrono::Local::now(),
    };
    let entry2 = DoneEntry {
        title: "Second".to_string(),
        detail: "detail text".to_string(),
        completed_at: chrono::Local::now(),
    };

    storage::append_done_entry_to(&entry1, &log_path).expect("append entry1");
    storage::append_done_entry_to(&entry2, &log_path).expect("append entry2");

    let contents = fs::read_to_string(&log_path).expect("read log file");
    let lines: Vec<&str> = contents.lines().collect();
    assert_eq!(lines.len(), 2);

    let v1: serde_json::Value = serde_json::from_str(lines[0]).expect("parse line 1");
    let v2: serde_json::Value = serde_json::from_str(lines[1]).expect("parse line 2");
    assert_eq!(v1["title"], "First");
    assert_eq!(v2["title"], "Second");
    assert_eq!(v2["detail"], "detail text");
}

// ── Daily Done filter (T035) ──────────────────────────────────────────────────

#[test]
fn daily_filter_removes_done_tasks_from_previous_days() {
    use chrono::{Duration, Local};
    use tudo::app::AppState;

    let mut board = BoardState::default();

    // A Done task from yesterday
    let mut old_task = Task::new(1, "Old done".to_string());
    old_task.status = Status::Done;
    old_task.done_at = Some(Local::now() - Duration::days(1));
    board.tasks.push(old_task);

    // A Done task from today
    let mut today_task = Task::new(2, "Today done".to_string());
    today_task.status = Status::Done;
    today_task.done_at = Some(Local::now());
    board.tasks.push(today_task);

    // A Todo task (should always survive)
    board.tasks.push(Task::new(3, "Active".to_string()));
    board.next_id = 4;

    let mut app = AppState::new(board);
    app.apply_daily_filter();

    let done = app.tasks_for_column(Status::Done);
    let todo = app.tasks_for_column(Status::Todo);

    assert_eq!(done.len(), 1, "only today's Done task should remain");
    assert_eq!(done[0].title, "Today done");
    assert_eq!(todo.len(), 1, "Todo task should be unaffected");
}
