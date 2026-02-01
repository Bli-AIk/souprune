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
use super::config::{InputConfig, NavigationConfig, UIConfig};
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

/// Resource containing behavior configuration loaded from MOD.
/// Provides navigation and UI mappings from action names.
/// All configuration is optional - missing configs will log warnings.
///
/// 包含从 MOD 加载的行为配置的资源。
/// 提供从动作名称到导航和 UI 行为的映射。
/// 所有配置都是可选的 - 缺失的配置会记录警告。
#[derive(Resource, Debug, Clone)]
pub struct InputBehaviorConfig {
    /// Navigation configuration (flat)
    pub navigation: NavigationConfig,
    /// UI configuration (flat)
    pub ui: UIConfig,
}

impl InputBehaviorConfig {
    /// Create from InputConfig's behavior configuration.
    ///
    /// 从 InputConfig 的行为配置创建。
    pub fn from_config(config: &InputConfig) -> Self {
        Self {
            navigation: config.navigation.clone(),
            ui: config.ui.clone(),
        }
    }

    /// Get the action name for navigation up.
    /// 获取向上导航的动作名称。
    pub fn nav_up(&self) -> Option<&str> {
        self.navigation.up.as_deref()
    }

    /// Get the action name for navigation down.
    /// 获取向下导航的动作名称。
    pub fn nav_down(&self) -> Option<&str> {
        self.navigation.down.as_deref()
    }

    /// Get the action name for navigation left.
    /// 获取向左导航的动作名称。
    pub fn nav_left(&self) -> Option<&str> {
        self.navigation.left.as_deref()
    }

    /// Get the action name for navigation right.
    /// 获取向右导航的动作名称。
    pub fn nav_right(&self) -> Option<&str> {
        self.navigation.right.as_deref()
    }

    /// Get the action name for UI confirm.
    /// 获取 UI 确认的动作名称。
    pub fn ui_confirm(&self) -> Option<&str> {
        self.ui.confirm.as_deref()
    }

    /// Get the action name for UI cancel.
    /// 获取 UI 取消的动作名称。
    pub fn ui_cancel(&self) -> Option<&str> {
        self.ui.cancel.as_deref()
    }

    /// Get the action name for opening menu.
    /// 获取打开菜单的动作名称。
    #[allow(dead_code)]
    pub fn ui_menu(&self) -> Option<&str> {
        self.ui.menu.as_deref()
    }

    /// Validate that all referenced actions exist in the registry.
    /// Returns a list of validation errors if any action is not registered.
    ///
    /// 验证所有引用的动作是否存在于注册表中。
    /// 如果有任何动作未注册，则返回验证错误列表。
    pub fn validate(&self, registry: &ActionRegistry) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();

        // Helper to check optional action name
        let check_action = |name: Option<&String>, field: &str, errors: &mut Vec<String>| {
            if let Some(action_name) = name
                && !registry.is_registered(action_name)
            {
                errors.push(format!(
                    "{} action '{}' is not registered in ActionRegistry",
                    field, action_name
                ));
            }
        };

        // Validate navigation actions
        check_action(self.navigation.up.as_ref(), "navigation.up", &mut errors);
        check_action(
            self.navigation.down.as_ref(),
            "navigation.down",
            &mut errors,
        );
        check_action(
            self.navigation.left.as_ref(),
            "navigation.left",
            &mut errors,
        );
        check_action(
            self.navigation.right.as_ref(),
            "navigation.right",
            &mut errors,
        );

        // Validate UI actions
        check_action(self.ui.confirm.as_ref(), "ui.confirm", &mut errors);
        check_action(self.ui.cancel.as_ref(), "ui.cancel", &mut errors);
        check_action(self.ui.menu.as_ref(), "ui.menu", &mut errors);

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}
