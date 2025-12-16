# Changelog

All notable changes to Stellar will be documented in this file.

## [1.0.0-beta.2] - 2025-12-15

### Added
- **Custom path input** - Paste any folder path (e.g., ~/Desktop, /path/to/folder)
- **Undo cleanup** - Empty folders are removed when undoing operations
- **Post-action menu** - "Back to menu" / "Exit" options after Undo and History

### Changed
- **Rust-themed banner** - Random yellow-orange colors on each start
- **Folder selection** - New FolderChoice enum for flexible selection

---

## [1.0.0-beta.1] - 2024-12-15

### Added
- **CLI mode** - Direct folder argument (`stellar ~/Downloads`)
- **3 Organization modes** - Category, Date, and Hybrid (category/year)
- **Smart renaming** - Unicode/accent support (élève → eleve)
- **Duplicate detection** - SHA-256 based duplicate finder
- **Undo functionality** - Revert last operation
- **History** - View last 10 operations
- **Recursive scan** - Organize subdirectories with `-R` flag
- **Dry-run mode** - Preview changes before applying
- **Progress bar** - Visual feedback during operations
- **Statistics** - Summary after organization (files moved, bytes, duration)
- **Watch mode** - Auto-organize new files with graceful Ctrl+C handling
- **Folder locking** - Prevent multiple instances on same folder
- **Project detection** - Skip folders with .git, package.json, Cargo.toml, etc.
- **Protected folders** - Refuse to organize system/dev directories

### Configuration
- Config file at `~/.config/stellar/stellar.toml`
- Customizable categories
- Saved preferences for organization and rename modes

### Categories (default)
- Documents, Images, Videos, Audio, Archives, Code, Executables, Fonts, Ebooks

### CLI Options
- `-m, --mode` - Organization mode (category, date, hybrid)
- `-r, --rename` - Rename mode (clean, date-prefix, skip)
- `-R, --recursive` - Include subdirectories
- `-d, --dry-run` - Preview only
- `-w, --watch` - Watch mode
