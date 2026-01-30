//! # resources.rs
//!
//! # resources.rs 文件
//!
//! ## Module Overview
//!
//! ## 模块概述
//!
//! Defines `PlayerInputSettings`, a resource that manages input mappings (keyboard, gamepad) for player actions, supporting multiple control schemes.
//!
//! 定义 `PlayerInputSettings`，该资源管理玩家动作的输入映射（键盘、手柄），支持多种控制方案。

use super::actions::{Action, ActionRegistry};
use super::config::InputConfig;
use bevy::prelude::*;
use leafwing_input_manager::prelude::*;

#[derive(Resource)]
pub(crate) struct PlayerInputSettings {
    maps: Vec<InputMap<Action>>,
}

impl PlayerInputSettings {
    /// Create new settings from an InputConfig and ActionRegistry.
    ///
    /// 从 InputConfig 和 ActionRegistry 创建新设置。
    pub fn from_config(config: &InputConfig, registry: &ActionRegistry) -> Self {
        Self {
            maps: vec![config.build_input_map(registry)],
        }
    }

    #[allow(dead_code)]
    pub fn get_map(&self, index: usize) -> Option<&InputMap<Action>> {
        self.maps.get(index)
    }

    pub fn get_merged_map(&self) -> InputMap<Action> {
        let mut merged = InputMap::default();

        for map in &self.maps {
            merged.merge(map);
        }

        merged
    }
}

// Note: No Default implementation for PlayerInputSettings.
// It must be created from InputConfig and ActionRegistry loaded from MOD configuration.
// 注意：PlayerInputSettings 没有 Default 实现。
// 它必须从 MOD 配置加载的 InputConfig 和 ActionRegistry 创建。
