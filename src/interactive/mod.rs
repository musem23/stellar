// Stellar - Interactive Mode
// @musem23
//
// Main interactive menu loop and folder organization flow.
// Handles user interaction for organizing, watching, and finding duplicates.

pub mod security;
pub mod settings;

use std::collections::HashMap;
use std::path::PathBuf;
use std::{env, fs};

use crate::config::Config;
use crate::duplicates;
use crate::history;
use crate::lock;
use crate::modes::{OrganizationMode, RenameMode};
use crate::organizer;
use crate::scanner;
use crate::ui;
use crate::watcher;

/// Run the interactive mode (main menu loop)
pub fn run(mut config: Config) {
    ui::print_banner();
    let home_dir = env::var("HOME").unwrap_or_else(|_| ".".to_string());

    loop {
        match ui::select_main_menu() {
            Some(0) => organize_folder(&config, &home_dir),
            Some(1) => watch_folder(&config, &home_dir),
            Some(2) => find_duplicates(&config, &home_dir),
            Some(3) => {
                if !undo_operation() {
                    return;
                }
            }
            Some(4) => {
                if !show_history() {
                    return;
                }
            }
            Some(5) => security::menu(&home_dir),
            Some(6) => settings::menu(&mut config),
            _ => return,
        }
    }
}

fn organize_folder(config: &Config, home_dir: &str) {
    let folders = get_available_folders(home_dir, &config.protected);

    let source_dir = match ui::select_folder(&folders) {
        ui::FolderChoice::Index(idx) => format!("{}/{}", home_dir, folders[idx]),
        ui::FolderChoice::CustomPath(path) => match resolve_path(&path) {
            Some(p) => p,
            None => {
                ui::print_error(&format!("Invalid path: {}", path));
                return;
            }
        },
        ui::FolderChoice::Back => return,
    };

    if !std::path::Path::new(&source_dir).is_dir() {
        ui::print_error(&format!("Not a directory: {}", source_dir));
        return;
    }

    if scanner::is_project_folder(&source_dir) {
        ui::print_warning("This appears to be a project folder.");
        if !ui::confirm_with_default("Continue anyway?", false) {
            return;
        }
    }

    let _lock = match lock::FolderLock::acquire(&source_dir) {
        Ok(l) => l,
        Err(e) => {
            ui::print_error(&e);
            return;
        }
    };

    let recursive = ui::confirm_with_default("Scan subdirectories recursively?", false);

    let (org_mode, rename_mode) = if ui::confirm_use_defaults() {
        (
            OrganizationMode::from_index(config.preferences.organization_mode),
            RenameMode::from_index(config.preferences.rename_mode),
        )
    } else {
        let org = match ui::select_organization_mode(config.preferences.organization_mode) {
            Some(m) => OrganizationMode::from_index(m),
            None => return,
        };
        let rm = match ui::select_rename_mode(config.preferences.rename_mode) {
            Some(idx) => RenameMode::from_index(idx),
            None => return,
        };
        (org, rm)
    };

    let files_map = scan_files(&source_dir, &config.categories, org_mode, recursive);
    if files_map.is_empty() {
        ui::print_info("No files to organize in this folder.");
        return;
    }

    if ui::ask_dry_run() {
        let preview =
            organizer::generate_dry_run_preview(&source_dir, &files_map, rename_mode.map(Into::into).as_ref());
        ui::print_dry_run_preview(&preview);
        if !ui::confirm("Proceed with these changes?") {
            ui::print_info("Operation cancelled.");
            return;
        }
    } else {
        ui::print_preview(&files_map);
        if !ui::confirm("Proceed with organization?") {
            ui::print_info("Operation cancelled.");
            return;
        }
    }

    let result = organizer::move_files(&source_dir, &files_map, rename_mode.map(Into::into).as_ref());
    organizer::record_moves(&source_dir, result.moves);
    ui::print_statistics(&result.stats);
    ui::print_success("Files organized successfully!");

    if ui::confirm("Open folder?") {
        organizer::open_folder(&source_dir);
    }
}

fn watch_folder(config: &Config, home_dir: &str) {
    let folders = get_available_folders(home_dir, &config.protected);

    let folder_path = match ui::select_folder(&folders) {
        ui::FolderChoice::Index(idx) => format!("{}/{}", home_dir, folders[idx]),
        ui::FolderChoice::CustomPath(path) => match resolve_path(&path) {
            Some(p) => p,
            None => {
                ui::print_error(&format!("Invalid path: {}", path));
                return;
            }
        },
        ui::FolderChoice::Back => return,
    };

    if !std::path::Path::new(&folder_path).is_dir() {
        ui::print_error(&format!("Not a directory: {}", folder_path));
        return;
    }

    let _lock = match lock::FolderLock::acquire(&folder_path) {
        Ok(l) => l,
        Err(e) => {
            ui::print_error(&e);
            return;
        }
    };

    let rename_mode = if ui::confirm_use_defaults() {
        RenameMode::from_index(config.preferences.rename_mode)
    } else {
        match ui::select_rename_mode(config.preferences.rename_mode) {
            Some(idx) => RenameMode::from_index(idx),
            None => return,
        }
    };

    watcher::watch_folder(&folder_path, &config.categories, rename_mode.map(Into::into));
}

