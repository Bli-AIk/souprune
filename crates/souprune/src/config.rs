use anyhow::{Context, Result};
use serde::Deserialize;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;
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

pub fn get_asset_roots(mod_name: &str) -> Vec<PathBuf> {
    let mut roots = Vec::new();
    let project_path = Path::new("projects").join(mod_name);

    roots.push(project_path.clone());

    #[cfg(feature = "debug")]
    {
        roots.push(
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../../")
                .join(&project_path),
        );
    }

    roots.push(PathBuf::from("assets"));
    roots.push(PathBuf::from("crates/souprune/assets"));

    roots
}

pub fn resolve_path(relative_path: &str) -> Option<PathBuf> {
    let config = load_config();
    let roots = get_asset_roots(&config.project.mod_name);

    for root in roots {
        let candidate = root.join(relative_path);
        if candidate.exists() {
            return Some(candidate);
        }
    }
    None
}

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
