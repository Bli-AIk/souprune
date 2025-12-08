use anyhow::{Context, Result};
use serde::Deserialize;
use std::sync::OnceLock;
use std::{fs, path::Path};
use tracing::error;

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

fn read_config_from_disk<P: AsRef<Path>>(path: P) -> Result<SoupruneConfig> {
    let path_ref = path.as_ref();
    let contents = fs::read_to_string(path_ref)
        .with_context(|| format!("Failed to read config file at {}", path_ref.display()))?;

    toml::from_str(&contents)
        .with_context(|| format!("Failed to parse config file at {}", path_ref.display()))
}

pub fn load_config() -> &'static SoupruneConfig {
    CONFIG.get_or_init(|| match read_config_from_disk("projects/config.toml") {
        Ok(config) => config,
        Err(err) => {
            error!(
                "{}
Falling back to default configuration (example_mod)",
                err
            );
            default_config()
        }
    })
}

fn default_config() -> SoupruneConfig {
    SoupruneConfig {
        project: ProjectConfig {
            mod_name: "example_mod".to_string(),
            language: "en-US".to_string(),
        },
        window: WindowConfig {
            resolution_scale: 2,
        },
    }
}
