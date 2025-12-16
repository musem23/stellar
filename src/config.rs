// Stellar - Configuration Module
// @musem23
//
// Handles loading, saving, and providing default configuration.
// Config is stored in TOML format at ~/.config/stellar/stellar.toml
// or locally in ./stellar.toml (takes precedence).
// Default config is embedded from stellar.toml at compile time.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::{env, fs};

/// Default configuration embedded at compile time
const DEFAULT_CONFIG: &str = include_str!("../stellar.toml");

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
    pub organization_mode: usize,
    #[serde(default)]
    pub rename_mode: usize,
}

#[derive(Deserialize, Serialize)]
pub struct Protected {
    pub system: Vec<String>,
    pub user: Vec<String>,
    pub dev: Vec<String>,
}

/// Load config from local file, user config, or embedded default
pub fn load_config() -> Result<Config, String> {
    let paths = [
        Some(PathBuf::from("./stellar.toml")),
        Some(get_user_config_path()),
    ];

    for path in paths.into_iter().flatten() {
        if let Ok(content) = fs::read_to_string(&path) {
            if let Ok(config) = toml::from_str(&content) {
                return Ok(config);
            }
        }
    }

    toml::from_str(DEFAULT_CONFIG).map_err(|e| format!("Failed to parse default config: {}", e))
}

/// Save config to user config directory
pub fn save_config(config: &Config) -> Result<(), String> {
    let path = get_user_config_path();

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create config directory: {}", e))?;
    }

    let toml_str =
        toml::to_string_pretty(config).map_err(|e| format!("Failed to serialize config: {}", e))?;

    fs::write(&path, toml_str).map_err(|e| format!("Failed to write config: {}", e))?;

    Ok(())
}

/// Find which category an extension belongs to
pub fn find_category(categories: &HashMap<String, Vec<String>>, ext: &str) -> Option<String> {
    let ext_lower = ext.to_lowercase();
    categories
        .iter()
        .find(|(_, exts)| exts.contains(&ext_lower))
        .map(|(name, _)| name.clone())
}

fn get_user_config_path() -> PathBuf {
    let home = env::var("HOME").unwrap_or_else(|_| ".".to_string());
    PathBuf::from(home)
        .join(".config")
        .join("stellar")
        .join("stellar.toml")
}
