// Stellar - Vault Command Handlers
// @musem23
//
// CLI command handlers for vault operations.
// Extracted from main.rs for better separation of concerns.

use std::path::PathBuf;

use crate::ui;
use crate::vault::storage::SecurityLevel;
use crate::vault::{self, Vault};

/// Vault CLI subcommands
#[derive(Debug, Clone)]
pub enum VaultAction {
    Init { level: SecurityLevel },
    Add { files: Vec<String> },
    List,
    Extract { name: String, dest: String },
    Destroy { name: String },
    Recover,
}

/// Lock a single file with password (encrypt in place)
pub fn run_lock(file: &str, keep: bool) {
    let path = match resolve_path(file) {
        Some(p) => PathBuf::from(p),
        None => {
            ui::print_error(&format!("Invalid path: {}", file));
            return;
        }
    };

    let password = match prompt_password("Password: ") {
        Some(p) => p,
        None => return,
    };

    let confirm = match prompt_password("Confirm password: ") {
        Some(p) => p,
        None => return,
    };

    if password != confirm {
        ui::print_error("Passwords do not match");
        return;
    }

    if let Err(e) = vault::validate_password(&password) {
        ui::print_error(&format!("{}", e));
        return;
    }

    match vault::lock_file(&path, &password, keep) {
        Ok(vault_path) => {
            ui::print_success(&format!("Locked: {}", vault_path.display()));
            if !keep {
                ui::print_info("Original file removed");
            }
        }
        Err(e) => ui::print_error(&format!("{}", e)),
    }
}

/// Unlock a .stlr file (decrypt)
pub fn run_unlock(file: &str) {
    let path = match resolve_path(file) {
        Some(p) => PathBuf::from(p),
        None => {
            ui::print_error(&format!("Invalid path: {}", file));
            return;
        }
    };

    let password = match prompt_password("Password: ") {
        Some(p) => p,
        None => return,
    };

    match vault::unlock_file(&path, &password) {
        Ok(original_path) => {
            ui::print_success(&format!("Unlocked: {}", original_path.display()));
        }
        Err(e) => ui::print_error(&format!("{}", e)),
    }
}

/// Run a vault subcommand
pub fn run_vault(action: VaultAction) {
    let vault = Vault::open(None);

    match action {
        VaultAction::Init { level } => init_vault(&vault, level),
        VaultAction::Add { files } => add_to_vault(&vault, files),
        VaultAction::List => list_vault(&vault),
        VaultAction::Extract { name, dest } => extract_from_vault(&vault, &name, &dest),
        VaultAction::Destroy { name } => destroy_in_vault(&vault, &name),
        VaultAction::Recover => recover_vault(&vault),
    }
}

fn init_vault(vault: &Vault, level: SecurityLevel) {
    if vault.is_initialized() {
        ui::print_error("Vault already initialized");
        return;
    }

    let password = match prompt_new_password() {
        Some(p) => p,
        None => return,
    };

    if level == SecurityLevel::Maximum {
        ui::print_warning("WARNING: Maximum security mode - NO recovery possible!");
        ui::print_warning("If you forget your password, all data will be LOST.");
        if !ui::confirm_with_default("Continue?", false) {
            ui::print_info("Cancelled");
            return;
        }
    }

    match vault.init(&password, level) {
        Ok(Some(codes)) => {
            ui::print_success("Vault initialized!");
            println!();
            ui::print_warning("RECOVERY CODES - Write these down and store separately:");
            println!();
            println!("   Code 1: {}", codes.code1);
            println!("   Code 2: {}", codes.code2);
            println!();
            ui::print_warning("Both codes are required for recovery.");
            ui::print_warning("These codes will NOT be shown again.");
        }
        Ok(None) => {
            ui::print_success("Vault initialized (maximum security - no recovery)");
        }
        Err(e) => ui::print_error(&format!("{}", e)),
    }
}

fn add_to_vault(vault: &Vault, files: Vec<String>) {
    let password = match prompt_password("Vault password: ") {
        Some(p) => p,
        None => return,
    };

    for file in files {
        let path = match resolve_path(&file) {
            Some(p) => PathBuf::from(p),
            None => {
                ui::print_error(&format!("Invalid path: {}", file));
                continue;
            }
        };

        match vault.add(&path, &password) {
            Ok(entry) => {
                ui::print_success(&format!("Added: {} ({} bytes)", entry.name, entry.size));
            }
            Err(e) => ui::print_error(&format!("Failed to add {}: {}", file, e)),
        }
    }
}

