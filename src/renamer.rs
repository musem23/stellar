// Stellar - File Renamer Module
// @musem23
//
// Renames files using different strategies:
// - Clean: lowercase, dashes, remove accents and duplicates (élève → eleve)
// - DatePrefix: prepend modification date (2024-01-15-filename)
// Uses Unicode normalization (NFD) to handle accented characters.

use chrono::{DateTime, Local};
use std::path::Path;
use unicode_normalization::UnicodeNormalization;

pub enum RenameMode {
    Clean,
    DatePrefix,
}

/// Rename a file according to the specified mode
pub fn rename_file(path: &Path, mode: &RenameMode) -> String {
    let stem = path
        .file_stem()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_default();

    let ext = path.extension().map(|e| e.to_string_lossy().to_lowercase());

    let new_stem = match mode {
        RenameMode::Clean => slugify(&stem),
        RenameMode::DatePrefix => {
            let date = get_file_date(path);
            format!("{}-{}", date, slugify(&stem))
        }
    };

    match ext {
        Some(e) => format!("{}.{}", new_stem, e),
        None => new_stem,
    }
}

/// Convert text to a clean, URL-friendly slug
/// Handles accents: élève café → eleve-cafe
fn slugify(text: &str) -> String {
    // NFD normalization separates base characters from accents
    let normalized: String = text.nfd().collect();

    let mut result = String::with_capacity(text.len());
    let mut prev_dash = true; // Start true to avoid leading dash

    for c in normalized.chars() {
        if c.is_ascii_alphanumeric() {
            result.push(c.to_ascii_lowercase());
            prev_dash = false;
        } else if matches!(c, ' ' | '_' | '-' | '.') && !prev_dash {
            result.push('-');
            prev_dash = true;
        }
        // Skip accents (combining diacritical marks) and other chars
    }

    // Remove duplicate suffixes and trailing dashes
    remove_copy_suffixes(&result)
}

/// Remove common duplicate suffixes like (1), -copy, (copie)
fn remove_copy_suffixes(name: &str) -> String {
    const PATTERNS: &[&str] = &[
        "-1", "-2", "-3", "-4", "-5", "-6", "-7", "-8", "-9", "-copy", "-copie",
    ];

    let mut result = name.to_string();
    for pattern in PATTERNS {
        if result.ends_with(pattern) {
            result.truncate(result.len() - pattern.len());
        }
    }

    // Collapse multiple dashes and trim
    while result.contains("--") {
        result = result.replace("--", "-");
    }

    result.trim_matches('-').to_string()
}

fn get_file_date(path: &Path) -> String {
    path.metadata()
        .ok()
        .and_then(|m| m.modified().ok())
        .map(|t| {
            let dt: DateTime<Local> = t.into();
            dt.format("%Y-%m-%d").to_string()
        })
        .unwrap_or_else(|| Local::now().format("%Y-%m-%d").to_string())
}
