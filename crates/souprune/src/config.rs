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

    /// Resource paths configuration.
    ///
    /// 资源路径配置。
    #[serde(skip)]
    pub resources: ResourcePaths,
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
#[serde(default)]
pub struct GameConfig {
    /// Path to the global rules file (loaded into FRE Global layer at startup).
    /// This contains initial player data and game-wide facts.
    ///
    /// 全局规则文件路径（启动时加载到 FRE 全局层）。
    /// 包含初始玩家数据和游戏全局事实。
    pub global_rules: String,

    /// Initial map path to load when entering Overworld.
    /// If empty and `initial_battle_path` is set, the game will start in Battle mode.
    ///
    /// 进入 Overworld 时加载的初始地图路径。
    /// 如果为空且设置了 `initial_battle_path`，游戏将以 Battle 模式启动。
    pub initial_map_path: String,

    /// Debug battle chapter path to load when entering Battle state.
    ///
    /// 进入 Battle 状态时加载的调试用战斗章节路径。
    pub initial_battle_path: String,

    /// Path to player behavior configuration file.
    ///
    /// 玩家行为配置文件路径。
    pub player_behavior_path: String,

    /// Path to input configuration file (RON format).
    ///
    /// 输入配置文件路径（RON 格式）。
    pub input_config_path: String,

    /// Path to state configuration file (RON format).
    ///
    /// 状态配置文件路径（RON 格式）。
    pub states_config: String,

    /// Path to chase state configuration file (RON format).
    /// If None, chase state functionality is disabled.
    ///
    /// 追逐战状态配置文件路径（RON 格式）。
    /// 如果为 None，则禁用追逐战功能。
    pub chase_config: Option<String>,

    /// Default dialogue view layout path for Tiled objects.
    /// Can be overridden per-object via `dialogue_view` property.
    ///
    /// Tiled 对象的默认对话视图布局路径。
    /// 可通过 `dialogue_view` 属性覆盖。
    pub dialogue_view_default: String,

    /// Texture modules required before transitioning from AppSetup.
    ///
    /// 从 AppSetup 状态转换前需要加载的纹理模块。
    pub required_modules: Vec<String>,

    /// Keywords for layer names that should be hidden (e.g., prototype, collision).
    ///
    /// 需要隐藏的图层名关键字（如 prototype、collision）。
    pub hidden_layer_keywords: Vec<String>,
}

impl Default for GameConfig {
    fn default() -> Self {
        Self {
            global_rules: String::new(),
            initial_map_path: String::new(),
            initial_battle_path: String::new(),
            player_behavior_path: String::new(),
            input_config_path: String::new(),
            states_config: "config/states.ron".to_string(),
            chase_config: None,
            dialogue_view_default: "states/overworld/view/dialogue.view.ron".to_string(),
            required_modules: vec!["overworld".to_string(), "common".to_string()],
            hidden_layer_keywords: vec!["prototype".to_string(), "collision".to_string()],
        }
    }
}

/// Render configuration for resolution and Z-ordering.
///
/// 渲染配置，包含分辨率和 Z 轴排序设置。
#[derive(Clone, Deserialize)]
#[serde(default)]
pub struct RenderConfig {
    /// Base resolution width (game world units).
    ///
    /// 基准分辨率宽度（游戏世界单位）。
    pub base_resolution_width: u32,

    /// Base resolution height (game world units).
    ///
    /// 基准分辨率高度（游戏世界单位）。
    pub base_resolution_height: u32,

    /// Z-offset applied to tilemap layers.
    ///
    /// 应用于 tilemap 图层的 Z 轴偏移。
    pub z_layer_tilemap: f32,

    /// Base Z value for layer sorting.
    ///
    /// 图层排序的基准 Z 值。
    pub z_layer_base: f32,

    /// Z step between consecutive layers.
    ///
    /// 连续图层之间的 Z 步长。
    pub z_layer_step: f32,
}

impl Default for RenderConfig {
    fn default() -> Self {
        Self {
            base_resolution_width: 320,
            base_resolution_height: 240,
            z_layer_tilemap: 10.0,
            z_layer_base: -2.0,
            z_layer_step: 0.5,
        }
    }
}

