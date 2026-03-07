use tudo::app::AppState;
use tudo::model::{BoardState, InputState, Memo, Status, Task, UrlHitRegion};

// ── T002: memo_new_has_correct_fields ────────────────────────────────────────

#[test]
fn memo_new_has_correct_fields() {
    let m = Memo::new(1, "title".to_string());
    assert_eq!(m.id, 1);
    assert_eq!(m.title, "title");
    assert_eq!(m.detail, "");
}

// ── T003 (model): board_state_with_memos_round_trips ─────────────────────────

#[test]
fn board_state_with_memos_round_trips() {
    let mut board = BoardState::default();
    board.memos.push(Memo::new(1, "Alpha".to_string()));
    let mut m2 = Memo::new(2, "Beta".to_string());
    m2.detail = "some detail".to_string();
    board.memos.push(m2);
    board.next_memo_id = 3;

    let json = serde_json::to_string(&board).expect("serialize");
    let loaded: BoardState = serde_json::from_str(&json).expect("deserialize");

    assert_eq!(loaded.memos.len(), 2);
    assert_eq!(loaded.memos[0].id, 1);
    assert_eq!(loaded.memos[0].title, "Alpha");
    assert_eq!(loaded.memos[0].detail, "");
    assert_eq!(loaded.memos[1].id, 2);
    assert_eq!(loaded.memos[1].title, "Beta");
    assert_eq!(loaded.memos[1].detail, "some detail");
    assert_eq!(loaded.next_memo_id, 3);
}

// ── T004: board_state_missing_memos_field_deserializes_to_empty ──────────────

#[test]
fn board_state_missing_memos_field_deserializes_to_empty() {
    let json = r#"{"version":1,"next_id":1,"tasks":[],"saved_at":"2026-03-01T00:00:00+09:00"}"#;
    let board: BoardState = serde_json::from_str(json).expect("deserialize");
    assert!(board.memos.is_empty());
    assert_eq!(board.next_memo_id, 1);
}

// ── T003: UrlHitRegion struct field accessibility ─────────────────────────────

#[test]
fn url_hit_region_fields_accessible() {
    let r = UrlHitRegion {
        row: 3,
        col_start: 10,
        col_end: 30,
        url: "https://example.com".to_string(),
    };
    assert_eq!(r.row, 3u16);
    assert_eq!(r.col_start, 10u16);
    assert_eq!(r.col_end, 30u16);
    assert_eq!(r.url, "https://example.com");
}

#[test]
fn url_hit_region_derives_debug_clone_partialeq() {
    let r = UrlHitRegion {
        row: 1,
        col_start: 0,
        col_end: 5,
        url: "http://x.com".to_string(),
    };
    let r2 = r.clone();
    assert_eq!(r, r2);
    let _ = format!("{:?}", r);
}

// ── T005: AppState::new() has empty clickable_urls ───────────────────────────

#[test]
fn appstate_new_has_empty_clickable_urls() {
    let app = AppState::new(BoardState::default());
    assert!(app.clickable_urls.is_empty());
}

// ── Status transition tests (T005) ──────────────────────────────────────────

#[test]
fn status_next_todo_to_doing() {
    assert_eq!(Status::Todo.next(), Some(Status::Doing));
}

#[test]
fn status_next_doing_to_checking() {
    assert_eq!(Status::Doing.next(), Some(Status::Checking));
}

#[test]
fn status_next_checking_to_done() {
    assert_eq!(Status::Checking.next(), Some(Status::Done));
}

#[test]
fn status_next_done_is_none() {
    assert_eq!(Status::Done.next(), None);
}

#[test]
fn status_prev_done_to_checking() {
    assert_eq!(Status::Done.prev(), Some(Status::Checking));
}

#[test]
fn status_prev_checking_to_doing() {
    assert_eq!(Status::Checking.prev(), Some(Status::Doing));
}

#[test]
fn status_prev_doing_to_todo() {
    assert_eq!(Status::Doing.prev(), Some(Status::Todo));
}

#[test]
fn status_prev_todo_is_none() {
    assert_eq!(Status::Todo.prev(), None);
}

// ── BoardState invariant tests (T007) ────────────────────────────────────────

#[test]
fn boardstate_next_id_greater_than_all_task_ids() {
    let board = BoardState::with_tasks(
        vec![
            Task::new(1, "Alpha".to_string()),
            Task::new(2, "Beta".to_string()),
        ],
        3,
    );
    assert!(board.next_id > board.tasks.all_tasks().map(|t| t.id).max().unwrap_or(0));
}

#[test]
fn boardstate_version_is_one() {
    let board = BoardState::default();
    assert_eq!(board.version, 1);
}

#[test]
fn task_title_must_not_be_empty() {
    // Task::new should trim and reject empty titles
    let result = Task::try_new(1, "  ".to_string());
    assert!(result.is_err());
}