fn find_duplicates(config: &Config, home_dir: &str) {
    let folders = get_available_folders(home_dir, &config.protected);

    let source_dir = match ui::select_folder(&folders) {
        ui::FolderChoice::Index(idx) => format!("{}/{}", home_dir, folders[idx]),
        ui::FolderChoice::CustomPath(path) => match resolve_path(&path) {
            Some(p) => p,
            None => {
                ui::print_error(&format!("Invalid path: {}", path));
                return;
            }
        },
        ui::FolderChoice::Back => return,
    };

    if !std::path::Path::new(&source_dir).is_dir() {
        ui::print_error(&format!("Not a directory: {}", source_dir));
        return;
    }
    let spinner = ui::create_spinner("Scanning for duplicates...");

    let all_files: Vec<PathBuf> = fs::read_dir(&source_dir)
        .into_iter()
        .flatten()
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.is_file())
        .collect();

    let duplicate_groups = duplicates::find_duplicates(&all_files);
    spinner.finish_and_clear();

    if duplicate_groups.is_empty() {
        ui::print_success("No duplicate files found!");
        return;
    }

    ui::print_duplicates(&duplicate_groups);
    handle_duplicates(&duplicate_groups);
}

fn handle_duplicates(groups: &[duplicates::DuplicateGroup]) {
    let action = match ui::select_duplicates_action() {
        Some(a) => a,
        None => return,
    };

    match action {
        0 => remove_all_duplicates(groups),
        1 => review_duplicates(groups),
        _ => {}
    }
}

fn remove_all_duplicates(groups: &[duplicates::DuplicateGroup]) {
    if !ui::confirm_with_default(
        "This will permanently delete duplicate files. Continue?",
        false,
    ) {
        ui::print_info("Operation cancelled.");
        return;
    }

    let mut removed = 0;
    let mut freed_bytes: u64 = 0;

    for group in groups {
        for file in group.files.iter().skip(1) {
            if fs::remove_file(file).is_ok() {
                removed += 1;
                freed_bytes += group.size;
            }
        }
    }

    ui::print_success(&format!(
        "Removed {} duplicate files, freed {}",
        removed,
        duplicates::format_size(freed_bytes)
    ));
}

fn review_duplicates(groups: &[duplicates::DuplicateGroup]) {
    for (i, group) in groups.iter().enumerate() {
        ui::print_info(&format!("Group {} of {}:", i + 1, groups.len()));
        for file in &group.files {
            println!("  - {}", file.display());
        }

        if let Some(keep_idx) = ui::select_file_to_keep(&group.files) {
            for (j, file) in group.files.iter().enumerate() {
                if j != keep_idx {
                    if let Err(e) = fs::remove_file(file) {
                        ui::print_error(&format!("Failed to remove {}: {}", file.display(), e));
                    }
                }
            }
            ui::print_success("Duplicates removed for this group.");
        }

        if i < groups.len() - 1 && !ui::confirm("Continue to next group?") {
            break;
        }
    }
}

fn undo_operation() -> bool {
    let operations = history::get_last_operations(1);
    if operations.is_empty() {
        ui::print_info("No operations to undo.");
        return ui::prompt_after_action();
    }

    let last_op = &operations[0];
    ui::print_info(&format!(
        "Last operation: {} - {} files moved from {}",
        last_op.timestamp,
        last_op.moves.len(),
        last_op.folder
    ));

    if !ui::confirm_with_default("Undo this operation?", false) {
        ui::print_info("Undo cancelled.");
        return ui::prompt_after_action();
    }

    let spinner = ui::create_spinner("Undoing operation...");
    match history::undo_last_operation() {
        Ok(result) => {
            spinner.finish_and_clear();
            if result.failed == 0 {
                ui::print_success(&format!(
                    "Successfully restored {} files from {}",
                    result.restored, result.operation_time
                ));
            } else {
                ui::print_warning(&format!(
                    "Restored {} files, {} failed",
                    result.restored, result.failed
                ));
                for error in &result.errors {
                    ui::print_error(error);
                }
            }
        }
        Err(e) => {
            spinner.finish_and_clear();
            ui::print_error(&e);
        }
    }
    ui::prompt_after_action()
}

fn show_history() -> bool {
    let operations = history::get_last_operations(10);
    ui::print_history(&operations);
    ui::prompt_after_action()
}

// ============================================================================
// Helpers
// ============================================================================

fn scan_files(
    source_dir: &str,
    categories: &HashMap<String, Vec<String>>,
    org_mode: OrganizationMode,
    recursive: bool,
) -> HashMap<String, Vec<PathBuf>> {
    let mut files_map = match org_mode {
        OrganizationMode::Category => scanner::scan_by_category(source_dir, categories),
        OrganizationMode::Date => scanner::scan_by_date(source_dir),
        OrganizationMode::Hybrid => scanner::scan_hybrid(source_dir, categories),
    };

    if recursive {
        let recursive_files = scanner::scan_recursive(source_dir, categories, org_mode.to_index());
        for (category, files) in recursive_files {
            files_map.entry(category).or_default().extend(files);
        }
    }

    files_map
}

pub fn resolve_path(path: &str) -> Option<String> {
    let expanded = if path.starts_with('~') {
        let home = env::var("HOME").ok()?;
        path.replacen('~', &home, 1)
    } else {
        path.to_string()
    };

    let p = std::path::Path::new(&expanded);
    if p.is_absolute() {
        Some(expanded)
    } else {
        env::current_dir()
            .ok()
            .map(|cwd| cwd.join(&expanded).to_string_lossy().to_string())
    }
}

fn get_available_folders(home_dir: &str, protected: &crate::config::Protected) -> Vec<String> {
    let mut folders = Vec::new();

    let entries = match fs::read_dir(home_dir) {
        Ok(e) => e,
        Err(_) => return folders,
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }

        let name = match path.file_name() {
            Some(n) => n.to_string_lossy().to_string(),
            None => continue,
        };

        if name.starts_with('.') {
            continue;
        }

        if protected.user.contains(&name)
            || protected.system.contains(&name)
            || protected.dev.contains(&name)
        {
            continue;
        }

        folders.push(name);
    }

    folders.sort();
    folders
}