fn list_vault(vault: &Vault) {
    let password = match prompt_password("Vault password: ") {
        Some(p) => p,
        None => return,
    };

    match vault.list(&password) {
        Ok(entries) => {
            if entries.is_empty() {
                ui::print_info("Vault is empty");
            } else {
                println!();
                println!("{:<30} {:>12} {}", "NAME", "SIZE", "ADDED");
                println!("{}", "-".repeat(60));
                for entry in entries {
                    let size = format_size(entry.size);
                    let date = entry.added_at.format("%Y-%m-%d %H:%M");
                    let name = if entry.is_directory {
                        format!("{}/", entry.name)
                    } else {
                        entry.name
                    };
                    println!("{:<30} {:>12} {}", name, size, date);
                }
            }
        }
        Err(e) => ui::print_error(&format!("{}", e)),
    }
}

fn extract_from_vault(vault: &Vault, name: &str, dest: &str) {
    let password = match prompt_password("Vault password: ") {
        Some(p) => p,
        None => return,
    };

    let dest_path = match resolve_path(dest) {
        Some(p) => PathBuf::from(p),
        None => {
            ui::print_error(&format!("Invalid destination: {}", dest));
            return;
        }
    };

    match vault.extract(name, &password, &dest_path) {
        Ok(path) => {
            ui::print_success(&format!("Extracted: {}", path.display()));
        }
        Err(e) => ui::print_error(&format!("{}", e)),
    }
}

fn destroy_in_vault(vault: &Vault, name: &str) {
    let password = match prompt_password("Vault password: ") {
        Some(p) => p,
        None => return,
    };

    ui::print_warning(&format!(
        "This will permanently delete '{}' from the vault",
        name
    ));
    if !ui::confirm_with_default("Continue?", false) {
        ui::print_info("Cancelled");
        return;
    }

    match vault.destroy(name, &password) {
        Ok(()) => {
            ui::print_success(&format!("Destroyed: {}", name));
        }
        Err(e) => ui::print_error(&format!("{}", e)),
    }
}

fn recover_vault(vault: &Vault) {
    if !vault.is_initialized() {
        ui::print_error("Vault not initialized");
        return;
    }

    ui::print_info("Enter your recovery codes:");

    let code1 = match ui::input_text("Code 1") {
        Some(c) => c,
        None => return,
    };

    let code2 = match ui::input_text("Code 2") {
        Some(c) => c,
        None => return,
    };

    let new_password = match prompt_new_password() {
        Some(p) => p,
        None => return,
    };

    match vault.recover(&code1, &code2, &new_password) {
        Ok(new_codes) => {
            ui::print_success("Vault recovered!");
            println!();
            ui::print_warning("NEW RECOVERY CODES:");
            println!();
            println!("   Code 1: {}", new_codes.code1);
            println!("   Code 2: {}", new_codes.code2);
            println!();
            ui::print_warning("Old codes are now invalid.");
        }
        Err(e) => ui::print_error(&format!("{}", e)),
    }
}

// ============================================================================
// Helpers
// ============================================================================

pub fn prompt_password(prompt: &str) -> Option<String> {
    rpassword::prompt_password(prompt).ok()
}

pub fn prompt_new_password() -> Option<String> {
    let password = prompt_password("Password: ")?;
    let confirm = prompt_password("Confirm password: ")?;

    if password != confirm {
        ui::print_error("Passwords do not match");
        return None;
    }

    if let Err(e) = vault::validate_password(&password) {
        ui::print_error(&format!("{}", e));
        return None;
    }

    Some(password)
}

pub fn resolve_path(path: &str) -> Option<String> {
    let expanded = if path.starts_with('~') {
        let home = std::env::var("HOME").ok()?;
        path.replacen('~', &home, 1)
    } else {
        path.to_string()
    };

    let p = std::path::Path::new(&expanded);
    if p.is_absolute() {
        Some(expanded)
    } else {
        std::env::current_dir()
            .ok()
            .map(|cwd| cwd.join(&expanded).to_string_lossy().to_string())
    }
}

pub fn format_size(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if bytes >= GB {
        format!("{:.1} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.1} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.1} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}
