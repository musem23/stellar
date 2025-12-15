use console::{style, Term};
use dialoguer::{theme::ColorfulTheme, Confirm, Input, Select};
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
        format!("{} Organize a folder", style("[>]").green()),
        format!("{} Watch mode (auto-organize)", style("[~]").cyan()),
        format!("{} Settings", style("[*]").yellow()),
        format!("{} Exit", style("[x]").red()),
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
        .map(|f| format!("{} {}", style("[/]").cyan(), f))
        .collect();

    Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Select a folder to organize")
        .items(&display_folders)
        .default(0)
        .interact()
        .ok()
}

pub fn select_organization_mode(default: usize) -> Option<usize> {
    let options = vec![
        format!("{} By category (Documents, Images, Videos...)", style("[#]").green()),
        format!("{} By date (2024/01-january, 2024/02-february...)", style("[@]").cyan()),
        format!("{} Back", style("[<]").dim()),
    ];

    let choice = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Select organization mode")
        .items(&options)
        .default(default)
        .interact()
        .ok()?;

    if choice == 2 {
        None
    } else {
        Some(choice)
    }
}

pub fn select_rename_mode(default: usize) -> Option<usize> {
    let options = vec![
        format!("{} Clean (lowercase, dashes, no duplicates)", style("[~]").green()),
        format!("{} Date prefix (2024-01-15-filename.pdf)", style("[@]").cyan()),
        format!("{} Skip renaming", style("[-]").yellow()),
        format!("{} Back", style("[<]").dim()),
    ];

    let choice = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Select rename mode")
        .items(&options)
        .default(default)
        .interact()
        .ok()?;

    if choice == 3 {
        None
    } else {
        Some(choice)
    }
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

pub fn select_settings_menu(org_mode: usize, rename_mode: usize) -> Option<usize> {
    let org_name = match org_mode {
        0 => style("Category").green().to_string(),
        1 => style("Date").cyan().to_string(),
        _ => style("Category").green().to_string(),
    };
    let rename_name = match rename_mode {
        0 => style("Clean").green().to_string(),
        1 => style("Date prefix").cyan().to_string(),
        2 => style("Skip").yellow().to_string(),
        _ => style("Clean").green().to_string(),
    };

    let options = vec![
        format!("{} View categories", style("[#]").cyan()),
        format!("{} Add category", style("[+]").green()),
        format!("{} Edit category", style("[~]").yellow()),
        format!("{} Remove category", style("[-]").red()),
        format!("{} Organization mode: {}", style("[O]").magenta(), org_name),
        format!("{} Rename mode: {}", style("[R]").magenta(), rename_name),
        format!("{} Save changes", style("[S]").green().bold()),
        format!("{} Back", style("[<]").dim()),
    ];

    Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Settings")
        .items(&options)
        .default(0)
        .interact()
        .ok()
}

pub fn select_default_organization_mode(current: usize) -> Option<usize> {
    let options = vec![
        format!("{} By category (Documents, Images, Videos...)", style("[#]").green()),
        format!("{} By date (2024/01-january, 2024/02-february...)", style("[@]").cyan()),
    ];

    Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Default organization mode")
        .items(&options)
        .default(current)
        .interact()
        .ok()
}

pub fn select_default_rename_mode(current: usize) -> Option<usize> {
    let options = vec![
        format!("{} Clean (lowercase, dashes, no duplicates)", style("[~]").green()),
        format!("{} Date prefix (2024-01-15-filename.pdf)", style("[@]").cyan()),
        format!("{} Skip renaming", style("[-]").yellow()),
    ];

    Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Default rename mode")
        .items(&options)
        .default(current)
        .interact()
        .ok()
}

pub fn display_categories(categories: &HashMap<String, Vec<String>>) {
    println!("\n{}\n", style("Current categories:").bold());

    let mut sorted: Vec<_> = categories.iter().collect();
    sorted.sort_by(|a, b| a.0.cmp(b.0));

    for (category, extensions) in sorted {
        println!(
            "  {} {} {}",
            style("[/]").cyan(),
            style(category).bold(),
            style(format!("({})", extensions.join(", "))).dim()
        );
    }
    println!();
}

pub fn select_category(categories: &HashMap<String, Vec<String>>) -> Option<String> {
    let mut names: Vec<_> = categories.keys().cloned().collect();
    names.sort();

    let display: Vec<String> = names
        .iter()
        .map(|n| format!("{} {}", style("[/]").cyan(), n))
        .collect();

    let idx = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Select a category")
        .items(&display)
        .default(0)
        .interact()
        .ok()?;

    Some(names[idx].clone())
}

pub fn input_category_name() -> Option<String> {
    Input::with_theme(&ColorfulTheme::default())
        .with_prompt("Category name")
        .interact_text()
        .ok()
}

pub fn input_extensions() -> Option<Vec<String>> {
    let input: String = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("Extensions (comma separated, e.g.: pdf, doc, txt)")
        .interact_text()
        .ok()?;

    let extensions: Vec<String> = input
        .split(',')
        .map(|s| s.trim().to_lowercase().trim_start_matches('.').to_string())
        .filter(|s| !s.is_empty())
        .collect();

    if extensions.is_empty() {
        None
    } else {
        Some(extensions)
    }
}

pub fn confirm_use_defaults() -> bool {
    Confirm::with_theme(&ColorfulTheme::default())
        .with_prompt("Use saved preferences?")
        .default(true)
        .interact()
        .unwrap_or(true)
}
