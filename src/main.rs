// Stellar - File Organization CLI
// @musem23
//
// Entry point for the Stellar application.
// Routes CLI arguments to appropriate modules.
// Interactive mode is handled by the interactive module.

mod config;
mod duplicates;
mod history;
mod interactive;
mod lock;
mod modes;
mod organizer;
mod renamer;
mod scanner;
mod stats;
#[cfg(test)]
mod tests;
mod ui;
mod vault;
mod watcher;

use clap::{Parser, Subcommand};
use modes::{OrganizationMode, RenameMode};
use std::path::Path;
use vault::commands::{resolve_path, VaultAction};
use vault::storage::SecurityLevel;

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

    /// Subcommand
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Lock a file in place (encrypt)
    Lock {
        /// File to lock
        file: String,
        /// Keep the original file
        #[arg(short, long)]
        keep: bool,
    },
    /// Unlock a .stlr file (decrypt)
    Unlock {
        /// File to unlock
        file: String,
    },
    /// Vault commands (centralized secure storage)
    Vault {
        #[command(subcommand)]
        action: VaultCommands,
    },
}

#[derive(Subcommand)]
enum VaultCommands {
    /// Initialize a new vault
    Init {
        /// Security level: standard (with recovery) or maximum (no recovery)
        #[arg(short, long, default_value = "standard", value_parser = ["standard", "maximum"])]
        level: String,
    },
    /// Add files to the vault
    Add {
        /// Files or directories to add
        #[arg(required = true)]
        files: Vec<String>,
    },
    /// List vault contents
    List,
    /// Extract a file from the vault
    Extract {
        /// Name of the file to extract
        name: String,
        /// Destination directory
        #[arg(short, long, default_value = ".")]
        dest: String,
    },
    /// Permanently remove a file from the vault
    Destroy {
        /// Name of the file to destroy
        name: String,
    },
    /// Recover vault access using recovery codes
    Recover,
}

fn main() {
    let cli = Cli::parse();

    // Handle subcommands first
    if let Some(cmd) = cli.command {
        match cmd {
            Commands::Lock { file, keep } => vault::commands::run_lock(&file, keep),
            Commands::Unlock { file } => vault::commands::run_unlock(&file),
            Commands::Vault { action } => vault::commands::run_vault(convert_vault_action(action)),
        }
        return;
    }

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
        interactive::run(config);
    }
}

fn convert_vault_action(cmd: VaultCommands) -> VaultAction {
    match cmd {
        VaultCommands::Init { level } => VaultAction::Init {
            level: if level == "maximum" {
                SecurityLevel::Maximum
            } else {
                SecurityLevel::Standard
            },
        },
        VaultCommands::Add { files } => VaultAction::Add { files },
        VaultCommands::List => VaultAction::List,
        VaultCommands::Extract { name, dest } => VaultAction::Extract { name, dest },
        VaultCommands::Destroy { name } => VaultAction::Destroy { name },
        VaultCommands::Recover => VaultAction::Recover,
    }
}

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
        let rename_mode = RenameMode::from_str(&cli.rename);
        ui::print_info(&format!("Watching folder: {}", source_dir));
        watcher::watch_folder(&source_dir, &config.categories, rename_mode.map(Into::into));
        return;
    }

    let _lock = match lock::FolderLock::acquire(&source_dir) {
        Ok(l) => l,
        Err(e) => {
            ui::print_error(&e);
            return;
        }
    };

    let org_mode = OrganizationMode::from_str(&cli.mode);
    let rename_mode = RenameMode::from_str(&cli.rename);
    let files_map = scan_files(&source_dir, &config.categories, org_mode, cli.recursive);

    if files_map.is_empty() {
        ui::print_info("No files to organize.");
        return;
    }

    if cli.dry_run {
        let preview = organizer::generate_dry_run_preview(
            &source_dir,
            &files_map,
            rename_mode.map(Into::into).as_ref(),
        );
        ui::print_dry_run_preview(&preview);
        ui::print_info("Dry-run complete. No changes were made.");
        return;
    }

    ui::print_preview(&files_map);
    let result = organizer::move_files(&source_dir, &files_map, rename_mode.map(Into::into).as_ref());
    organizer::record_moves(&source_dir, result.moves);
    ui::print_statistics(&result.stats);
    ui::print_success("Files organized successfully!");
}

fn scan_files(
    source_dir: &str,
    categories: &std::collections::HashMap<String, Vec<String>>,
    org_mode: OrganizationMode,
    recursive: bool,
) -> std::collections::HashMap<String, Vec<std::path::PathBuf>> {
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
