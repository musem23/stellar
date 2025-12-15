use std::path::Path;
use chrono::{DateTime, Local};

pub enum RenameMode {
    Clean,
    DatePrefix,
}

pub fn rename_file(path: &Path, mode: &RenameMode) -> String {
    let file_name = path.file_stem().unwrap().to_string_lossy().to_string();
    let extension = path
        .extension()
        .map(|e| e.to_string_lossy().to_string())
        .unwrap_or_default();

    let new_name = match mode {
        RenameMode::Clean => clean_name(&file_name),
        RenameMode::DatePrefix => date_prefix_name(path, &file_name),
    };

    if extension.is_empty() {
        new_name
    } else {
        format!("{}.{}", new_name, extension.to_lowercase())
    }
}

fn clean_name(name: &str) -> String {
    let cleaned: String = name
        .chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '-' || c == '_' {
                c.to_ascii_lowercase()
            } else if c == ' ' {
                '-'
            } else {
                '-'
            }
        })
        .collect();

    remove_duplicates(&cleaned)
}

fn remove_duplicates(name: &str) -> String {
    let patterns = ["(1)", "(2)", "(3)", "(4)", "(5)", "(copy)", "-copy", "_copy"];
    let mut result = name.to_string();

    for pattern in patterns {
        result = result.replace(pattern, "");
    }

    result
        .replace("--", "-")
        .replace("--", "-")
        .trim_matches('-')
        .to_string()
}

fn date_prefix_name(path: &Path, name: &str) -> String {
    let date_str = if let Ok(metadata) = path.metadata() {
        if let Ok(modified) = metadata.modified() {
            let datetime: DateTime<Local> = modified.into();
            datetime.format("%Y-%m-%d").to_string()
        } else {
            Local::now().format("%Y-%m-%d").to_string()
        }
    } else {
        Local::now().format("%Y-%m-%d").to_string()
    };

    format!("{}-{}", date_str, clean_name(name))
}
