//! # config.rs
//!
//! # config.rs 文件
//!
//! ## Module Overview
//!
//! ## 模块概述
//!
//! This module handles loading and accessing the global configuration for the engine. It reads `projects/config.toml` to determine the active project mod, language settings, and window configuration, and provides utilities for resolving asset paths.
//!
//! 本模块处理引擎全局配置的加载与访问。它读取 `projects/config.toml` 以确定当前激活的项目模组、语言设置和窗口配置，并提供了用于解析资产路径的实用工具。

use anyhow::{Context, Result};
use bevy::prelude::Resource;
use serde::Deserialize;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;
use tracing::error;

#[derive(Clone, Deserialize, Resource)]
pub struct SoupruneConfig {
    /// Currently loaded Project (Mod) configuration.
    ///
    /// 当前加载的 Project (Mod) 配置。
    pub project: ProjectConfig,

    /// Window configuration settings.
    ///
    /// 窗口配置设置。
    pub window: WindowConfig,

    /// Game flow configuration settings.
    ///
    /// 游戏流程配置设置。
    #[serde(default)]
    pub game: GameConfig,

    /// Render configuration settings.
    ///
    /// 渲染配置设置。
    #[serde(default)]
    pub render: RenderConfig,
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

/// Game flow configuration for paths and module loading.
///
/// 游戏流程配置，包含路径和模块加载设置。
#[derive(Clone, Deserialize)]
pub struct GameConfig {
    /// Initial map path to load when entering Overworld.
    ///
    /// 进入 Overworld 时加载的初始地图路径。
    #[serde(default = "default_initial_map_path")]
    pub initial_map_path: String,

    /// Debug battle chapter path to load when entering Battle state.
    ///
    /// 进入 Battle 状态时加载的调试用战斗章节路径。
    #[serde(default = "default_initial_battle_path")]
    pub initial_battle_path: String,

    /// Path to player behavior configuration file.
    ///
    /// 玩家行为配置文件路径。
    #[serde(default = "default_player_behavior_path")]
    pub player_behavior_path: String,

    /// Texture modules required before transitioning from AppSetup.
    ///
    /// 从 AppSetup 状态转换前需要加载的纹理模块。
    #[serde(default = "default_required_modules")]
    pub required_modules: Vec<String>,

    /// Keywords for layer names that should be hidden (e.g., prototype, collision).
    ///
    /// 需要隐藏的图层名关键字（如 prototype、collision）。
    #[serde(default = "default_hidden_layer_keywords")]
    pub hidden_layer_keywords: Vec<String>,
}

impl Default for GameConfig {
    fn default() -> Self {
        Self {
            initial_map_path: default_initial_map_path(),
            initial_battle_path: default_initial_battle_path(),
            player_behavior_path: default_player_behavior_path(),
            required_modules: default_required_modules(),
            hidden_layer_keywords: default_hidden_layer_keywords(),
        }
    }
}

fn default_initial_map_path() -> String {
    String::new()
}

fn default_initial_battle_path() -> String {
    String::new()
}

fn default_player_behavior_path() -> String {
    String::new()
}

fn default_required_modules() -> Vec<String> {
    vec!["overworld".to_string(), "common".to_string()]
}

fn default_hidden_layer_keywords() -> Vec<String> {
    vec!["prototype".to_string(), "collision".to_string()]
}

/// Render configuration for resolution and Z-ordering.
///
/// 渲染配置，包含分辨率和 Z 轴排序设置。
#[derive(Clone, Deserialize)]
pub struct RenderConfig {
    /// Base resolution width (game world units).
    ///
    /// 基准分辨率宽度（游戏世界单位）。
    #[serde(default = "default_base_resolution_width")]
    pub base_resolution_width: u32,

    /// Base resolution height (game world units).
    ///
    /// 基准分辨率高度（游戏世界单位）。
    #[serde(default = "default_base_resolution_height")]
    pub base_resolution_height: u32,

