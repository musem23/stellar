// Stellar - User Interface Module
// @musem23
//
// Handles all terminal UI interactions using dialoguer and console crates.
// Provides menus, prompts, progress bars, and styled output messages.
// All user-facing text and formatting is centralized here.

use console::{style, Term};
use dialoguer::{theme::ColorfulTheme, Confirm, Input, Select};
use indicatif::{ProgressBar, ProgressStyle};
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Duration;

use crate::duplicates::DuplicateGroup;
use crate::history::Operation;
use crate::stats::{format_duration, format_size, DryRunPreview, OrganizationStats, SkippedFile};

// ============================================================================
// Banner & Main Menu
// ============================================================================

pub fn print_banner() {
    let _ = Term::stdout().clear_screen();

    let seed = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as usize;

    let c1 = seed % 6;
    let c2 = (seed / 6 + 2) % 6;
    let c3 = (seed / 36 + 4) % 6;

    println!();
    println!("    {}", apply_color(r"\|/", c1));
    println!(
        "   {} {}",
        apply_color("--*--", c2),
        apply_color("Stellar", c3)
    );
    println!("    {}", apply_color(r"/|\", (c1 + 3) % 6));
    println!();
    println!("  {}\n", style("Organize your files in a snap").dim());
}

fn apply_color(text: &str, color_idx: usize) -> console::StyledObject<&str> {
    match color_idx {
        0 => style(text).yellow().bold(),
        1 => style(text).color256(214).bold(),
        2 => style(text).color256(220).bold(),
        3 => style(text).color256(178).bold(),
        4 => style(text).color256(222).bold(),
        5 => style(text).color256(179).bold(),
        _ => style(text).yellow().bold(),
    }
}

pub fn select_main_menu() -> Option<usize> {
    let options = [
        ("[>]", "Organize a folder", "green"),
        ("[~]", "Watch mode (auto-organize)", "cyan"),
        ("[=]", "Find duplicates", "yellow"),
        ("[<]", "Undo last operation", "magenta"),
        ("[H]", "History", "blue"),
        ("[L]", "Security (lock/vault)", "red"),
        ("[*]", "Settings", "yellow"),
        ("[x]", "Exit", "red"),
    ];

    select_menu("What would you like to do?", &options, 0)
}

// ============================================================================
// Folder & Mode Selection
// ============================================================================

pub enum FolderChoice {
    Index(usize),
    CustomPath(String),
    Back,
}

pub fn select_folder(folders: &[String]) -> FolderChoice {
    let mut items: Vec<String> = folders
        .iter()
        .map(|f| format!("{} {}", style("[/]").cyan(), f))
        .collect();
    items.push(format!("{} Enter custom path...", style("[>]").yellow()));
    items.push(format!("{} Back", style("[<]").dim()));

    let choice = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Select a folder")
        .items(&items)
        .default(0)
        .interact()
        .unwrap_or(folders.len() + 1);

    if choice == folders.len() {
        if let Some(path) = input_custom_path() {
            FolderChoice::CustomPath(path)
        } else {
            FolderChoice::Back
        }
    } else if choice == folders.len() + 1 {
        FolderChoice::Back
    } else {
        FolderChoice::Index(choice)
    }
}

pub fn input_custom_path() -> Option<String> {
    Input::with_theme(&ColorfulTheme::default())
        .with_prompt("Enter folder path (e.g., ~/Desktop or /path/to/folder)")
        .allow_empty(true)
        .interact_text()
        .ok()
        .filter(|s: &String| !s.is_empty())
}

pub fn select_organization_mode(default: usize) -> Option<usize> {
    let options = [
        ("[#]", "By category (Documents, Images, Videos...)", "green"),
        ("[@]", "By date (2024/01-january...)", "cyan"),
        ("[+]", "Hybrid (Documents/2024, Images/2024...)", "magenta"),
        ("[<]", "Back", "dim"),
    ];
    select_with_back("Select organization mode", &options, default, 3)
}

pub fn select_rename_mode(default: usize) -> Option<usize> {
    let options = [
        ("[~]", "Clean (lowercase, dashes, no duplicates)", "green"),
        ("[@]", "Date prefix (2024-01-15-filename.pdf)", "cyan"),
        ("[-]", "Skip renaming", "yellow"),
        ("[<]", "Back", "dim"),
    ];
    select_with_back("Select rename mode", &options, default, 3)
}

pub fn select_default_organization_mode(current: usize) -> Option<usize> {
    select_organization_mode(current)
}

pub fn select_default_rename_mode(current: usize) -> Option<usize> {
    select_rename_mode(current)
}

// ============================================================================
// Preview & Statistics
// ============================================================================

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
    println!(
        "\n  {} {}\n",
        style("Total:").bold(),
        style(format!("{} files", total)).green()
    );
}

pub fn print_dry_run_preview(preview: &DryRunPreview) {
    println!(
        "\n{}\n",
        style("Dry-run preview (no changes made):").bold().yellow()
    );

    for (i, mv) in preview.moves.iter().take(20).enumerate() {
        let from_name = mv.from.file_name().unwrap().to_string_lossy();
        let to_folder = mv
            .to
            .parent()
            .unwrap()
            .file_name()
            .unwrap()
            .to_string_lossy();
        let to_name = mv.to.file_name().unwrap().to_string_lossy();
        let rename_tag = if mv.is_rename {
            style(" (renamed)").magenta()
        } else {
            style("")
        };

        println!(
            "  {} {} {} {}/{} {}",
            style(format!("{:>3}.", i + 1)).dim(),
            style(&from_name).red(),
            style("->").dim(),
            style(&to_folder).cyan(),
            style(&to_name).green(),
            rename_tag
        );
    }

    if preview.moves.len() > 20 {
        println!(
            "\n  {} {}",
            style("...").dim(),
            style(format!("and {} more files", preview.moves.len() - 20)).dim()
        );
    }

    println!(
        "\n  {} {} files ({})\n",
        style("Total:").bold(),
        style(preview.total_files).green(),
        style(format_size(preview.total_bytes)).cyan()
    );
}

pub fn print_statistics(stats: &OrganizationStats) {
    let sep = style("=".repeat(50)).dim();
    println!("\n{}", sep);
    println!("{}\n", style("  Organization Statistics").bold().cyan());

    println!(
        "  {} {} files moved",
        style("[>]").green(),
        style(stats.files_moved).green().bold()
    );

    if stats.files_renamed > 0 {
        println!(
            "  {} {} files renamed",
            style("[~]").magenta(),
            style(stats.files_renamed).magenta()
        );
    }
    if stats.files_skipped > 0 {
        println!(
            "  {} {} files skipped",
            style("[-]").yellow(),
            style(stats.files_skipped).yellow()
        );
        // Show details for skipped files
        if !stats.skipped_files.is_empty() {
            print_skipped_files(&stats.skipped_files);
        }
    }
    if stats.duplicates_found > 0 {
        println!(
            "  {} {} duplicates found",
            style("[=]").cyan(),
            style(stats.duplicates_found).cyan()
        );
    }

    println!(
        "  {} {} processed",
        style("[#]").blue(),
        style(format_size(stats.total_bytes)).blue()
    );
    println!(
        "  {} completed in {}",
        style("[T]").dim(),
        style(format_duration(stats.duration_ms)).dim()
    );

    if !stats.categories.is_empty() {
        println!("\n  {}", style("By category:").bold());
        let mut sorted: Vec<_> = stats.categories.iter().collect();
        sorted.sort_by(|a, b| b.1.cmp(a.1));
        for (cat, count) in sorted {
            println!(
                "    {} {} ({})",
                style("[/]").cyan(),
                cat,
                style(count).dim()
            );
        }
    }

    println!("{}\n", sep);
}

/// Display details about skipped files and their reasons
fn print_skipped_files(skipped: &[SkippedFile]) {
    println!("\n  {}", style("Skipped files:").bold().yellow());

    // Show up to 10 skipped files with reasons
    for (i, sf) in skipped.iter().take(10).enumerate() {
        let filename = sf.path.file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| sf.path.to_string_lossy().to_string());

        println!(
            "    {} {} - {}",
            style(format!("{}.", i + 1)).dim(),
            style(&filename).red(),
            style(&sf.reason).dim()
        );
    }

    if skipped.len() > 10 {
        println!(
            "    {} {}",
            style("...").dim(),
            style(format!("and {} more", skipped.len() - 10)).dim()
        );
    }
}

