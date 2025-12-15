use notify::{Config, RecommendedWatcher, RecursiveMode, Watcher, EventKind};
use std::collections::HashMap;
use std::path::Path;
use std::sync::mpsc::channel;
use std::time::Duration;

use crate::config;
use crate::organizer;
use crate::renamer::RenameMode;
use crate::ui;

pub fn watch_folder(
    folder_path: &str,
    categories: &HashMap<String, Vec<String>>,
    rename_mode: Option<RenameMode>,
) {
    ui::print_info(&format!("Watching folder: {}", folder_path));
    ui::print_info("Press Ctrl+C to stop watching...\n");

    let (tx, rx) = channel();

    let mut watcher = RecommendedWatcher::new(tx, Config::default()).unwrap();
    watcher.watch(Path::new(folder_path), RecursiveMode::NonRecursive).unwrap();

    loop {
        match rx.recv_timeout(Duration::from_secs(1)) {
            Ok(Ok(event)) => {
                if let EventKind::Create(_) = event.kind {
                    for path in event.paths {
                        if path.is_file() {
                            handle_new_file(&path, folder_path, categories, rename_mode.as_ref());
                        }
                    }
                }
            }
            Ok(Err(e)) => {
                ui::print_error(&format!("Watch error: {}", e));
            }
            Err(_) => {}
        }
    }
}

fn handle_new_file(
    file_path: &Path,
    folder_path: &str,
    categories: &HashMap<String, Vec<String>>,
    rename_mode: Option<&RenameMode>,
) {
    if let Some(ext) = file_path.extension() {
        let ext = ext.to_string_lossy().to_lowercase();
        let category = config::find_category(categories, &ext)
            .unwrap_or_else(|| "Others".to_string());

        let mut files_map = HashMap::new();
        files_map.insert(category.clone(), vec![file_path.to_path_buf()]);

        let file_name = file_path.file_name().unwrap().to_string_lossy();
        ui::print_info(&format!("New file: {} -> {}", file_name, category));

        organizer::move_files(folder_path, &files_map, rename_mode);
    }
}
