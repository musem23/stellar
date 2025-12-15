use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use crate::renamer::{self, RenameMode};

pub fn move_files(
    source_dir: &str,
    files_map: &HashMap<String, Vec<PathBuf>>,
    rename_mode: Option<&RenameMode>,
) {
    for (folder_name, files) in files_map {
        let dest_dir = format!("{}/{}", source_dir, folder_name);
        fs::create_dir_all(&dest_dir).unwrap();

        for file_path in files {
            let new_name = match rename_mode {
                Some(mode) => renamer::rename_file(file_path, mode),
                None => file_path.file_name().unwrap().to_string_lossy().to_string(),
            };

            let dest_path = format!("{}/{}", dest_dir, new_name);
            let dest_path = handle_conflict(&dest_path);
            fs::rename(file_path, &dest_path).unwrap();
        }
    }
}

fn handle_conflict(path: &str) -> String {
    if !std::path::Path::new(path).exists() {
        return path.to_string();
    }

    let path_obj = std::path::Path::new(path);
    let stem = path_obj.file_stem().unwrap().to_string_lossy();
    let ext = path_obj
        .extension()
        .map(|e| format!(".{}", e.to_string_lossy()))
        .unwrap_or_default();
    let parent = path_obj.parent().unwrap().to_string_lossy();

    let mut counter = 1;
    loop {
        let new_path = format!("{}/{}-{}{}", parent, stem, counter, ext);
        if !std::path::Path::new(&new_path).exists() {
            return new_path;
        }
        counter += 1;
    }
}

pub fn open_folder(path: &str) {
    std::process::Command::new("open")
        .arg(path)
        .spawn()
        .unwrap();
}