static CONFIG: OnceLock<SoupruneConfig> = OnceLock::new();

pub fn get_asset_roots(mod_name: &str) -> Vec<PathBuf> {
    let mut roots = Vec::new();
    let project_path = Path::new("projects").join(mod_name);

    // Primary: project's assets directory
    // 主要：项目的 assets 目录
    roots.push(project_path.join("assets"));

    // Also check the project root as a fallback location
    // 同时检查项目根目录作为备选位置
    roots.push(project_path.clone());

    // Fallback to absolute path to ensure assets are found
    // 回退到绝对路径以确保找到资源
    if let Ok(abs_path) = dunce::canonicalize(&project_path) {
        roots.push(abs_path.join("assets"));
        roots.push(abs_path);
    }

    #[cfg(feature = "debug")]
    {
        roots.push(
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../../")
                .join(&project_path)
                .join("assets"),
        );
        roots.push(
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../../")
                .join(&project_path),
        );
    }

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

/// Resource paths configuration from mod.toml [resources] section.
///
/// mod.toml 中 [resources] 节的资源路径配置。
#[derive(Clone, Deserialize)]
#[serde(default)]
#[derive(Default)]
pub struct ResourcePaths {
    /// Path to textures directory relative to mod root.
    ///
    /// 纹理目录路径，相对于 mod 根目录。
    pub textures: String,

    /// Path to audio directory relative to mod root.
    ///
    /// 音频目录路径，相对于 mod 根目录。
    pub audios: String,
}

#[derive(Deserialize)]
struct ModConfigFile {
    game: Option<GameConfigPartial>,
    #[serde(default)]
    resources: Option<ResourcePathsPartial>,
}

#[derive(Deserialize, Default)]
struct ResourcePathsPartial {
    textures: Option<String>,
    audios: Option<String>,
}

#[derive(Deserialize)]
struct GameConfigPartial {
    global_rules: Option<String>,
    initial_map_path: Option<String>,
    initial_battle_path: Option<String>,
    player_behavior_path: Option<String>,
    input_config_path: Option<String>,
    states_config: Option<String>,
    chase_config: Option<String>,
    required_modules: Option<Vec<String>>,
    hidden_layer_keywords: Option<Vec<String>>,
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

            // Initialize resources - will be populated from mod.toml
            config.resources = ResourcePaths::default();

            let mod_name = &config.project.mod_name;
            let mod_config_path = Path::new("projects").join(mod_name).join("mod.toml");

            if mod_config_path.exists() {
                match read_mod_config(&mod_config_path) {
                    Ok(mod_cfg) => {
                        if let Some(game_partial) = mod_cfg.game {
                            if let Some(val) = game_partial.global_rules {
                                config.game.global_rules = val;
                            }
                            if let Some(val) = game_partial.initial_map_path {
                                config.game.initial_map_path = val;
                            }
                            if let Some(val) = game_partial.initial_battle_path {
                                config.game.initial_battle_path = val;
                            }
                            if let Some(val) = game_partial.player_behavior_path {
                                config.game.player_behavior_path = val;
                            }
                            if let Some(val) = game_partial.input_config_path {
                                config.game.input_config_path = val;
                            }
                            if let Some(val) = game_partial.states_config {
                                config.game.states_config = val;
                            }
                            if let Some(val) = game_partial.chase_config {
                                config.game.chase_config = Some(val);
                            }
                            if let Some(val) = game_partial.required_modules {
                                config.game.required_modules = val;
                            }
                            if let Some(val) = game_partial.hidden_layer_keywords {
                                config.game.hidden_layer_keywords = val;
                            }
                        }
                        // Load resource paths from [resources] section (required)
                        if let Some(res_partial) = mod_cfg.resources {
                            if let Some(val) = res_partial.textures {
                                config.resources.textures = val;
                            }
                            if let Some(val) = res_partial.audios {
                                config.resources.audios = val;
                            }
                        }

                        // Validate required resource paths
                        if config.resources.textures.is_empty() {
                            error!("mod.toml: [resources].textures is required");
                        }
                        if config.resources.audios.is_empty() {
                            error!("mod.toml: [resources].audios is required");
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
        resources: ResourcePaths::default(),
    }
}
