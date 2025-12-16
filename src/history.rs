// Stellar - History Module
// @musem23
//
// Records file operations in a JSON history file for undo functionality.
// Stores the last 50 operations at ~/.config/stellar/history.json.
// Each operation contains the original and destination paths of moved files.

use chrono::Local;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::{env, fs};

const MAX_HISTORY: usize = 50;

#[derive(Serialize, Deserialize, Clone)]
pub struct FileMove {
    pub from: String,
    pub to: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Operation {
    pub timestamp: String,
    pub folder: String,
    pub moves: Vec<FileMove>,
}

#[derive(Serialize, Deserialize, Default)]
struct History {
    operations: Vec<Operation>,
}

pub struct UndoResult {
    pub operation_time: String,
    pub restored: usize,
    pub failed: usize,
    pub errors: Vec<String>,
}

/// Record a new operation to history
pub fn record_operation(folder: &str, moves: Vec<FileMove>) -> Result<(), String> {
    let mut history = load_history();

    history.operations.push(Operation {
        timestamp: Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
        folder: folder.to_string(),
        moves,
    });

    if history.operations.len() > MAX_HISTORY {
        history.operations = history
            .operations
            .split_off(history.operations.len() - MAX_HISTORY);
    }

    save_history(&history)
}

/// Undo the last operation by reversing all file moves
pub fn undo_last_operation() -> Result<UndoResult, String> {
    let mut history = load_history();

    let operation = history
        .operations
        .pop()
        .ok_or_else(|| "No operations to undo.".to_string())?;

    let mut restored = 0;
    let mut failed = 0;
    let mut errors = Vec::new();
    let mut folders_to_check: Vec<PathBuf> = Vec::new();

    for mv in &operation.moves {
        let dest_path = PathBuf::from(&mv.to);
        if let Some(parent) = dest_path.parent() {
            if !folders_to_check.contains(&parent.to_path_buf()) {
                folders_to_check.push(parent.to_path_buf());
            }
        }

        match restore_file(mv) {
            Ok(_) => restored += 1,
            Err(e) => {
                errors.push(e);
                failed += 1;
            }
        }
    }

    cleanup_empty_folders(&folders_to_check);

    save_history(&history)?;

    Ok(UndoResult {
        operation_time: operation.timestamp,
        restored,
        failed,
        errors,
    })
}

/// Get the N most recent operations
pub fn get_last_operations(count: usize) -> Vec<Operation> {
    let history = load_history();
    let len = history.operations.len();
    let start = len.saturating_sub(count);
    history.operations[start..].to_vec()
}

// ============================================================================
// Private helpers
// ============================================================================

fn get_history_path() -> PathBuf {
    let home = env::var("HOME").unwrap_or_else(|_| ".".to_string());
    PathBuf::from(home)
        .join(".config")
        .join("stellar")
        .join("history.json")
}

fn load_history() -> History {
    let path = get_history_path();
    fs::read_to_string(&path)
        .ok()
        .and_then(|c| serde_json::from_str(&c).ok())
        .unwrap_or_default()
}

fn save_history(history: &History) -> Result<(), String> {
    let path = get_history_path();

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("Failed to create directory: {}", e))?;
    }

    let json =
        serde_json::to_string_pretty(history).map_err(|e| format!("Failed to serialize: {}", e))?;

    fs::write(&path, json).map_err(|e| format!("Failed to write: {}", e))
}

fn restore_file(mv: &FileMove) -> Result<(), String> {
    let from = PathBuf::from(&mv.to);
    let to = PathBuf::from(&mv.from);

    if !from.exists() {
        return Err(format!("File not found: {}", mv.to));
    }

    if let Some(parent) = to.parent() {
        if !parent.exists() {
            fs::create_dir_all(parent).map_err(|e| format!("Cannot create directory: {}", e))?;
        }
    }

    fs::rename(&from, &to).map_err(|e| format!("Failed to restore {}: {}", mv.from, e))
}

/// Remove empty folders after undo (recursive up to parent)
fn cleanup_empty_folders(folders: &[PathBuf]) {
    for folder in folders {
        let mut current = folder.clone();

        while current.exists() {
            let is_empty = fs::read_dir(&current)
                .map(|mut entries| entries.next().is_none())
                .unwrap_or(false);

            if is_empty {
                let _ = fs::remove_dir(&current);
                if let Some(parent) = current.parent() {
                    current = parent.to_path_buf();
                } else {
                    break;
                }
            } else {
                break;
            }
        }
    }
}