// ============================================================================
// Duplicates
// ============================================================================

pub fn print_duplicates(groups: &[DuplicateGroup]) {
    if groups.is_empty() {
        print_info("No duplicate files found.");
        return;
    }

    let total_dupes: usize = groups.iter().map(|g| g.files.len() - 1).sum();
    let wasted: u64 = groups
        .iter()
        .map(|g| g.size * (g.files.len() as u64 - 1))
        .sum();

    println!("\n{}\n", style("Duplicate files found:").bold().yellow());

    for (i, group) in groups.iter().enumerate() {
        println!(
            "  {} {} ({} each)",
            style(format!("Group {}:", i + 1)).bold(),
            style(format!("{} files", group.files.len())).cyan(),
            style(format_size(group.size)).dim()
        );
        for (j, file) in group.files.iter().enumerate() {
            let marker = if j == 0 {
                style("[K]").green()
            } else {
                style("[D]").red()
            };
            println!("    {} {}", marker, file.display());
        }
        println!();
    }

    println!(
        "  {} {} duplicate files wasting {}\n",
        style("Summary:").bold(),
        style(total_dupes).red(),
        style(format_size(wasted)).red()
    );
}

pub fn select_duplicates_action() -> Option<usize> {
    let options = [
        ("[x]", "Remove all duplicates (keep first)", "red"),
        ("[?]", "Review each group", "yellow"),
        ("[<]", "Cancel", "dim"),
    ];
    select_with_back("What do you want to do with duplicates?", &options, 2, 2)
}

