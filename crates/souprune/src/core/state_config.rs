//! State configuration system for data-driven game state management.
//!
//! 状态配置系统，用于数据驱动的游戏状态管理。
//!
//! This module provides a way to configure UI behavior, view layouts, and other
//! settings for each game state through RON files, enabling modders to customize
//! game state behavior without modifying code.
//!
//! 本模块提供了一种通过 RON 文件配置每个游戏状态下的 UI 行为、视图布局等
//! 设置的方式，使 modder 能够在不修改代码的情况下自定义游戏状态行为。

use bevy::prelude::*;
use serde::Deserialize;
use std::collections::HashMap;

/// State configuration asset loaded from RON files.
///
/// 从 RON 文件加载的状态配置资产。
#[derive(Asset, TypePath, Debug, Deserialize, Clone)]
pub struct StateConfig {
    /// Configuration for each state.
    ///
    /// 每个状态的配置。
    pub states: HashMap<String, StateDefinition>,
}

/// Definition of a single state's configuration.
///
/// 单个状态的配置定义。
#[derive(Debug, Deserialize, Clone)]
pub struct StateDefinition {
    /// Whether UI is interactive in this state.
    ///
    /// 此状态下 UI 是否可交互。
    #[serde(default)]
    pub ui_interactive: bool,

    /// Whether the player can move in this state.
    ///
    /// 此状态下玩家是否可移动。
    #[serde(default)]
    pub player_movable: bool,

    /// Whether the camera should follow the player in this state.
    ///
    /// 此状态下相机是否跟随玩家。
    #[serde(default = "default_true")]
    pub camera_follow_player: bool,

    /// View layout to load when entering this state.
    ///
    /// 进入此状态时加载的视图布局。
    #[serde(default)]
    pub view_layout: Option<String>,

    /// Initial interactive layer to activate.
    ///
    /// 要激活的初始交互层。
    #[serde(default)]
    pub initial_layer: Option<String>,

    /// Sound to play when entering this state.
    ///
    /// 进入此状态时播放的声音。
    #[serde(default)]
    pub on_enter_sound: Option<String>,

    /// Sound to play when exiting this state.
    ///
    /// 退出此状态时播放的声音。
    #[serde(default)]
    pub on_exit_sound: Option<String>,

    /// Optional path to chase configuration (only for chase-like states).
    /// If present, enables chase-specific systems for this state.
    ///
    /// 可选的追逐战配置路径（仅用于类似追逐战的状态）。
    /// 如果存在，则为此状态启用追逐战特定系统。
    #[serde(default)]
    pub chase_config: Option<String>,
}

fn default_true() -> bool {
    true
}

impl Default for StateDefinition {
    fn default() -> Self {
        Self {
            ui_interactive: false,
            player_movable: true,
            camera_follow_player: true,
            view_layout: None,
            initial_layer: None,
            on_enter_sound: None,
            on_exit_sound: None,
            chase_config: None,
        }
    }
}

/// Resource that holds the loaded state configuration.
/// When loaded, provides direct access to StateConfig.
///
/// 保存已加载状态配置的资源。
/// 加载后，提供对 StateConfig 的直接访问。
#[derive(Resource)]
pub struct LoadedStateConfig(pub StateConfig);

impl Default for LoadedStateConfig {
    fn default() -> Self {
        // Default with a Normal state that allows movement
        let mut states = HashMap::new();
        states.insert(
            "Normal".to_string(),
            StateDefinition {
                player_movable: true,
                camera_follow_player: true,
                ..Default::default()
            },
        );
        Self(StateConfig { states })
    }
}

impl LoadedStateConfig {
    /// Get the configuration for a specific state.
    ///
    /// 获取特定状态的配置。
    pub fn get(&self, state_name: &str) -> Option<&StateDefinition> {
        self.0.states.get(state_name)
    }

    /// Check if UI is interactive for the given state.
    ///
    /// 检查给定状态下 UI 是否可交互。
    pub fn is_ui_interactive(&self, state_name: &str) -> bool {
        self.get(state_name)
            .map(|s| s.ui_interactive)
            .unwrap_or(false)
    }

