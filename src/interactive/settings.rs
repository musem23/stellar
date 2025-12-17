// Stellar - Settings Menu (Interactive)
// @musem23
//
// Interactive menu for managing categories, organization mode,
// rename mode, and saving configuration.

use crate::config::{self, Config};
use crate::ui;

/// Settings menu entry point
pub fn menu(config: &mut Config) {
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

fn add_category(config: &mut Config) {
    if let Some(name) = ui::input_category_name() {
        if let Some(extensions) = ui::input_extensions() {
            config.categories.insert(name.clone(), extensions);
            ui::print_success(&format!("Category '{}' added", name));
        }
    }
}

fn edit_category(config: &mut Config) {
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

fn remove_category(config: &mut Config) {
    if let Some(name) = ui::select_category(&config.categories) {
        if ui::confirm(&format!("Remove category '{}'?", name)) {
            config.categories.remove(&name);
            ui::print_success(&format!("Category '{}' removed", name));
        }
    }
}

fn update_org_mode(config: &mut Config) {
    if let Some(mode) = ui::select_default_organization_mode(config.preferences.organization_mode) {
        config.preferences.organization_mode = mode;
        ui::print_success("Default organization mode updated");
    }
}

fn update_rename_mode(config: &mut Config) {
    if let Some(mode) = ui::select_default_rename_mode(config.preferences.rename_mode) {
        config.preferences.rename_mode = mode;
        ui::print_success("Default rename mode updated");
    }
}

fn save_config(config: &Config) {
    match config::save_config(config) {
        Ok(_) => ui::print_success("Config saved to ~/.config/stellar/stellar.toml"),
        Err(e) => ui::print_error(&e),
    }
}