pub fn select_file_to_keep(files: &[PathBuf]) -> Option<usize> {
    let items: Vec<String> = files
        .iter()
        .map(|f| format!("{} {}", style("[/]").cyan(), f.display()))
        .collect();

    Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Select file to KEEP (others will be deleted)")
        .items(&items)
        .default(0)
        .interact()
        .ok()
}

// ============================================================================
// History
// ============================================================================

pub fn print_history(operations: &[Operation]) {
    if operations.is_empty() {
        print_info("No operations in history.");
        return;
    }

    println!("\n{}\n", style("Recent operations:").bold());
    for (i, op) in operations.iter().rev().enumerate() {
        println!(
            "  {} {} - {} ({} files)",
            style(format!("{}.", i + 1)).dim(),
            style(&op.timestamp).cyan(),
            style(&op.folder).bold(),
            style(op.moves.len()).green()
        );
    }
    println!();
}

// ============================================================================
// Settings Menu
// ============================================================================

pub fn select_settings_menu(org_mode: usize, rename_mode: usize) -> Option<usize> {
    let org_label = match org_mode {
        0 => style("Category").green(),
        1 => style("Date").cyan(),
        2 => style("Hybrid").magenta(),
        _ => style("Category").green(),
    };
    let rename_label = match rename_mode {
        0 => style("Clean").green(),
        1 => style("Date prefix").cyan(),
        2 => style("Skip").yellow(),
        _ => style("Clean").green(),
    };

    let options = vec![
        format!("{} View categories", style("[#]").cyan()),
        format!("{} Add category", style("[+]").green()),
        format!("{} Edit category", style("[~]").yellow()),
        format!("{} Remove category", style("[-]").red()),
        format!(
            "{} Organization mode: {}",
            style("[O]").magenta(),
            org_label
        ),
        format!("{} Rename mode: {}", style("[R]").magenta(), rename_label),
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

pub fn display_categories(categories: &HashMap<String, Vec<String>>) {
    println!("\n{}\n", style("Current categories:").bold());
    let mut sorted: Vec<_> = categories.iter().collect();
    sorted.sort_by(|a, b| a.0.cmp(b.0));
    for (cat, exts) in sorted {
        println!(
            "  {} {} {}",
            style("[/]").cyan(),
            style(cat).bold(),
            style(format!("({})", exts.join(", "))).dim()
        );
    }
    println!();
}

pub fn select_category(categories: &HashMap<String, Vec<String>>) -> Option<String> {
    let mut names: Vec<_> = categories.keys().cloned().collect();
    names.sort();

    let mut items: Vec<String> = names
        .iter()
        .map(|n| format!("{} {}", style("[/]").cyan(), n))
        .collect();
    items.push(format!("{} Back", style("[<]").dim()));

    let choice = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Select a category")
        .items(&items)
        .default(0)
        .interact()
        .ok()?;

    if choice == names.len() {
        None
    } else {
        Some(names[choice].clone())
    }
}

pub fn input_category_name() -> Option<String> {
    Input::with_theme(&ColorfulTheme::default())
        .with_prompt("Category name (empty to cancel)")
        .allow_empty(true)
        .interact_text()
        .ok()
        .filter(|s: &String| !s.is_empty())
}

pub fn input_text(prompt: &str) -> Option<String> {
    Input::with_theme(&ColorfulTheme::default())
        .with_prompt(prompt)
        .interact_text()
        .ok()
}

pub fn input_extensions() -> Option<Vec<String>> {
    let input: String = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("Extensions (comma separated, e.g.: pdf, doc, txt)")
        .allow_empty(true)
        .interact_text()
        .ok()?;

    if input.is_empty() {
        return None;
    }

    let exts: Vec<String> = input
        .split(',')
        .map(|s| s.trim().to_lowercase().trim_start_matches('.').to_string())
        .filter(|s| !s.is_empty())
        .collect();

    if exts.is_empty() {
        None
    } else {
        Some(exts)
    }
}

// ============================================================================
// Confirmations & Messages
// ============================================================================

pub fn confirm(message: &str) -> bool {
    Confirm::with_theme(&ColorfulTheme::default())
        .with_prompt(message)
        .default(true)
        .interact()
        .unwrap_or(false)
}

pub fn confirm_with_default(message: &str, default: bool) -> bool {
    Confirm::with_theme(&ColorfulTheme::default())
        .with_prompt(message)
        .default(default)
        .interact()
        .unwrap_or(default)
}

pub fn confirm_use_defaults() -> bool {
    confirm_with_default("Use saved preferences?", true)
}

pub fn ask_dry_run() -> bool {
    confirm_with_default("Preview changes first (dry-run)?", true)
}

/// Prompt user after an action to go back or exit
pub fn prompt_after_action() -> bool {
    let options = vec![
        format!("{} Back to menu", style("[<]").cyan()),
        format!("{} Exit", style("[x]").red()),
    ];

    let choice = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("What next?")
        .items(&options)
        .default(0)
        .interact()
        .unwrap_or(1);

    choice == 0
}

pub fn print_success(msg: &str) {
    println!("\n{} {}", style("[+]").green().bold(), style(msg).green());
}

pub fn print_error(msg: &str) {
    println!("\n{} {}", style("[!]").red().bold(), style(msg).red());
}

pub fn print_info(msg: &str) {
    println!("\n{} {}", style("[i]").blue().bold(), msg);
}

pub fn print_warning(msg: &str) {
    println!("\n{} {}", style("[!]").yellow().bold(), style(msg).yellow());
}

// ============================================================================
// Security Menu
// ============================================================================

pub fn select_security_menu() -> Option<usize> {
    let options = [
        ("[L]", "Lock a file (encrypt in place)", "cyan"),
        ("[U]", "Unlock a file (decrypt)", "green"),
        ("[V]", "Vault (secure storage)", "yellow"),
        ("[<]", "Back", "dim"),
    ];

    select_menu("Security", &options, 0)
}

pub fn select_vault_menu(is_initialized: bool) -> Option<usize> {
    if !is_initialized {
        let options = [("[+]", "Initialize vault", "green"), ("[<]", "Back", "dim")];
        return select_menu("Vault", &options, 0);
    }

    let options = [
        ("[+]", "Add files to vault", "green"),
        ("[=]", "List vault contents", "cyan"),
        ("[>]", "Extract from vault", "yellow"),
        ("[x]", "Destroy (delete permanently)", "red"),
        ("[R]", "Recover access", "magenta"),
        ("[<]", "Back", "dim"),
    ];

    select_menu("Vault", &options, 0)
}

pub fn select_security_level() -> Option<usize> {
    let options = [
        ("[S]", "Standard (recovery codes available)", "green"),
        ("[M]", "Maximum (no recovery - most secure)", "red"),
        ("[<]", "Back", "dim"),
    ];

    select_menu("Security Level", &options, 0)
}

pub fn input_file_path(prompt: &str) -> Option<String> {
    let options = [
        ("[>]", "Browse Downloads", "cyan"),
        ("[>]", "Browse Desktop", "cyan"),
        ("[>]", "Browse current directory", "cyan"),
        ("[?]", "Type path manually", "yellow"),
        ("[<]", "Back", "dim"),
    ];

    loop {
        let choice = select_menu(prompt, &options, 0)?;

        let result = match choice {
            0 => browse_directory("~/Downloads"),
            1 => browse_directory("~/Desktop"),
            2 => browse_directory("."),
            3 => {
                let path: String = Input::with_theme(&ColorfulTheme::default())
                    .with_prompt("Path (~ supported)")
                    .interact_text()
                    .ok()?;
                Some(expand_path(&path))
            }
            _ => return None,
        };

        if result.is_some() {
            return result;
        }
    }
}

fn expand_path(path: &str) -> String {
    if path.starts_with('~') {
        if let Ok(home) = std::env::var("HOME") {
            return path.replacen('~', &home, 1);
        }
    }
    path.to_string()
}

fn browse_directory(dir: &str) -> Option<String> {
    let expanded = if dir.starts_with('~') {
        let home = std::env::var("HOME").ok()?;
        dir.replacen('~', &home, 1)
    } else {
        dir.to_string()
    };

    let path = std::path::Path::new(&expanded);
    if !path.is_dir() {
        println!("{}", style("Directory not found").red());
        return None;
    }

    let mut entries: Vec<_> = std::fs::read_dir(path)
        .ok()?
        .filter_map(|e| e.ok())
        .filter(|e| !e.file_name().to_string_lossy().starts_with('.'))
        .collect();

    entries.sort_by_key(|e| e.file_name());

    let items: Vec<String> = entries
        .iter()
        .map(|e| {
            let name = e.file_name().to_string_lossy().to_string();
            if e.path().is_dir() {
                format!("{}/", name)
            } else {
                name
            }
        })
        .collect();

    if items.is_empty() {
        println!("{}", style("Directory is empty").yellow());
        return None;
    }

    let item_refs: Vec<&str> = items.iter().map(|s| s.as_str()).collect();
    let idx = select_from_list(&format!("Select from {}", dir), &item_refs)?;

    let selected = &entries[idx];
    let selected_path = selected.path();

    if selected_path.is_dir() {
        browse_directory(&selected_path.to_string_lossy())
    } else {
        Some(selected_path.to_string_lossy().to_string())
    }
}

pub fn select_from_list(prompt: &str, items: &[&str]) -> Option<usize> {
    let mut menu_items: Vec<String> = items
        .iter()
        .map(|s| format!("{} {}", style("[>]").cyan(), s))
        .collect();
    menu_items.push(format!("{} Back", style("[<]").dim()));

    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt(prompt)
        .items(&menu_items)
        .default(0)
        .interact_opt()
        .ok()??;

    if selection == items.len() {
        None
    } else {
        Some(selection)
    }
}

// ============================================================================
// Progress Indicators
// ============================================================================

pub fn create_progress_bar(total: u64, message: &str) -> ProgressBar {
    let pb = ProgressBar::new(total);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} {msg}")
            .unwrap()
            .progress_chars("#>-"),
    );
    pb.set_message(message.to_string());
    pb
}