    /// Check if player can move in the given state.
    ///
    /// 检查玩家在给定状态下是否可移动。
    pub fn is_player_movable(&self, state_name: &str) -> bool {
        self.get(state_name)
            .map(|s| s.player_movable)
            .unwrap_or(true)
    }

    /// Check if camera should follow player in the given state.
    ///
    /// 检查给定状态下相机是否跟随玩家。
    pub fn is_camera_follow_player(&self, state_name: &str) -> bool {
        self.get(state_name)
            .map(|s| s.camera_follow_player)
            .unwrap_or(true)
    }

    /// Get the chase config path for the given state (if any).
    /// States with chase_config enable chase-specific behaviors.
    ///
    /// 获取给定状态的追逐战配置路径（如果有）。
    /// 具有 chase_config 的状态启用追逐战特定行为。
    pub fn get_chase_config_path(&self, state_name: &str) -> Option<&str> {
        self.get(state_name).and_then(|s| s.chase_config.as_deref())
    }

    /// Check if the given state is a chase-like state (has chase_config).
    ///
    /// 检查给定状态是否为追逐战类状态（具有 chase_config）。
    pub fn is_chase_state(&self, state_name: &str) -> bool {
        self.get_chase_config_path(state_name).is_some()
    }

    /// Get the view layout path for the given state (if any).
    ///
    /// 获取给定状态的视图布局路径（如果有）。
    pub fn get_view_layout(&self, state_name: &str) -> Option<&str> {
        self.get(state_name).and_then(|s| s.view_layout.as_deref())
    }
}

/// Handle for tracking state config asset loading.
#[derive(Resource, Default)]
pub struct StateConfigHandle(pub Option<Handle<StateConfig>>);

/// Plugin for state configuration system.
///
/// 状态配置系统的插件。
pub struct StateConfigPlugin;

impl Plugin for StateConfigPlugin {
    fn build(&self, app: &mut App) {
        app.init_asset::<StateConfig>()
            .register_asset_loader(crate::core::ron_loader::RonAssetLoader::<StateConfig>::new(
                &["states.ron"],
            ))
            .init_resource::<LoadedStateConfig>()
            .init_resource::<StateConfigHandle>()
            .add_systems(Startup, load_state_config_system)
            .add_systems(Update, process_loaded_state_config_system);
    }
}

/// Load state configuration on startup.
///
/// 启动时加载状态配置。
fn load_state_config_system(
    mut config_handle: ResMut<StateConfigHandle>,
    asset_server: Res<AssetServer>,
    config: Res<crate::config::SoupruneConfig>,
) {
    // Read states_config path from project configuration
    // 从项目配置中读取 states_config 路径
    let config_path = if config.game.states_config.is_empty() {
        "config/states.ron".to_string()
    } else {
        config.game.states_config.clone()
    };

    info!("Loading state configuration from: {}", config_path);

    let handle: Handle<StateConfig> = asset_server.load(config_path);
    config_handle.0 = Some(handle);
}

/// Process loaded state configuration.
///
/// 处理已加载的状态配置。
fn process_loaded_state_config_system(
    mut loaded_config: ResMut<LoadedStateConfig>,
    config_handle: Res<StateConfigHandle>,
    state_configs: Res<Assets<StateConfig>>,
    mut processed: Local<bool>,
) {
    if *processed {
        return;
    }

    if let Some(handle) = &config_handle.0
        && let Some(config) = state_configs.get(handle)
    {
        info!(
            "State configuration loaded with {} states",
            config.states.len()
        );
        for (name, def) in &config.states {
            debug!(
                "  State '{}': ui_interactive={}, player_movable={}, camera_follow={}",
                name, def.ui_interactive, def.player_movable, def.camera_follow_player
            );
        }
        *loaded_config = LoadedStateConfig(config.clone());
        *processed = true;
    }
}
