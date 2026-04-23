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
use souprune_schema::config::{StateConfig as SchemaStateConfig, StateDefinition};
use std::collections::HashMap;

/// State configuration asset loaded from RON files.
///
/// 从 RON 文件加载的状态配置资源。
#[derive(Asset, TypePath, Debug, Deserialize, Clone)]
#[serde(transparent)]
pub struct StateConfig(pub SchemaStateConfig);

/// Resource that holds the loaded state configuration.
/// When loaded, provides direct access to StateConfig.
///
/// 保存已加载状态配置的资源。
/// 加载后，提供对 StateConfig 的直接访问。
#[derive(Resource)]
pub struct LoadedStateConfig(pub SchemaStateConfig);

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
        Self(SchemaStateConfig { states })
    }
}

impl LoadedStateConfig {
    /// Get the configuration for a specific state.
    ///
    /// 获取特定状态的配置。
    pub fn get(&self, state_name: &str) -> Option<&StateDefinition> {
        self.0.states.get(state_name)
    }

    /// Check if player can interact in the given state.
    ///
    /// 检查玩家在给定状态下是否可以交互。
    pub fn can_interact(&self, state_name: &str) -> bool {
        self.get(state_name)
            .map(state_definition_can_interact)
            .unwrap_or(true)
    }

    /// Check if UI is interactive for the given state.
    ///
    /// 检查给定状态下 UI 是否可交互。
    pub fn is_view_interactive(&self, state_name: &str) -> bool {
        self.get(state_name)
            .map(|s| s.view_interactive)
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

    /// Iterate over all configured states.
    ///
    /// 遍历所有状态配置。
    pub fn iter(&self) -> impl Iterator<Item = (&String, &StateDefinition)> {
        self.0.states.iter()
    }

    /// Collect all configured state names.
    ///
    /// 收集全部状态名称。
    pub fn state_names(&self) -> Vec<&String> {
        self.0.states.keys().collect()
    }
}

/// Handle for tracking state config asset loading.
#[derive(Resource, Default)]
pub struct StateConfigHandle(pub Option<Handle<StateConfig>>);

/// Resource indicating whether state configuration has been loaded from file.
/// This prevents chase system from using default values.
///
/// 指示状态配置是否已从文件加载的资源。
/// 这可以防止追逐战系统使用默认值。
#[derive(Resource, Default)]
pub struct StateConfigLoaded(pub bool);

/// Plugin for state configuration system.
///
/// 状态配置系统的插件。
pub struct StateConfigPlugin;

impl Plugin for StateConfigPlugin {
    fn build(&self, app: &mut App) {
        let schedule = crate::game_schedule(app);
        app.init_asset::<StateConfig>()
            .register_asset_loader(crate::core::ron_loader::RonAssetLoader::<StateConfig>::new(
                &["flow.ron"],
            ))
            .init_resource::<LoadedStateConfig>()
            .init_resource::<StateConfigHandle>()
            .init_resource::<StateConfigLoaded>()
            .add_systems(Startup, load_state_config_system)
            .add_systems(schedule, process_loaded_state_config_system);
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
    // Read flow_config_path from project configuration
    // 从项目配置中读取 flow_config_path 路径
    let config_path = if config.game.flow_config_path.is_empty() {
        "app/flow.ron".to_string()
    } else {
        config.game.flow_config_path.clone()
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
    mut config_loaded: ResMut<StateConfigLoaded>,
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
            config.0.states.len()
        );
        for (name, def) in &config.0.states {
            debug!(
                "  State '{}': view_interactive={}, player_movable={}, camera_follow={}",
                name, def.view_interactive, def.player_movable, def.camera_follow_player
            );
        }
        *loaded_config = LoadedStateConfig(config.0.clone());
        config_loaded.0 = true;
        *processed = true;
    }
}

pub fn state_definition_can_interact(def: &StateDefinition) -> bool {
    def.player_can_interact.unwrap_or(def.player_movable)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_runtime_wrapper_and_preserves_state_behavior() {
        let ron = r#"(
            states: {
                "Menu": (
                    view_interactive: true,
                    player_movable: false,
                    player_can_interact: Some(true),
                    camera_follow_player: false,
                    view_layout: Some("ui/menu.view.ron"),
                ),
            },
        )"#;

        let config: StateConfig = ron::from_str(ron).expect("state config should parse");
        let loaded = LoadedStateConfig(config.0.clone());

        assert!(loaded.is_view_interactive("Menu"));
        assert!(!loaded.is_player_movable("Menu"));
        assert!(loaded.can_interact("Menu"));
        assert_eq!(loaded.get_view_layout("Menu"), Some("ui/menu.view.ron"));
    }
}
