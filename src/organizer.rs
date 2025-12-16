// Stellar - File Organizer Module
// @musem23
//
// Moves and renames files to their destination folders.
// Handles naming conflicts by appending numeric suffixes.
// Generates dry-run previews and records moves for undo functionality.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::{fs, process::Command};

use crate::history::{self, FileMove};
use crate::renamer::{self, RenameMode};
use crate::stats::{DryRunPreview, OrganizationStats};
use crate::ui;

pub struct MoveResult {
    pub stats: OrganizationStats,
    pub moves: Vec<FileMove>,
}

/// Move files to their destination folders with optional renaming
pub fn move_files(
    source_dir: &str,
    files_map: &HashMap<String, Vec<PathBuf>>,
    rename_mode: Option<&RenameMode>,
) -> MoveResult {
    let mut stats = OrganizationStats::new();
    let mut moves: Vec<FileMove> = Vec::new();

    let total: usize = files_map.values().map(|v| v.len()).sum();
    let progress = ui::create_progress_bar(total as u64, "Organizing files...");

    for (folder_name, files) in files_map {
        let dest_dir = Path::new(source_dir).join(folder_name);
        if fs::create_dir_all(&dest_dir).is_err() {
            continue;
        }

        for file_path in files {
            let result = move_single_file(file_path, &dest_dir, rename_mode, &mut stats);
            if let Some(file_move) = result {
                moves.push(file_move);
            }
            progress.inc(1);
        }
    }

    progress.finish_with_message("Done!");
    stats.finish();

    MoveResult { stats, moves }
}

/// Generate a preview of what would happen without making changes
pub fn generate_dry_run_preview(
    source_dir: &str,
    files_map: &HashMap<String, Vec<PathBuf>>,
    rename_mode: Option<&RenameMode>,
) -> DryRunPreview {
    let mut preview = DryRunPreview::new();

    for (folder_name, files) in files_map {
        let dest_dir = Path::new(source_dir).join(folder_name);

        for file_path in files {
            let size = file_path.metadata().map(|m| m.len()).unwrap_or(0);
            let (new_name, is_rename) = get_new_name(file_path, rename_mode);
            let dest_path = dest_dir.join(&new_name);

            preview.add_move(file_path.clone(), dest_path, size, is_rename);
        }
    }

    preview
}

/// Record file moves to history for undo functionality
pub fn record_moves(folder: &str, moves: Vec<FileMove>) {
    if !moves.is_empty() {
        let _ = history::record_operation(folder, moves);
    }
}

/// Open folder in system file manager
pub fn open_folder(path: &str) {
    let cmd = if cfg!(target_os = "macos") {
        "open"
    } else if cfg!(target_os = "windows") {
        "explorer"
    } else {
        "xdg-open"
    };

    let _ = Command::new(cmd).arg(path).spawn();
}

// ============================================================================
// Private helpers
// ============================================================================

fn move_single_file(
    file_path: &Path,
    dest_dir: &Path,
    rename_mode: Option<&RenameMode>,
    stats: &mut OrganizationStats,
) -> Option<FileMove> {
    let size = file_path.metadata().map(|m| m.len()).unwrap_or(0);
    let (new_name, was_renamed) = get_new_name(file_path, rename_mode);

    if was_renamed {
        stats.add_renamed();
    }

    let dest_path = resolve_conflict(&dest_dir.join(&new_name));
    let from = file_path.to_string_lossy().to_string();
    let to = dest_path.to_string_lossy().to_string();

    match fs::rename(file_path, &dest_path) {
        Ok(_) => {
            let folder = dest_dir.file_name()?.to_string_lossy().to_string();
            stats.add_file(&folder, size);
            Some(FileMove { from, to })
        }
        Err(_) => {
            stats.add_skipped();
            None
        }
    }
}

fn get_new_name(file_path: &Path, rename_mode: Option<&RenameMode>) -> (String, bool) {
    let original = file_path
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_default();

    match rename_mode {
        Some(mode) => {
            let renamed = renamer::rename_file(file_path, mode);
            let was_renamed = renamed != original;
            (renamed, was_renamed)
        }
        None => (original, false),
    }
}

fn resolve_conflict(path: &Path) -> PathBuf {
    if !path.exists() {
        return path.to_path_buf();
    }

    let stem = path.file_stem().unwrap_or_default().to_string_lossy();
    let ext = path
        .extension()
        .map(|e| format!(".{}", e.to_string_lossy()))
        .unwrap_or_default();
    let parent = path.parent().unwrap_or(Path::new("."));

    (1..)
        .map(|i| parent.join(format!("{}-{}{}", stem, i, ext)))
        .find(|p| !p.exists())
        .unwrap_or_else(|| path.to_path_buf())
}
