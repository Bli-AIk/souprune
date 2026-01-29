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
}

impl Default for StateDefinition {
    fn default() -> Self {
        Self {
            ui_interactive: false,
            player_movable: true,
            view_layout: None,
            initial_layer: None,
            on_enter_sound: None,
            on_exit_sound: None,
        }
    }
}

/// Resource that holds the loaded state configuration.
///
/// 保存已加载状态配置的资源。
#[derive(Resource, Default)]
pub struct LoadedStateConfig {
    pub config: Option<StateConfig>,
    pub handle: Option<Handle<StateConfig>>,
}

impl LoadedStateConfig {
    /// Get the configuration for a specific state.
    ///
    /// 获取特定状态的配置。
    pub fn get(&self, state_name: &str) -> Option<&StateDefinition> {
        self.config.as_ref()?.states.get(state_name)
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
}

/// Plugin for state configuration system.
///
/// 状态配置系统的插件。
pub struct StateConfigPlugin;

impl Plugin for StateConfigPlugin {
    fn build(&self, app: &mut App) {
        app.init_asset::<StateConfig>()
            .register_asset_loader(
                crate::core::ron_loader::RonAssetLoader::<StateConfig>::new(&["states.ron"]),
            )
            .init_resource::<LoadedStateConfig>()
            .add_systems(Startup, load_state_config_system)
            .add_systems(Update, process_loaded_state_config_system);
    }
}

/// Load state configuration on startup.
///
/// 启动时加载状态配置。
fn load_state_config_system(
    mut loaded_config: ResMut<LoadedStateConfig>,
    asset_server: Res<AssetServer>,
) {
    let config_path = "config/states.ron";
    info!("Loading state configuration from: {}", config_path);

    let handle: Handle<StateConfig> = asset_server.load(config_path);
    loaded_config.handle = Some(handle);
}

/// Process loaded state configuration.
///
/// 处理已加载的状态配置。
fn process_loaded_state_config_system(
    mut loaded_config: ResMut<LoadedStateConfig>,
    state_configs: Res<Assets<StateConfig>>,
) {
    if loaded_config.config.is_some() {
        return;
    }

    if let Some(handle) = &loaded_config.handle {
        if let Some(config) = state_configs.get(handle) {
            info!("State configuration loaded with {} states", config.states.len());
            for (name, def) in &config.states {
                debug!(
                    "  State '{}': ui_interactive={}, player_movable={}, view_layout={:?}",
                    name, def.ui_interactive, def.player_movable, def.view_layout
                );
            }
            loaded_config.config = Some(config.clone());
        }
    }
}
