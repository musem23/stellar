// Stellar - File Organization CLI
// @musem23
//
// Entry point for the Stellar application. Handles both CLI arguments
// and interactive mode. Routes user actions to the appropriate modules
// for file organization, watching, duplicate detection, and settings.

mod config;
mod duplicates;
mod history;
mod lock;
mod organizer;
mod renamer;
mod scanner;
mod stats;
#[cfg(test)]
mod tests;
mod ui;
mod watcher;

use clap::Parser;
use renamer::RenameMode;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::{env, fs};

#[derive(Parser)]
#[command(name = "stellar")]
#[command(author = "musem23")]
#[command(version)]
#[command(about = "Organize your files in a snap")]
#[command(
    long_about = "Stellar is a fast CLI tool to organize your files automatically.\n\n\
    It can sort files by category (Documents, Images, etc.), by date (2024/01-january),\n\
    or hybrid mode (Documents/2024). Features include smart renaming with accent support,\n\
    duplicate detection, undo functionality, and watch mode for auto-organizing new files."
)]
#[command(after_help = "EXAMPLES:\n    \
    stellar ~/Downloads              Organize by category (interactive)\n    \
    stellar ~/Downloads -m date      Organize by date\n    \
    stellar ~/Downloads -m hybrid    Organize by category/year\n    \
    stellar ~/Downloads -R           Include subdirectories\n    \
    stellar ~/Downloads --dry-run    Preview without changes\n    \
    stellar ~/Downloads --watch      Auto-organize new files")]
struct Cli {
    /// Path to the folder to organize (interactive mode if omitted)
    #[arg(value_name = "FOLDER")]
    folder: Option<String>,

    /// Organization mode
    #[arg(short, long, default_value = "category", value_parser = ["category", "date", "hybrid"])]
    mode: String,

    /// Rename mode
    #[arg(short, long, default_value = "clean", value_parser = ["clean", "date-prefix", "skip"])]
    rename: String,

    /// Scan subdirectories recursively
    #[arg(short = 'R', long)]
    recursive: bool,

    /// Preview changes without applying them
    #[arg(short, long)]
    dry_run: bool,

    /// Watch folder and auto-organize new files
    #[arg(short, long)]
    watch: bool,
}

fn main() {
    let cli = Cli::parse();

    let config = match config::load_config() {
        Ok(c) => c,
        Err(e) => {
            ui::print_error(&format!("Failed to load config: {}", e));
            return;
        }
    };

    if let Some(ref folder_path) = cli.folder {
        run_cli_mode(&config, folder_path, &cli);
    } else {
        run_interactive_mode(config);
    }
}

// ============================================================================
// CLI Mode
// ============================================================================

fn run_cli_mode(config: &config::Config, folder_path: &str, cli: &Cli) {
    let source_dir = match resolve_path(folder_path) {
        Some(p) => p,
        None => {
            ui::print_error(&format!("Invalid path: {}", folder_path));
            return;
        }
    };

    if !Path::new(&source_dir).is_dir() {
        ui::print_error(&format!("Not a directory: {}", source_dir));
        return;
    }

    if scanner::is_project_folder(&source_dir) {
        ui::print_error("This is a project folder (contains .git, package.json, etc.). Aborting.");
        return;
    }

    if cli.watch {
        let rename_mode = parse_rename_mode(&cli.rename);
        ui::print_info(&format!("Watching folder: {}", source_dir));
        watcher::watch_folder(&source_dir, &config.categories, rename_mode);
        return;
    }

    let _lock = match lock::FolderLock::acquire(&source_dir) {
        Ok(l) => l,
        Err(e) => {
            ui::print_error(&e);
            return;
        }
    };

    let org_mode = parse_org_mode(&cli.mode);
    let rename_mode = parse_rename_mode(&cli.rename);
    let files_map = scan_files(&source_dir, &config.categories, org_mode, cli.recursive);

    if files_map.is_empty() {
        ui::print_info("No files to organize.");
        return;
    }

    if cli.dry_run {
        let preview =
            organizer::generate_dry_run_preview(&source_dir, &files_map, rename_mode.as_ref());
        ui::print_dry_run_preview(&preview);
        ui::print_info("Dry-run complete. No changes were made.");
        return;
    }

    ui::print_preview(&files_map);
    let result = organizer::move_files(&source_dir, &files_map, rename_mode.as_ref());
    organizer::record_moves(&source_dir, result.moves);
    ui::print_statistics(&result.stats);
    ui::print_success("Files organized successfully!");
}

fn resolve_path(path: &str) -> Option<String> {
    let expanded = if path.starts_with('~') {
        let home = env::var("HOME").ok()?;
        path.replacen('~', &home, 1)
    } else {
        path.to_string()
    };

    let path = Path::new(&expanded);
    if path.is_absolute() {
        Some(expanded)
    } else {
        env::current_dir()
            .ok()
            .map(|cwd| cwd.join(&expanded).to_string_lossy().to_string())
    }
}

fn parse_org_mode(mode: &str) -> usize {
    match mode.to_lowercase().as_str() {
        "category" | "cat" | "c" => 0,
        "date" | "d" => 1,
        "hybrid" | "h" => 2,
        _ => 0,
    }
}