#[test]
fn task_title_valid_when_non_empty() {
    let task = Task::try_new(1, "  hello  ".to_string());
    assert!(task.is_ok());
    assert_eq!(task.unwrap().title, "hello");
}

// ── tasks_for_column / focused_task tests (T010, T011) ───────────────────────

#[test]
fn tasks_for_column_returns_only_matching_status() {
    let mut app = AppState::new(BoardState::default());
    app.board
        .tasks
        .insert_at_top(Status::Todo, Task::new(1, "Todo task".to_string()));
    let mut doing = Task::new(2, "Doing task".to_string());
    doing.status = Status::Doing;
    app.board.tasks.insert_at_top(Status::Doing, doing);

    let todo_tasks = app.tasks_for_column(Status::Todo);
    assert_eq!(todo_tasks.len(), 1);
    assert_eq!(todo_tasks[0].title, "Todo task");

    let doing_tasks = app.tasks_for_column(Status::Doing);
    assert_eq!(doing_tasks.len(), 1);
    assert_eq!(doing_tasks[0].title, "Doing task");
}

#[test]
fn focused_task_returns_correct_task() {
    let mut app = AppState::new(BoardState::default());
    // Insert in reverse order so "First" ends up at index 0 (insert_at_top reverses order)
    app.board
        .tasks
        .insert_at_top(Status::Todo, Task::new(2, "Second".to_string()));
    app.board
        .tasks
        .insert_at_top(Status::Todo, Task::new(1, "First".to_string()));
    app.focused_col = 0;
    app.focused_card[0] = 0;

    let task = app.focused_task();
    assert!(task.is_some());
    assert_eq!(task.unwrap().title, "First");
}

#[test]
fn focused_task_returns_none_on_empty_column() {
    let app = AppState::new(BoardState::default());
    assert!(app.focused_task().is_none());
}

// ── InputState tests (T020) ──────────────────────────────────────────────────

#[test]
fn input_state_push_char_appends() {
    let mut input = InputState::default();
    input.push_char('h');
    input.push_char('i');
    assert_eq!(input.value(), "hi");
}

#[test]
fn input_state_pop_char_removes_last() {
    let mut input = InputState::default();
    input.push_char('a');
    input.push_char('b');
    input.pop_char();
    assert_eq!(input.value(), "a");
}

#[test]
fn input_state_pop_char_no_op_when_empty() {
    let mut input = InputState::default();
    input.pop_char(); // should not panic
    assert_eq!(input.value(), "");
}

#[test]
fn input_state_clear_resets_buffer() {
    let mut input = InputState::default();
    input.push_char('x');
    input.clear();
    assert_eq!(input.value(), "");
}

// ── advance_status / retreat_status tests (T028) ─────────────────────────────

#[test]
fn advance_status_todo_to_doing() {
    let mut app = AppState::new(BoardState::default());
    app.board
        .tasks
        .insert_at_top(Status::Todo, Task::new(1, "Task".to_string()));
    app.focused_col = 0;
    app.focused_card[0] = 0;

    app.advance_status();
    assert_eq!(
        app.board.tasks.tasks_for(Status::Doing)[0].status,
        Status::Doing
    );
}

#[test]
fn advance_status_done_is_noop() {
    let mut app = AppState::new(BoardState::default());
    let mut task = Task::new(1, "Task".to_string());
    task.status = Status::Done;
    app.board.tasks.insert_at_top(Status::Done, task);
    app.focused_col = 3; // Done column

    app.advance_status();
    assert_eq!(
        app.board.tasks.tasks_for(Status::Done)[0].status,
        Status::Done
    );
}

#[test]
fn retreat_status_doing_to_todo() {
    let mut app = AppState::new(BoardState::default());
    let mut task = Task::new(1, "Task".to_string());
    task.status = Status::Doing;
    app.board.tasks.insert_at_top(Status::Doing, task);
    app.focused_col = 1; // Doing column

    app.retreat_status();
    assert_eq!(
        app.board.tasks.tasks_for(Status::Todo)[0].status,
        Status::Todo
    );
}

#[test]
fn retreat_status_todo_is_noop() {
    let mut app = AppState::new(BoardState::default());
    app.board
        .tasks
        .insert_at_top(Status::Todo, Task::new(1, "Task".to_string()));
    app.focused_col = 0; // Todo column

    app.retreat_status();
    assert_eq!(
        app.board.tasks.tasks_for(Status::Todo)[0].status,
        Status::Todo
    );
}

#[test]
fn delete_focused_card_removes_task() {
    let mut app = AppState::new(BoardState::default());
    app.board
        .tasks
        .insert_at_top(Status::Todo, Task::new(1, "To delete".to_string()));
    app.focused_col = 0;
    app.focused_card[0] = 0;

    app.delete_focused_card();
    assert!(app.board.tasks.is_empty());
}
