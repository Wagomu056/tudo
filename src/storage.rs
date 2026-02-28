use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};

use directories::ProjectDirs;

use crate::model::{AppError, BoardState, DoneEntry};

/// Resolve the platform-appropriate local data directory for the application.
/// Creates the directory if it does not exist.
///
/// Returns paths like:
///   macOS:   ~/Library/Application Support/tudo
///   Linux:   ~/.local/share/tudo  (or $XDG_DATA_HOME/tudo)
///   Windows: %LOCALAPPDATA%\tudo
///   Fallback: ~/.tudo
pub fn resolve_data_dir() -> Result<PathBuf, AppError> {
    let path = if let Some(proj) = ProjectDirs::from("", "", "tudo") {
        proj.data_local_dir().to_path_buf()
    } else {
        let home = std::env::var("HOME")
            .or_else(|_| std::env::var("USERPROFILE"))
            .map_err(|_| {
                AppError::Other(
                    "cannot determine application data directory: \
                     no valid home directory found"
                        .to_string(),
                )
            })?;
        PathBuf::from(home).join(".tudo")
    };
    fs::create_dir_all(&path)?;
    Ok(path)
}

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

/// Load board state from `current.log` in the platform data directory.
pub fn load_board() -> Result<BoardState, AppError> {
    let data_dir = resolve_data_dir()?;
    let path = data_dir.join("current.log");
    let path_str = path
        .to_str()
        .ok_or_else(|| AppError::Other("data directory path is not valid UTF-8".to_string()))?;
    load_board_from(path_str)
}

/// Persist board state to the given path (pretty-printed JSON, overwrite).
pub fn save_board_to(board: &mut BoardState, path: &str) -> Result<(), AppError> {
    board.saved_at = chrono::Local::now();
    let json = serde_json::to_string_pretty(board)?;
    fs::write(path, json)?;
    Ok(())
}

/// Persist board state to `current.log` in the platform data directory.
pub fn save_board(board: &mut BoardState) -> Result<(), AppError> {
    let data_dir = resolve_data_dir()?;
    let path = data_dir.join("current.log");
    let path_str = path
        .to_str()
        .ok_or_else(|| AppError::Other("data directory path is not valid UTF-8".to_string()))?;
    save_board_to(board, path_str)
}

/// Append a completed task entry to the given log file (JSON Lines, append-only).
pub fn append_done_entry_to(entry: &DoneEntry, path: &Path) -> Result<(), AppError> {
    let line = serde_json::to_string(entry)?;
    let mut file = OpenOptions::new().create(true).append(true).open(path)?;
    writeln!(file, "{}", line)?;
    Ok(())
}

/// Append a completed task entry to today's `YYYYMMDD.log` in the platform data directory.
pub fn append_done_entry(entry: &DoneEntry) -> Result<(), AppError> {
    let data_dir = resolve_data_dir()?;
    let filename = format!("{}.log", chrono::Local::now().format("%Y%m%d"));
    append_done_entry_to(entry, &data_dir.join(filename))
}
