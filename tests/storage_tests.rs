use std::fs;
use std::path::{Path, PathBuf};

use chrono::Local;
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

// ── resolve_data_dir (T002) ───────────────────────────────────────────────────

#[test]
fn resolve_data_dir_returns_ok() {
    let result = storage::resolve_data_dir();
    assert!(
        result.is_ok(),
        "resolve_data_dir() should return Ok, got: {:?}",
        result
    );
}

#[test]
fn resolve_data_dir_creates_directory() {
    let path = storage::resolve_data_dir().expect("resolve_data_dir failed");
    assert!(
        path.is_dir(),
        "data directory should exist on disk at {:?}",
        path
    );
}

#[test]
fn resolve_data_dir_path_ends_with_tudo() {
    let path = storage::resolve_data_dir().expect("resolve_data_dir failed");
    let last = path.file_name().expect("path should have a last component");
    assert_eq!(
        last, "tudo",
        "last path component should be 'tudo', got {:?}",
        last
    );
}

// ── US1: No CWD pollution (T004) ─────────────────────────────────────────────

#[test]
fn save_board_writes_to_data_dir_not_cwd() {
    storage::save_board(&mut BoardState::default()).expect("save_board failed");
    assert!(
        !Path::new("current.log").exists(),
        "current.log must not exist in the current working directory"
    );
    let data_dir = storage::resolve_data_dir().expect("resolve_data_dir failed");
    assert!(
        data_dir.join("current.log").exists(),
        "current.log must exist in the data directory {:?}",
        data_dir
    );
}

#[test]
fn load_board_reads_from_data_dir() {
    // Write via the high-level API so the file is in the data dir (not CWD)
    storage::save_board(&mut BoardState::default()).expect("save_board failed");
    // load_board must find the file written above — version is always 1 for any valid board
    let loaded = storage::load_board().expect("load_board failed");
    assert_eq!(loaded.version, 1);
}

#[test]
fn append_done_entry_writes_to_data_dir_not_cwd() {
    let entry = DoneEntry {
        title: "Test Task".to_string(),
        detail: "".to_string(),
        completed_at: Local::now(),
    };
    storage::append_done_entry(&entry).expect("append_done_entry failed");

    let today = Local::now().format("%Y%m%d").to_string();
    let log_name = format!("{}.log", today);
    assert!(
        !Path::new(&log_name).exists(),
        "{} must not exist in the current working directory",
        log_name
    );
    let data_dir = storage::resolve_data_dir().expect("resolve_data_dir failed");
    assert!(
        data_dir.join(&log_name).exists(),
        "{} must exist in the data directory {:?}",
        log_name,
        data_dir
    );
}

// ── US2: Consistent location (T008) ──────────────────────────────────────────

#[test]
fn save_and_load_round_trip_via_data_dir() {
    let mut board = BoardState::default();
    board.tasks.push(Task::new(1, "RoundTrip Task".to_string()));
    board.next_id = 2;

    storage::save_board(&mut board).expect("save_board failed");
    let loaded = storage::load_board().expect("load_board failed");

    assert_eq!(loaded.next_id, 2);
    assert_eq!(loaded.tasks.len(), 1);
    assert_eq!(loaded.tasks[0].title, "RoundTrip Task");
}

#[test]
fn resolve_data_dir_returns_same_path_on_repeated_calls() {
    let path1 = storage::resolve_data_dir().expect("first call failed");
    let path2 = storage::resolve_data_dir().expect("second call failed");
    assert_eq!(
        path1, path2,
        "resolve_data_dir() must return the same path on repeated calls"
    );
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
