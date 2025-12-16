// Stellar - Statistics Module
// @musem23
//
// Tracks organization statistics: files moved, renamed, skipped, bytes processed.
// Provides dry-run preview structures and formatting utilities for sizes and durations.

use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Instant;

// ============================================================================
// Organization Statistics
// ============================================================================

#[derive(Default)]
pub struct OrganizationStats {
    pub files_moved: usize,
    pub files_renamed: usize,
    pub files_skipped: usize,
    pub duplicates_found: usize,
    pub total_bytes: u64,
    pub categories: HashMap<String, usize>,
    pub duration_ms: u64,
    start_time: Option<Instant>,
}

impl OrganizationStats {
    pub fn new() -> Self {
        Self {
            start_time: Some(Instant::now()),
            ..Default::default()
        }
    }

    pub fn finish(&mut self) {
        if let Some(start) = self.start_time {
            self.duration_ms = start.elapsed().as_millis() as u64;
        }
    }

    pub fn add_file(&mut self, category: &str, size: u64) {
        self.files_moved += 1;
        self.total_bytes += size;
        *self.categories.entry(category.to_string()).or_insert(0) += 1;
    }

    pub fn add_renamed(&mut self) {
        self.files_renamed += 1;
    }

    pub fn add_skipped(&mut self) {
        self.files_skipped += 1;
    }
}

// ============================================================================
// Dry-Run Preview
// ============================================================================

pub struct DryRunPreview {
    pub moves: Vec<PreviewMove>,
    pub total_files: usize,
    pub total_bytes: u64,
}

pub struct PreviewMove {
    pub from: PathBuf,
    pub to: PathBuf,
    pub is_rename: bool,
}

impl DryRunPreview {
    pub fn new() -> Self {
        Self {
            moves: Vec::new(),
            total_files: 0,
            total_bytes: 0,
        }
    }

    pub fn add_move(&mut self, from: PathBuf, to: PathBuf, size: u64, is_rename: bool) {
        self.moves.push(PreviewMove {
            from,
            to,
            is_rename,
        });
        self.total_files += 1;
        self.total_bytes += size;
    }
}

// ============================================================================
// Formatting Utilities
// ============================================================================

pub fn format_size(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    match bytes {
        b if b >= GB => format!("{:.2} GB", b as f64 / GB as f64),
        b if b >= MB => format!("{:.2} MB", b as f64 / MB as f64),
        b if b >= KB => format!("{:.2} KB", b as f64 / KB as f64),
        b => format!("{} B", b),
    }
}

pub fn format_duration(ms: u64) -> String {
    match ms {
        m if m >= 60000 => format!("{:.1} min", m as f64 / 60000.0),
        m if m >= 1000 => format!("{:.1} s", m as f64 / 1000.0),
        m => format!("{} ms", m),
    }
}
