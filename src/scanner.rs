// Stellar - File Scanner Module
// @musem23
//
// Scans directories for files and groups them by category, date, or hybrid mode.
// Supports recursive scanning while respecting project folders and protected paths.
// Detects project folders by common indicators (.git, package.json, Cargo.toml, etc.)

use chrono::{DateTime, Datelike, Local};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use crate::config;

const PROJECT_INDICATORS: &[&str] = &[
    ".git",
    ".svn",
    ".hg",
    "package.json",
    "yarn.lock",
    "pnpm-lock.yaml",
    "Cargo.toml",
    "Cargo.lock",
    "pyproject.toml",
    "setup.py",
    "requirements.txt",
    "Pipfile",
    "Gemfile",
    "go.mod",
    "pom.xml",
    "build.gradle",
    "composer.json",
    "Dockerfile",
];

const PROTECTED_SUBFOLDERS: &[&str] = &[
    "node_modules",
    "target",
    "build",
    "dist",
    "venv",
    "__pycache__",
    ".cargo",
    ".next",
    ".nuxt",
    "vendor",
    "bin",
    "obj",
];

const MONTHS: &[&str] = &[
    "01-january",
    "02-february",
    "03-march",
    "04-april",
    "05-may",
    "06-june",
    "07-july",
    "08-august",
    "09-september",
    "10-october",
    "11-november",
    "12-december",
];

/// Check if a folder contains project indicator files
pub fn is_project_folder(path: &str) -> bool {
    let path = Path::new(path);
    PROJECT_INDICATORS
        .iter()
        .any(|indicator| path.join(indicator).exists())
}

/// Scan files and group by category
pub fn scan_by_category(
    source_dir: &str,
    categories: &HashMap<String, Vec<String>>,
) -> HashMap<String, Vec<PathBuf>> {
    scan_files(source_dir, |_path, ext| {
        config::find_category(categories, ext).unwrap_or_else(|| "Others".into())
    })
}

/// Scan files and group by year/month
pub fn scan_by_date(source_dir: &str) -> HashMap<String, Vec<PathBuf>> {
    scan_files(source_dir, |path, _| get_date_folder(path))
}

/// Scan files and group by category/year (hybrid)
pub fn scan_hybrid(
    source_dir: &str,
    categories: &HashMap<String, Vec<String>>,
) -> HashMap<String, Vec<PathBuf>> {
    scan_files(source_dir, |path, ext| {
        let category = config::find_category(categories, ext).unwrap_or_else(|| "Others".into());
        let year = get_year(path);
        format!("{}/{}", category, year)
    })
}

/// Recursively scan subdirectories (skips project/protected folders)
pub fn scan_recursive(
    source_dir: &str,
    categories: &HashMap<String, Vec<String>>,
    org_mode: usize,
) -> HashMap<String, Vec<PathBuf>> {
    let mut results: HashMap<String, Vec<PathBuf>> = HashMap::new();
    scan_recursive_inner(source_dir, source_dir, categories, org_mode, &mut results);
    results
}

// ============================================================================
// Private helpers
// ============================================================================

fn scan_files<F>(source_dir: &str, get_folder: F) -> HashMap<String, Vec<PathBuf>>
where
    F: Fn(&Path, &str) -> String,
{
    let mut grouped: HashMap<String, Vec<PathBuf>> = HashMap::new();

    let entries = match fs::read_dir(source_dir) {
        Ok(e) => e,
        Err(_) => return grouped,
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }

        if let Some(name) = path.file_name() {
            let name_str = name.to_string_lossy();
            if name_str.starts_with('.')
                || name_str.ends_with(".DS_Store")
                || name_str.ends_with(".localized")
            {
                continue;
            }
        }

        let ext = match path.extension() {
            Some(e) => e.to_string_lossy().to_lowercase(),
            None => continue,
        };

        let folder = get_folder(&path, &ext);
        grouped.entry(folder).or_default().push(path);
    }

    grouped
}

fn scan_recursive_inner(
    root_dir: &str,
    current_dir: &str,
    categories: &HashMap<String, Vec<String>>,
    org_mode: usize,
    results: &mut HashMap<String, Vec<PathBuf>>,
) {
    let entries = match fs::read_dir(current_dir) {
        Ok(e) => e,
        Err(_) => return,
    };

    for entry in entries.flatten() {
        let path = entry.path();

        if path.is_dir() {
            if should_skip_directory(&path, categories) {
                continue;
            }
            scan_recursive_inner(
                root_dir,
                &path.to_string_lossy(),
                categories,
                org_mode,
                results,
            );
        } else if path.is_file() && current_dir != root_dir {
            if let Some(folder) = classify_file(&path, categories, org_mode) {
                results.entry(folder).or_default().push(path);
            }
        }
    }
}

fn should_skip_directory(path: &Path, categories: &HashMap<String, Vec<String>>) -> bool {
    let name = match path.file_name() {
        Some(n) => n.to_string_lossy().to_lowercase(),
        None => return true,
    };

    name.starts_with('.')
        || PROTECTED_SUBFOLDERS.contains(&name.as_str())
        || is_project_folder(&path.to_string_lossy())
        || is_category_folder(&name, categories)
}

fn is_category_folder(name: &str, categories: &HashMap<String, Vec<String>>) -> bool {
    let lower = name.to_lowercase();
    lower == "others" || categories.keys().any(|c| c.to_lowercase() == lower)
}

fn classify_file(
    path: &Path,
    categories: &HashMap<String, Vec<String>>,
    org_mode: usize,
) -> Option<String> {
    let ext = path.extension()?.to_string_lossy().to_lowercase();

    Some(match org_mode {
        0 => config::find_category(categories, &ext).unwrap_or_else(|| "Others".into()),
        1 => get_date_folder(path),
        2 => {
            let cat = config::find_category(categories, &ext).unwrap_or_else(|| "Others".into());
            format!("{}/{}", cat, get_year(path))
        }
        _ => "Others".into(),
    })
}

fn get_file_datetime(path: &Path) -> DateTime<Local> {
    path.metadata()
        .ok()
        .and_then(|m| m.modified().ok())
        .map(|t| t.into())
        .unwrap_or_else(Local::now)
}

fn get_date_folder(path: &Path) -> String {
    let dt = get_file_datetime(path);
    let month_idx = dt.month0() as usize;
    format!("{}/{}", dt.format("%Y"), MONTHS[month_idx])
}

fn get_year(path: &Path) -> String {
    get_file_datetime(path).format("%Y").to_string()
}
