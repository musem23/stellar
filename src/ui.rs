use console::{style, Term};
use dialoguer::{theme::ColorfulTheme, Confirm, Select};
use std::collections::HashMap;
use std::path::PathBuf;

pub fn print_banner() {
    let term = Term::stdout();
    let _ = term.clear_screen();

    println!(
        "{}",
        style(
            r"
  \|/
 --*--  Stellar
  /|\
"
        )
        .cyan()
        .bold()
    );
    println!("  {}\n", style("Organize your files in a snap").dim());
}

pub fn select_main_menu() -> Option<usize> {
    let options = vec![
        "[>] Organize a folder",
        "[~] Watch mode (auto-organize)",
        "[*] Settings",
        "[x] Exit",
    ];

    Select::with_theme(&ColorfulTheme::default())
        .with_prompt("What would you like to do?")
        .items(&options)
        .default(0)
        .interact()
        .ok()
}

pub fn select_folder(folders: &[String]) -> Option<usize> {
    let display_folders: Vec<String> = folders
        .iter()
        .map(|f| format!("[/] {}", f))
        .collect();

    Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Select a folder to organize")
        .items(&display_folders)
        .default(0)
        .interact()
        .ok()
}

pub fn select_organization_mode() -> Option<usize> {
    let options = vec![
        "[#] By category (Documents, Images, Videos...)",
        "[@] By date (2024/01-january, 2024/02-february...)",
    ];

    Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Select organization mode")
        .items(&options)
        .default(0)
        .interact()
        .ok()
}

pub fn select_rename_mode() -> Option<usize> {
    let options = vec![
        "[~] Clean (lowercase, dashes, no duplicates)",
        "[@] Date prefix (2024-01-15-filename.pdf)",
        "[-] Skip renaming",
    ];

    Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Select rename mode")
        .items(&options)
        .default(0)
        .interact()
        .ok()
}

pub fn print_preview(files_map: &HashMap<String, Vec<PathBuf>>) {
    println!("\n{}\n", style("Organization preview:").bold());

    let mut total = 0;
    for (category, files) in files_map {
        println!(
            "  {} {} {}",
            style("[/]").cyan(),
            style(category).bold(),
            style(format!("({} files)", files.len())).dim()
        );
        total += files.len();
    }
    println!("\n  {} {}\n", style("Total:").bold(), style(format!("{} files", total)).green());
}

pub fn confirm(message: &str) -> bool {
    Confirm::with_theme(&ColorfulTheme::default())
        .with_prompt(message)
        .default(true)
        .interact()
        .unwrap_or(false)
}

pub fn print_success(message: &str) {
    println!("\n{} {}", style("[+]").green().bold(), style(message).green());
}

pub fn print_error(message: &str) {
    println!("\n{} {}", style("[!]").red().bold(), style(message).red());
}

pub fn print_info(message: &str) {
    println!("\n{} {}", style("[i]").blue().bold(), message);
}
