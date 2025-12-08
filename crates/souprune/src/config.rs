use serde::Deserialize;
use std::sync::OnceLock;
use std::{fs, path::Path};

#[derive(Clone, Deserialize)]
pub struct SoupruneConfig {
    pub project: ProjectConfig,
    pub window: WindowConfig,
}

#[derive(Clone, Deserialize)]
pub struct ProjectConfig {
    pub mod_name: String,
    pub language: String,
}

#[derive(Clone, Deserialize)]
pub struct WindowConfig {
    pub resolution_scale: u32,
}

static CONFIG: OnceLock<SoupruneConfig> = OnceLock::new();

fn read_config_from_disk<P: AsRef<Path>>(path: P) -> SoupruneConfig {
    let path_ref = path.as_ref();
    let contents = fs::read_to_string(path_ref).unwrap_or_else(|e| {
        panic!(
            "Failed to read config file at {}: {}",
            path_ref.display(),
            e
        )
    });

    toml::from_str(&contents).expect("Failed to parse config file")
}

pub fn load_config() -> &'static SoupruneConfig {
    CONFIG.get_or_init(|| read_config_from_disk("projects/config.toml"))
}
