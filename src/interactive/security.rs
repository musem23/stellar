// Stellar - Security Menu (Interactive)
// @musem23
//
// Interactive menu for file encryption (lock/unlock) and vault management.
// Extracted from main.rs for better separation of concerns.

use std::path::PathBuf;

use crate::ui;
use crate::vault;
use crate::vault::commands::{format_size, prompt_new_password, prompt_password, resolve_path};
use crate::vault::storage::SecurityLevel;

/// Security menu entry point
pub fn menu(home_dir: &str) {
    loop {
        match ui::select_security_menu() {
            Some(0) => lock_file_interactive(home_dir),
            Some(1) => unlock_file_interactive(home_dir),
            Some(2) => vault_menu(),
            _ => return,
        }
    }
}

fn lock_file_interactive(_home_dir: &str) {
    let file_path = match ui::input_file_path("File to lock") {
        Some(p) => match resolve_path(&p) {
            Some(resolved) => PathBuf::from(resolved),
            None => {
                ui::print_error(&format!("Invalid path: {}", p));
                return;
            }
        },
        None => return,
    };

    if !file_path.exists() {
        ui::print_error(&format!("File not found: {}", file_path.display()));
        return;
    }

    let password = match prompt_new_password() {
        Some(p) => p,
        None => return,
    };

    let keep = ui::confirm_with_default("Keep original file?", false);

    let spinner = ui::create_spinner("Encrypting (securing with Argon2)...");
    let result = vault::lock_file(&file_path, &password, keep);
    spinner.finish_and_clear();

    match result {
        Ok(vault_path) => {
            ui::print_success(&format!("Locked: {}", vault_path.display()));
        }
        Err(e) => ui::print_error(&format!("{}", e)),
    }
}

fn unlock_file_interactive(_home_dir: &str) {
    let file_path = match ui::input_file_path("File to unlock (.stlr)") {
        Some(p) => match resolve_path(&p) {
            Some(resolved) => PathBuf::from(resolved),
            None => {
                ui::print_error(&format!("Invalid path: {}", p));
                return;
            }
        },
        None => return,
    };

    if !file_path.exists() {
        ui::print_error(&format!("File not found: {}", file_path.display()));
        return;
    }

    let password = match prompt_password("Password: ") {
        Some(p) => p,
        None => return,
    };

    let spinner = ui::create_spinner("Decrypting...");
    let result = vault::unlock_file(&file_path, &password);
    spinner.finish_and_clear();

    match result {
        Ok(original_path) => {
            ui::print_success(&format!("Unlocked: {}", original_path.display()));
        }
        Err(e) => ui::print_error(&format!("{}", e)),
    }
}

fn vault_menu() {
    let v = vault::Vault::open(None);

    loop {
        let is_init = v.is_initialized();
        match ui::select_vault_menu(is_init) {
            Some(0) if !is_init => vault_init(&v),
            Some(0) if is_init => vault_add(&v),
            Some(1) if is_init => vault_list(&v),
            Some(2) if is_init => vault_extract(&v),
            Some(3) if is_init => vault_destroy(&v),
            Some(4) if is_init => vault_recover(&v),
            _ => return,
        }
    }
}

fn vault_init(v: &vault::Vault) {
    let level = match ui::select_security_level() {
        Some(0) => SecurityLevel::Standard,
        Some(1) => SecurityLevel::Maximum,
        _ => return,
    };

    if level == SecurityLevel::Maximum {
        ui::print_warning("WARNING: Maximum security - NO recovery possible!");
        ui::print_warning("If you forget your password, all data will be LOST.");
        if !ui::confirm_with_default("Continue?", false) {
            return;
        }
    }

    let password = match prompt_new_password() {
        Some(p) => p,
        None => return,
    };

    match v.init(&password, level) {
        Ok(Some(codes)) => {
            ui::print_success("Vault initialized!");
            println!();
            ui::print_warning("RECOVERY CODES - Write these down:");
            println!();
            println!("   Code 1: {}", codes.code1);
            println!("   Code 2: {}", codes.code2);
            println!();
            ui::print_warning("Both codes required. NOT shown again.");
        }
        Ok(None) => {
            ui::print_success("Vault initialized (maximum security)");
        }
        Err(e) => ui::print_error(&format!("{}", e)),
    }
}

fn vault_add(v: &vault::Vault) {
    let file_path = match ui::input_file_path("File/folder to add") {
        Some(p) => match resolve_path(&p) {
            Some(resolved) => PathBuf::from(resolved),
            None => {
                ui::print_error("Invalid path");
                return;
            }
        },
        None => return,
    };

    let password = match prompt_password("Vault password: ") {
        Some(p) => p,
        None => return,
    };

    match v.add(&file_path, &password) {
        Ok(entry) => {
            ui::print_success(&format!("Added: {} ({} bytes)", entry.name, entry.size));
        }
        Err(e) => ui::print_error(&format!("{}", e)),
    }
}

fn vault_list(v: &vault::Vault) {
    let password = match prompt_password("Vault password: ") {
        Some(p) => p,
        None => return,
    };

    match v.list(&password) {
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

fn vault_extract(v: &vault::Vault) {
    let password = match prompt_password("Vault password: ") {
        Some(p) => p,
        None => return,
    };

    let entries = match v.list(&password) {
        Ok(e) => e,
        Err(e) => {
            ui::print_error(&format!("{}", e));
            return;
        }
    };

    if entries.is_empty() {
        ui::print_info("Vault is empty");
        return;
    }

    let names: Vec<&str> = entries.iter().map(|e| e.name.as_str()).collect();
    let idx = match ui::select_from_list("Select file to extract", &names) {
        Some(i) => i,
        None => return,
    };

    let dest = match ui::input_file_path("Extract to (directory)") {
        Some(p) => match resolve_path(&p) {
            Some(resolved) => PathBuf::from(resolved),
            None => PathBuf::from("."),
        },
        None => PathBuf::from("."),
    };

    match v.extract(&entries[idx].name, &password, &dest) {
        Ok(path) => {
            ui::print_success(&format!("Extracted: {}", path.display()));
        }
        Err(e) => ui::print_error(&format!("{}", e)),
    }
}

fn vault_destroy(v: &vault::Vault) {
    let password = match prompt_password("Vault password: ") {
        Some(p) => p,
        None => return,
    };

    let entries = match v.list(&password) {
        Ok(e) => e,
        Err(e) => {
            ui::print_error(&format!("{}", e));
            return;
        }
    };

    if entries.is_empty() {
        ui::print_info("Vault is empty");
        return;
    }

    let names: Vec<&str> = entries.iter().map(|e| e.name.as_str()).collect();
    let idx = match ui::select_from_list("Select file to destroy", &names) {
        Some(i) => i,
        None => return,
    };

    ui::print_warning(&format!("This will PERMANENTLY delete '{}'", names[idx]));
    if !ui::confirm_with_default("Continue?", false) {
        return;
    }

    match v.destroy(&entries[idx].name, &password) {
        Ok(()) => {
            ui::print_success(&format!("Destroyed: {}", names[idx]));
        }
        Err(e) => ui::print_error(&format!("{}", e)),
    }
}

fn vault_recover(v: &vault::Vault) {
    ui::print_info("Enter recovery codes:");

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

    match v.recover(&code1, &code2, &new_password) {
        Ok(codes) => {
            ui::print_success("Vault recovered!");
            println!();
            ui::print_warning("NEW RECOVERY CODES:");
            println!("   Code 1: {}", codes.code1);
            println!("   Code 2: {}", codes.code2);
        }
        Err(e) => ui::print_error(&format!("{}", e)),
    }
}
