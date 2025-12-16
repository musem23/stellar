// Stellar - File Watcher Module
// @musem23
//
// Monitors a folder for new files and automatically organizes them.
// Uses the notify crate for cross-platform filesystem events.
// Gracefully handles Ctrl+C interruption.

use notify::{Config, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::collections::HashMap;
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::channel;
use std::sync::Arc;
use std::time::Duration;

use crate::config;
use crate::organizer;
use crate::renamer::RenameMode;
use crate::ui;

/// Watch a folder and auto-organize new files
pub fn watch_folder(
    folder_path: &str,
    categories: &HashMap<String, Vec<String>>,
    rename_mode: Option<RenameMode>,
) {
    ui::print_info(&format!("Watching folder: {}", folder_path));
    ui::print_info("Press Ctrl+C to stop watching...\n");

    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();

    if let Err(e) = ctrlc::set_handler(move || {
        r.store(false, Ordering::SeqCst);
    }) {
        ui::print_warning(&format!("Could not set Ctrl+C handler: {}", e));
    }

    let (tx, rx) = channel();

    let mut watcher = match RecommendedWatcher::new(tx, Config::default()) {
        Ok(w) => w,
        Err(e) => {
            ui::print_error(&format!("Failed to create watcher: {}", e));
            return;
        }
    };

    if let Err(e) = watcher.watch(Path::new(folder_path), RecursiveMode::NonRecursive) {
        ui::print_error(&format!("Failed to watch folder: {}", e));
        return;
    }

    while running.load(Ordering::SeqCst) {
        match rx.recv_timeout(Duration::from_millis(500)) {
            Ok(Ok(event)) => {
                if matches!(event.kind, EventKind::Create(_)) {
                    for path in event.paths {
                        if path.is_file() {
                            process_new_file(&path, folder_path, categories, rename_mode.as_ref());
                        }
                    }
                }
            }
            Ok(Err(e)) => ui::print_error(&format!("Watch error: {}", e)),
            Err(_) => {} // Timeout, check if still running
        }
    }

    ui::print_info("\nWatch mode stopped.");
}

fn process_new_file(
    file_path: &Path,
    folder_path: &str,
    categories: &HashMap<String, Vec<String>>,
    rename_mode: Option<&RenameMode>,
) {
    let ext = match file_path.extension() {
        Some(e) => e.to_string_lossy().to_lowercase(),
        None => return,
    };

    let category = config::find_category(categories, &ext).unwrap_or_else(|| "Others".into());
    let file_name = file_path.file_name().unwrap().to_string_lossy();

    ui::print_info(&format!("New file: {} -> {}", file_name, category));

    let mut files_map = HashMap::new();
    files_map.insert(category, vec![file_path.to_path_buf()]);

    organizer::move_files(folder_path, &files_map, rename_mode);
}
