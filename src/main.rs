mod config;
mod organizer;
mod renamer;
mod scanner;
mod ui;
mod watcher;

use config::load_config;
use renamer::RenameMode;
use std::env;
use std::fs;

fn main() {
    ui::print_banner();

    let mut config = match load_config() {
        Ok(c) => c,
        Err(e) => {
            ui::print_error(&format!("Failed to load config: {}", e));
            return;
        }
    };

    let home_dir = env::var("HOME").unwrap_or_else(|_| ".".to_string());

    loop {
        let menu_choice = match ui::select_main_menu() {
            Some(c) => c,
            None => return,
        };

        match menu_choice {
            0 => organize_folder(&config, &home_dir),
            1 => watch_folder(&config, &home_dir),
            2 => settings_menu(&mut config),
            3 => return,
            _ => return,
        }
    }
}

fn organize_folder(config: &config::Config, home_dir: &str) {
    let folders = get_available_folders(
        home_dir,
        &config.protected.user,
        &config.protected.system,
        &config.protected.dev,
    );

    if folders.is_empty() {
        ui::print_error("No folders available to organize.");
        return;
    }

    let folder_idx = match ui::select_folder(&folders) {
        Some(idx) => idx,
        None => return,
    };

    let selected_folder = &folders[folder_idx];
    let source_dir = format!("{}/{}", home_dir, selected_folder);

    let (org_mode, rename_mode) = if ui::confirm_use_defaults() {
        let rm = match config.preferences.rename_mode {
            0 => Some(RenameMode::Clean),
            1 => Some(RenameMode::DatePrefix),
            _ => None,
        };
        (config.preferences.organization_mode, rm)
    } else {
        let org = match ui::select_organization_mode(config.preferences.organization_mode) {
            Some(m) => m,
            None => return,
        };
        let rm = match ui::select_rename_mode(config.preferences.rename_mode) {
            Some(0) => Some(RenameMode::Clean),
            Some(1) => Some(RenameMode::DatePrefix),
            Some(2) => None,
            _ => return,
        };
        (org, rm)
    };

    let files_map = match org_mode {
        0 => scanner::scan_by_category(&source_dir, &config.categories),
        1 => scanner::scan_by_date(&source_dir),
        _ => return,
    };

    if files_map.is_empty() {
        ui::print_info("No files to organize in this folder.");
        return;
    }

    ui::print_preview(&files_map);

    if !ui::confirm("Proceed with organization?") {
        ui::print_info("Operation cancelled.");
        return;
    }

    organizer::move_files(&source_dir, &files_map, rename_mode.as_ref());
    ui::print_success("Files organized successfully!");

    if ui::confirm("Open folder?") {
        organizer::open_folder(&source_dir);
    }
}

fn watch_folder(config: &config::Config, home_dir: &str) {
    let folders = get_available_folders(
        home_dir,
        &config.protected.user,
        &config.protected.system,
        &config.protected.dev,
    );

    if folders.is_empty() {
        ui::print_error("No folders available to watch.");
        return;
    }

    let folder_idx = match ui::select_folder(&folders) {
        Some(idx) => idx,
        None => return,
    };

    let selected_folder = &folders[folder_idx];
    let folder_path = format!("{}/{}", home_dir, selected_folder);

    let rename_mode = if ui::confirm_use_defaults() {
        match config.preferences.rename_mode {
            0 => Some(RenameMode::Clean),
            1 => Some(RenameMode::DatePrefix),
            _ => None,
        }
    } else {
        match ui::select_rename_mode(config.preferences.rename_mode) {
            Some(0) => Some(RenameMode::Clean),
            Some(1) => Some(RenameMode::DatePrefix),
            Some(2) => None,
            _ => return,
        }
    };

    watcher::watch_folder(&folder_path, &config.categories, rename_mode);
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
            1 => {
                if let Some(name) = ui::input_category_name() {
                    if let Some(extensions) = ui::input_extensions() {
                        config.categories.insert(name.clone(), extensions);
                        ui::print_success(&format!("Category '{}' added", name));
                    }
                }
            }
            2 => {
                if let Some(name) = ui::select_category(&config.categories) {
                    ui::print_info(&format!(
                        "Current: {}",
                        config.categories.get(&name).unwrap().join(", ")
                    ));
                    if let Some(extensions) = ui::input_extensions() {
                        config.categories.insert(name.clone(), extensions);
                        ui::print_success(&format!("Category '{}' updated", name));
                    }
                }
            }
            3 => {
                if let Some(name) = ui::select_category(&config.categories) {
                    if ui::confirm(&format!("Remove category '{}'?", name)) {
                        config.categories.remove(&name);
                        ui::print_success(&format!("Category '{}' removed", name));
                    }
                }
            }
            4 => {
                if let Some(mode) = ui::select_default_organization_mode(config.preferences.organization_mode) {
                    config.preferences.organization_mode = mode;
                    ui::print_success("Default organization mode updated");
                }
            }
            5 => {
                if let Some(mode) = ui::select_default_rename_mode(config.preferences.rename_mode) {
                    config.preferences.rename_mode = mode;
                    ui::print_success("Default rename mode updated");
                }
            }
            6 => {
                match config::save_config(config) {
                    Ok(_) => ui::print_success("Config saved to ~/.config/stellar/stellar.toml"),
                    Err(e) => ui::print_error(&e),
                }
            }
            7 => return,
            _ => return,
        }
    }
}

fn get_available_folders(
    home_dir: &str,
    protected_user: &[String],
    protected_system: &[String],
    protected_dev: &[String],
) -> Vec<String> {
    let mut folders = Vec::new();

    if let Ok(entries) = fs::read_dir(home_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                if let Some(name) = path.file_name() {
                    let name = name.to_string_lossy().to_string();
                    if !name.starts_with('.')
                        && !protected_user.contains(&name)
                        && !protected_system.contains(&name)
                        && !protected_dev.contains(&name)
                    {
                        folders.push(name);
                    }
                }
            }
        }
    }

    folders.sort();
    folders
}