    /// Z-offset applied to tilemap layers.
    ///
    /// 应用于 tilemap 图层的 Z 轴偏移。
    #[serde(default = "default_z_layer_tilemap")]
    pub z_layer_tilemap: f32,

    /// Base Z value for layer sorting.
    ///
    /// 图层排序的基准 Z 值。
    #[serde(default = "default_z_layer_base")]
    pub z_layer_base: f32,

    /// Z step between consecutive layers.
    ///
    /// 连续图层之间的 Z 步长。
    #[serde(default = "default_z_layer_step")]
    pub z_layer_step: f32,
}

impl Default for RenderConfig {
    fn default() -> Self {
        Self {
            base_resolution_width: default_base_resolution_width(),
            base_resolution_height: default_base_resolution_height(),
            z_layer_tilemap: default_z_layer_tilemap(),
            z_layer_base: default_z_layer_base(),
            z_layer_step: default_z_layer_step(),
        }
    }
}

fn default_base_resolution_width() -> u32 {
    320
}

fn default_base_resolution_height() -> u32 {
    240
}

fn default_z_layer_tilemap() -> f32 {
    10.0
}

fn default_z_layer_base() -> f32 {
    -2.0
}

fn default_z_layer_step() -> f32 {
    0.5
}

static CONFIG: OnceLock<SoupruneConfig> = OnceLock::new();

pub fn get_asset_roots(mod_name: &str) -> Vec<PathBuf> {
    let mut roots = Vec::new();
    let project_path = Path::new("projects").join(mod_name);

    roots.push(project_path.clone());

    // Fallback to absolute path to ensure assets are found
    if let Ok(abs_path) = dunce::canonicalize(&project_path) {
        roots.push(abs_path);
    }

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

#[derive(Deserialize)]
struct ModConfigFile {
    game: Option<GameConfigPartial>,
}

#[derive(Deserialize)]
struct GameConfigPartial {
    initial_map_path: Option<String>,
    initial_battle_path: Option<String>,
    player_behavior_path: Option<String>,
}

fn read_mod_config<P: AsRef<Path>>(path: P) -> Result<ModConfigFile> {
    let path_ref = path.as_ref();
    let contents = fs::read_to_string(path_ref)
        .with_context(|| format!("Failed to read mod config file at {}", path_ref.display()))?;

    toml::from_str(&contents)
        .with_context(|| format!("Failed to parse mod config file at {}", path_ref.display()))
}

fn read_config_from_disk<P: AsRef<Path>>(path: P) -> Result<SoupruneConfig> {
    let path_ref = path.as_ref();
    let contents = fs::read_to_string(path_ref)
        .with_context(|| format!("Failed to read config file at {}", path_ref.display()))?;

    toml::from_str(&contents)
        .with_context(|| format!("Failed to parse config file at {}", path_ref.display()))
}

pub fn load_config() -> SoupruneConfig {
    CONFIG
        .get_or_init(|| {
            let mut config = read_config_from_disk("projects/config.toml").unwrap_or_else(|err| {
                error!(
                    "{}
Falling back to default configuration (example_mod)",
                    err
                );
                default_config()
            });

            let mod_name = &config.project.mod_name;
            let mod_config_path = Path::new("projects").join(mod_name).join("mod.toml");

            if mod_config_path.exists() {
                match read_mod_config(&mod_config_path) {
                    Ok(mod_cfg) => {
                        if let Some(game_partial) = mod_cfg.game {
                            if let Some(val) = game_partial.initial_map_path {
                                config.game.initial_map_path = val;
                            }
                            if let Some(val) = game_partial.initial_battle_path {
                                config.game.initial_battle_path = val;
                            }
                            if let Some(val) = game_partial.player_behavior_path {
                                config.game.player_behavior_path = val;
                            }
                        }
                    }
                    Err(e) => error!("Failed to load mod.toml: {}", e),
                }
            }

            config
        })
        .clone()
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
        game: GameConfig::default(),
        render: RenderConfig::default(),
    }
}