pub fn create_spinner(message: &str) -> ProgressBar {
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.cyan} {msg}")
            .unwrap(),
    );
    pb.set_message(message.to_string());
    pb.enable_steady_tick(Duration::from_millis(100));
    pb
}

// ============================================================================
// Helpers
// ============================================================================

fn select_menu(prompt: &str, options: &[(&str, &str, &str)], default: usize) -> Option<usize> {
    let items: Vec<String> = options
        .iter()
        .map(|(icon, text, color)| {
            let styled_icon = match *color {
                "green" => style(*icon).green(),
                "cyan" => style(*icon).cyan(),
                "yellow" => style(*icon).yellow(),
                "magenta" => style(*icon).magenta(),
                "blue" => style(*icon).blue(),
                "red" => style(*icon).red(),
                _ => style(*icon).dim(),
            };
            format!("{} {}", styled_icon, text)
        })
        .collect();

    Select::with_theme(&ColorfulTheme::default())
        .with_prompt(prompt)
        .items(&items)
        .default(default)
        .interact()
        .ok()
}

fn select_with_back(
    prompt: &str,
    options: &[(&str, &str, &str)],
    default: usize,
    back_idx: usize,
) -> Option<usize> {
    let choice = select_menu(prompt, options, default)?;
    if choice == back_idx {
        None
    } else {
        Some(choice)
    }
}