fn parse_rename_mode(mode: &str) -> Option<RenameMode> {
    match mode.to_lowercase().as_str() {
        "clean" | "c" => Some(RenameMode::Clean),
        "date-prefix" | "date" | "d" => Some(RenameMode::DatePrefix),
        "skip" | "none" | "s" => None,
        _ => Some(RenameMode::Clean),
    }
}

// ============================================================================
// Interactive Mode
// ============================================================================

fn run_interactive_mode(mut config: config::Config) {
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
            Some(5) => settings_menu(&mut config),
            _ => return,
        }
    }
}

fn organize_folder(config: &config::Config, home_dir: &str) {
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
            config.preferences.organization_mode,
            mode_from_index(config.preferences.rename_mode),
        )
    } else {
        let org = match ui::select_organization_mode(config.preferences.organization_mode) {
            Some(m) => m,
            None => return,
        };
        let rm = match ui::select_rename_mode(config.preferences.rename_mode) {
            Some(idx) => mode_from_index(idx),
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
            organizer::generate_dry_run_preview(&source_dir, &files_map, rename_mode.as_ref());
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

    let result = organizer::move_files(&source_dir, &files_map, rename_mode.as_ref());
    organizer::record_moves(&source_dir, result.moves);
    ui::print_statistics(&result.stats);
    ui::print_success("Files organized successfully!");

    if ui::confirm("Open folder?") {
        organizer::open_folder(&source_dir);
    }
}

fn watch_folder(config: &config::Config, home_dir: &str) {
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
        mode_from_index(config.preferences.rename_mode)
    } else {
        match ui::select_rename_mode(config.preferences.rename_mode) {
            Some(idx) => mode_from_index(idx),
            None => return,
        }
    };

    watcher::watch_folder(&folder_path, &config.categories, rename_mode);
}

fn find_duplicates(config: &config::Config, home_dir: &str) {
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

fn settings_menu(config: &mut config::Config) {
    loop {
        let choice = match ui::select_settings_menu(
            config.preferences.organization_mode,
            config.preferences.rename_mode,
        ) {
            Some(c) => c,
            None => return,
        };

        match choice {
            0 => ui::display_categories(&config.categories),
            1 => add_category(config),
            2 => edit_category(config),
            3 => remove_category(config),
            4 => update_org_mode(config),
            5 => update_rename_mode(config),
            6 => save_config(config),
            _ => return,
        }
    }
}

fn add_category(config: &mut config::Config) {
    if let Some(name) = ui::input_category_name() {
        if let Some(extensions) = ui::input_extensions() {
            config.categories.insert(name.clone(), extensions);
            ui::print_success(&format!("Category '{}' added", name));
        }
    }
}

fn edit_category(config: &mut config::Config) {
    if let Some(name) = ui::select_category(&config.categories) {
        if let Some(exts) = config.categories.get(&name) {
            ui::print_info(&format!("Current: {}", exts.join(", ")));
        }
        if let Some(extensions) = ui::input_extensions() {
            config.categories.insert(name.clone(), extensions);
            ui::print_success(&format!("Category '{}' updated", name));
        }
    }
}

fn remove_category(config: &mut config::Config) {
    if let Some(name) = ui::select_category(&config.categories) {
        if ui::confirm(&format!("Remove category '{}'?", name)) {
            config.categories.remove(&name);
            ui::print_success(&format!("Category '{}' removed", name));
        }
    }
}

fn update_org_mode(config: &mut config::Config) {
    if let Some(mode) = ui::select_default_organization_mode(config.preferences.organization_mode) {
        config.preferences.organization_mode = mode;
        ui::print_success("Default organization mode updated");
    }
}

fn update_rename_mode(config: &mut config::Config) {
    if let Some(mode) = ui::select_default_rename_mode(config.preferences.rename_mode) {
        config.preferences.rename_mode = mode;
        ui::print_success("Default rename mode updated");
    }
}

fn save_config(config: &config::Config) {
    match config::save_config(config) {
        Ok(_) => ui::print_success("Config saved to ~/.config/stellar/stellar.toml"),
        Err(e) => ui::print_error(&e),
    }
}

// ============================================================================
// Helpers
// ============================================================================

fn scan_files(
    source_dir: &str,
    categories: &HashMap<String, Vec<String>>,
    org_mode: usize,
    recursive: bool,
) -> HashMap<String, Vec<PathBuf>> {
    let mut files_map = match org_mode {
        0 => scanner::scan_by_category(source_dir, categories),
        1 => scanner::scan_by_date(source_dir),
        2 => scanner::scan_hybrid(source_dir, categories),
        _ => scanner::scan_by_category(source_dir, categories),
    };

    if recursive {
        let recursive_files = scanner::scan_recursive(source_dir, categories, org_mode);
        for (category, files) in recursive_files {
            files_map.entry(category).or_default().extend(files);
        }
    }

    files_map
}

fn mode_from_index(index: usize) -> Option<RenameMode> {
    match index {
        0 => Some(RenameMode::Clean),
        1 => Some(RenameMode::DatePrefix),
        _ => None,
    }
}

fn get_available_folders(home_dir: &str, protected: &config::Protected) -> Vec<String> {
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
