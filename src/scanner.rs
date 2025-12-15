use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use crate::config;

pub fn scan_by_category(
    source_dir: &str,
    categories: &HashMap<String, Vec<String>>,
) -> HashMap<String, Vec<PathBuf>> {
    let mut files_by_category: HashMap<String, Vec<PathBuf>> = HashMap::new();

    for entry in fs::read_dir(source_dir).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();

        if path.is_file() {
            if let Some(ext) = path.extension() {
                let ext = ext.to_string_lossy().to_lowercase();
                let category = config::find_category(categories, &ext)
                    .unwrap_or_else(|| "Others".to_string());
                files_by_category
                    .entry(category)
                    .or_insert_with(Vec::new)
                    .push(path);
            }
        }
    }

    files_by_category
}

pub fn scan_by_date(source_dir: &str) -> HashMap<String, Vec<PathBuf>> {
    let mut files_by_date: HashMap<String, Vec<PathBuf>> = HashMap::new();

    let months = [
        "01-january", "02-february", "03-march", "04-april",
        "05-may", "06-june", "07-july", "08-august",
        "09-september", "10-october", "11-november", "12-december"
    ];

    for entry in fs::read_dir(source_dir).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();

        if path.is_file() {
            if let Ok(metadata) = path.metadata() {
                if let Ok(modified) = metadata.modified() {
                    let datetime: chrono::DateTime<chrono::Local> = modified.into();
                    let year = datetime.format("%Y").to_string();
                    let month_idx = datetime.format("%m").to_string().parse::<usize>().unwrap() - 1;
                    let date_folder = format!("{}/{}", year, months[month_idx]);

                    files_by_date
                        .entry(date_folder)
                        .or_insert_with(Vec::new)
                        .push(path);
                }
            }
        }
    }

    files_by_date
}
