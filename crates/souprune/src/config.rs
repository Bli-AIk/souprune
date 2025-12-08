use serde::Deserialize;
use std::fs;

#[derive(Deserialize)]
pub struct SoupruneConfig {
    pub project: ProjectConfig,
    pub window: WindowConfig,
}

#[derive(Deserialize)]
pub struct ProjectConfig {
    pub mod_name: String,
    pub language: String,
}

#[derive(Deserialize)]
pub struct WindowConfig {
    pub resolution_scale: u32,
}

pub fn load_config() -> SoupruneConfig {
    let config_path = "projects/config.toml";
    let contents = fs::read_to_string(config_path)
        .unwrap_or_else(|e| panic!("Failed to read config file at {}: {}", config_path, e));

    toml::from_str(&contents).expect("Failed to parse config file")
}
