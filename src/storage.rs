use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::Path;

use crate::model::{AppError, BoardState, DoneEntry};

pub const CURRENT_LOG: &str = "current.log";

/// Load board state from the given path.
/// Returns an empty board if the file does not exist.
/// Returns an error if the file is present but cannot be parsed.
pub fn load_board_from(path: &str) -> Result<BoardState, AppError> {
    match fs::read_to_string(path) {
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(BoardState::default()),
        Err(e) => Err(AppError::Io(e)),
        Ok(contents) => {
            let board: BoardState = serde_json::from_str(&contents)?;
            if board.version != 1 {
                return Err(AppError::VersionMismatch(board.version));
            }
            Ok(board)
        }
    }
}

/// Load board state from the default `current.log` path.
pub fn load_board() -> Result<BoardState, AppError> {
    load_board_from(CURRENT_LOG)
}

/// Persist board state to the given path (pretty-printed JSON, overwrite).
pub fn save_board_to(board: &mut BoardState, path: &str) -> Result<(), AppError> {
    board.saved_at = chrono::Local::now();
    let json = serde_json::to_string_pretty(board)?;
    fs::write(path, json)?;
    Ok(())
}

/// Persist board state to the default `current.log` path.
pub fn save_board(board: &mut BoardState) -> Result<(), AppError> {
    save_board_to(board, CURRENT_LOG)
}

/// Append a completed task entry to the given log file (JSON Lines, append-only).
pub fn append_done_entry_to(entry: &DoneEntry, path: &Path) -> Result<(), AppError> {
    let line = serde_json::to_string(entry)?;
    let mut file = OpenOptions::new().create(true).append(true).open(path)?;
    writeln!(file, "{}", line)?;
    Ok(())
}

/// Append a completed task entry to today's `YYYYMMDD.log` in the current directory.
pub fn append_done_entry(entry: &DoneEntry) -> Result<(), AppError> {
    let filename = format!("{}.log", chrono::Local::now().format("%Y%m%d"));
    append_done_entry_to(entry, Path::new(&filename))
}
