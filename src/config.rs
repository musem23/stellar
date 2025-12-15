use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::PathBuf;

#[derive(Deserialize, Serialize)]
pub struct Config {
    pub protected: Protected,
    pub categories: HashMap<String, Vec<String>>,
    #[serde(default)]
    pub preferences: Preferences,
}

#[derive(Deserialize, Serialize, Default, Clone)]
pub struct Preferences {
    #[serde(default)]
    pub organization_mode: usize,  // 0 = category, 1 = date
    #[serde(default)]
    pub rename_mode: usize,        // 0 = clean, 1 = date prefix, 2 = skip
}
#[derive(Deserialize, Serialize)]
pub struct Protected {
    pub system: Vec<String>,
    pub user: Vec<String>,
    pub dev: Vec<String>,
}

fn get_config_path() -> PathBuf {
    let home = env::var("HOME").unwrap_or_else(|_| ".".to_string());
    PathBuf::from(home).join(".config").join("stellar").join("stellar.toml")
}

pub fn load_config() -> Result<Config, String> {
    let local_path = PathBuf::from("./stellar.toml");
    let user_path = get_config_path();

    if let Ok(config_str) = fs::read_to_string(&local_path) {
        if let Ok(config) = toml::from_str(&config_str) {
            return Ok(config);
        }
    }

    if let Ok(config_str) = fs::read_to_string(&user_path) {
        if let Ok(config) = toml::from_str(&config_str) {
            return Ok(config);
        }
    }

    Ok(default_config())
}

pub fn save_config(config: &Config) -> Result<(), String> {
    let config_path = get_config_path();

    if let Some(parent) = config_path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create config directory: {}", e))?;
    }

    let toml_str = toml::to_string_pretty(config)
        .map_err(|e| format!("Failed to serialize config: {}", e))?;

    fs::write(&config_path, toml_str)
        .map_err(|e| format!("Failed to write config: {}", e))?;

    Ok(())
}

fn default_config() -> Config {
    let mut categories = HashMap::new();

    categories.insert("Documents".to_string(), vec![
        "pdf", "doc", "docx", "txt", "odt", "rtf", "xls", "xlsx", "ppt", "pptx", "csv"
    ].iter().map(|s| s.to_string()).collect());

    categories.insert("Images".to_string(), vec![
        "png", "jpg", "jpeg", "gif", "webp", "svg", "bmp", "ico", "tiff", "heic"
    ].iter().map(|s| s.to_string()).collect());

    categories.insert("Videos".to_string(), vec![
        "mp4", "mkv", "avi", "mov", "wmv", "flv", "webm", "m4v"
    ].iter().map(|s| s.to_string()).collect());

    categories.insert("Audio".to_string(), vec![
        "mp3", "wav", "flac", "aac", "ogg", "wma", "m4a"
    ].iter().map(|s| s.to_string()).collect());

    categories.insert("Archives".to_string(), vec![
        "zip", "tar", "gz", "rar", "7z", "bz2", "xz"
    ].iter().map(|s| s.to_string()).collect());

    categories.insert("Code".to_string(), vec![
        "rs", "js", "ts", "py", "html", "css", "json", "yaml", "toml", "md", "sh"
    ].iter().map(|s| s.to_string()).collect());

    categories.insert("Fonts".to_string(), vec![
        "ttf", "otf", "woff", "woff2", "eot"
    ].iter().map(|s| s.to_string()).collect());

    categories.insert("Ebooks".to_string(), vec![
        "epub", "mobi", "azw", "azw3", "fb2", "djvu"
    ].iter().map(|s| s.to_string()).collect());

    Config {
        protected: Protected {
            system: vec![
                "/", "/System", "/Library", "/usr", "/bin", "/sbin", "/etc", "/var", "/private", "/Applications"
            ].iter().map(|s| s.to_string()).collect(),
            user: vec![
                ".ssh", ".gnupg", ".config", "Library", "Applications"
            ].iter().map(|s| s.to_string()).collect(),
            dev: vec![
                "node_modules", ".git", ".svn", "target", "venv", "__pycache__", ".cargo"
            ].iter().map(|s| s.to_string()).collect(),
        },
        categories,
        preferences: Preferences::default(),
    }
}

pub fn find_category(categories: &HashMap<String, Vec<String>>, ext: &str) -> Option<String> {
    for (category, extensions) in categories {
        if extensions.contains(&ext.to_string()) {
            return Some(category.clone());
        }
    }
    None
}
