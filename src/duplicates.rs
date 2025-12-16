// Stellar - Duplicate Detection Module
// @musem23
//
// Finds duplicate files by computing SHA-256 hashes.
// Groups files with identical content for user review or batch removal.

use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::PathBuf;

const BUFFER_SIZE: usize = 8192;

pub struct DuplicateGroup {
    pub files: Vec<PathBuf>,
    pub size: u64,
}

/// Find duplicate files by comparing SHA-256 hashes
pub fn find_duplicates(files: &[PathBuf]) -> Vec<DuplicateGroup> {
    let mut by_hash: HashMap<String, (Vec<PathBuf>, u64)> = HashMap::new();

    for path in files {
        if let Ok((hash, size)) = hash_file(path) {
            let entry = by_hash.entry(hash).or_insert_with(|| (Vec::new(), size));
            entry.0.push(path.clone());
        }
    }

    by_hash
        .into_values()
        .filter(|(files, _)| files.len() > 1)
        .map(|(files, size)| DuplicateGroup { files, size })
        .collect()
}

/// Format bytes as human-readable size
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

fn hash_file(path: &PathBuf) -> std::io::Result<(String, u64)> {
    let file = File::open(path)?;
    let size = file.metadata()?.len();

    let mut reader = BufReader::new(file);
    let mut hasher = Sha256::new();
    let mut buffer = [0u8; BUFFER_SIZE];

    loop {
        let n = reader.read(&mut buffer)?;
        if n == 0 {
            break;
        }
        hasher.update(&buffer[..n]);
    }

    Ok((format!("{:x}", hasher.finalize()), size))
}
