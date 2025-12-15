use serde::Deserialize;
use std::collections::HashMap;

#[derive(Deserialize)]
pub struct Config {
    pub protected: Protected,
    pub categories: HashMap<String, Vec<String>>,
}
#[derive(Deserialize)]
pub struct Protected {
    pub system: Vec<String>,
    pub user: Vec<String>,
    pub dev: Vec<String>,
}

pub fn load_config() -> Result<Config, String> {
    let config_path = "./stellar.toml";
    let config = std::fs::read_to_string(config_path)
        .map_err(|e| format!("Failed to read config file: {}", e))?;
    let config: Config = toml::from_str(&config)
        .map_err(|e| format!("Failed to parse config file: {}", e))?;
    Ok(config)
}

pub fn find_category(categories: &HashMap<String, Vec<String>>, ext: &str) -> Option<String> {
    for (category, extensions) in categories {
        if extensions.contains(&ext.to_string()) {
            return Some(category.clone());
        }
    }
    None
}