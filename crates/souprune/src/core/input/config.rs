//! # config.rs
//!
//! # config.rs 文件
//!
//! ## Module Overview
//!
//! ## 模块概述
//!
//! Defines `InputConfig`, a data-driven input configuration asset that allows
//! all key bindings and action definitions to be configured in RON files.
//!
//! 定义 `InputConfig`，一种数据驱动的输入配置资产，允许所有键位绑定和动作定义
//! 通过 RON 文件配置。

use super::actions::{Action, ActionRegistry};
use bevy::prelude::*;
use leafwing_input_manager::prelude::*;
use ron::de::from_str;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// Input binding types supported by the configuration system.
///
/// 配置系统支持的输入绑定类型。
#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum InputBinding {
    /// Keyboard key binding (e.g., "ArrowUp", "KeyW")
    ///
    /// 键盘按键绑定（例如 "ArrowUp"、"KeyW"）
    Key(String),

    /// Gamepad button binding (e.g., "DPadUp", "South")
    ///
    /// 手柄按钮绑定（例如 "DPadUp"、"South"）
    Gamepad(String),
}

/// Input configuration asset loaded from RON files.
///
/// 从 RON 文件加载的输入配置资产。
///
/// Example RON format (actions directly contain bindings):
/// ```ron
/// (
///     actions: {
///         "Up": [Key("ArrowUp"), Key("KeyW"), Gamepad("DPadUp")],
///         "Down": [Key("ArrowDown"), Key("KeyS"), Gamepad("DPadDown")],
///         "Confirm": [Key("KeyZ"), Key("Enter"), Gamepad("South")],
///         "Sprint": [Key("ShiftLeft"), Gamepad("LeftTrigger")],
///     },
/// )
/// ```
#[derive(Asset, TypePath, Debug, Clone, Deserialize, Serialize)]
pub struct InputConfig {
    /// Action definitions with their bindings.
    /// Keys are action names, values are binding lists.
    ///
    /// 动作定义及其绑定。
    /// 键是动作名称，值是绑定列表。
    pub actions: HashMap<String, Vec<InputBinding>>,
}

impl InputConfig {
    /// Load InputConfig from a RON file path.
    /// Panics if the file doesn't exist or cannot be parsed.
    /// Actions MUST be defined in the MOD configuration file.
    ///
    /// 从 RON 文件路径加载 InputConfig。
    /// 如果文件不存在或无法解析则 panic。
    /// Actions 必须在 MOD 配置文件中定义。
    pub fn load_from_file(path: impl AsRef<Path>) -> Self {
        let path = path.as_ref();

        if !path.exists() {
            panic!(
                "Input config file not found: {:?}\n\
                 Actions must be defined in the MOD configuration file.\n\
                 Please create the file with your action definitions.",
                path
            );
        }

        let contents = fs::read_to_string(path)
            .unwrap_or_else(|e| panic!("Failed to read input config from {:?}: {}", path, e));

        from_str::<Self>(&contents)
            .unwrap_or_else(|e| panic!("Failed to parse input config from {:?}: {}", path, e))
    }

    /// Build an ActionRegistry from this configuration.
    /// Creates a new registry and registers all actions defined in the config.
    ///
    /// 从此配置构建 ActionRegistry。
    /// 创建新的注册表并注册配置中定义的所有动作。
    pub fn build_registry(&self) -> ActionRegistry {
        let mut registry = ActionRegistry::new();

        // Register all actions from the config
        // 从配置中注册所有动作
        for action_name in self.actions.keys() {
            if let Err(e) = registry.register(action_name.clone()) {
                warn!("Failed to register action '{}': {}", action_name, e);
            }
        }

        registry
    }

    /// Build an `InputMap<Action>` from this configuration using the given registry.
    ///
    /// 使用给定的注册表从此配置构建 `InputMap<Action>`。
    pub fn build_input_map(&self, registry: &ActionRegistry) -> InputMap<Action> {
        let mut map = InputMap::default();

        for (action_name, bindings) in &self.actions {
            let Some(slot) = registry.get(action_name) else {
                warn!("Unknown action name in input config: {}", action_name);
                continue;
            };

            for binding in bindings {
                match binding {
                    InputBinding::Key(key_str) => {
                        if let Some(keycode) = Self::parse_keycode(key_str) {
                            map.insert(slot, keycode);
                        } else {
                            warn!("Unknown key code in input config: {}", key_str);
                        }
                    }
                    InputBinding::Gamepad(button_str) => {
                        if let Some(button) = Self::parse_gamepad_button(button_str) {
                            map.insert(slot, button);
                        } else {
                            warn!("Unknown gamepad button in input config: {}", button_str);
                        }
                    }
                }
            }
        }

        map
    }

    /// Parse key code string to KeyCode enum.
    ///
    /// 将按键代码字符串解析为 KeyCode 枚举。
    fn parse_keycode(key: &str) -> Option<KeyCode> {
        match key {
            // Arrow keys
            "ArrowUp" => Some(KeyCode::ArrowUp),
            "ArrowDown" => Some(KeyCode::ArrowDown),
            "ArrowLeft" => Some(KeyCode::ArrowLeft),
            "ArrowRight" => Some(KeyCode::ArrowRight),

            // Letter keys
            "KeyA" => Some(KeyCode::KeyA),
            "KeyB" => Some(KeyCode::KeyB),
            "KeyC" => Some(KeyCode::KeyC),
            "KeyD" => Some(KeyCode::KeyD),
            "KeyE" => Some(KeyCode::KeyE),
            "KeyF" => Some(KeyCode::KeyF),
            "KeyG" => Some(KeyCode::KeyG),
            "KeyH" => Some(KeyCode::KeyH),
            "KeyI" => Some(KeyCode::KeyI),
            "KeyJ" => Some(KeyCode::KeyJ),
            "KeyK" => Some(KeyCode::KeyK),
            "KeyL" => Some(KeyCode::KeyL),
            "KeyM" => Some(KeyCode::KeyM),
            "KeyN" => Some(KeyCode::KeyN),
            "KeyO" => Some(KeyCode::KeyO),
            "KeyP" => Some(KeyCode::KeyP),
            "KeyQ" => Some(KeyCode::KeyQ),
            "KeyR" => Some(KeyCode::KeyR),
            "KeyS" => Some(KeyCode::KeyS),
            "KeyT" => Some(KeyCode::KeyT),
            "KeyU" => Some(KeyCode::KeyU),
            "KeyV" => Some(KeyCode::KeyV),
            "KeyW" => Some(KeyCode::KeyW),
            "KeyX" => Some(KeyCode::KeyX),
            "KeyY" => Some(KeyCode::KeyY),
            "KeyZ" => Some(KeyCode::KeyZ),

            // Number keys
            "Digit0" => Some(KeyCode::Digit0),
            "Digit1" => Some(KeyCode::Digit1),
            "Digit2" => Some(KeyCode::Digit2),
            "Digit3" => Some(KeyCode::Digit3),
            "Digit4" => Some(KeyCode::Digit4),
            "Digit5" => Some(KeyCode::Digit5),
            "Digit6" => Some(KeyCode::Digit6),
            "Digit7" => Some(KeyCode::Digit7),
            "Digit8" => Some(KeyCode::Digit8),
            "Digit9" => Some(KeyCode::Digit9),

            // Special keys
            "Enter" => Some(KeyCode::Enter),
            "Escape" => Some(KeyCode::Escape),
            "Space" => Some(KeyCode::Space),
            "Tab" => Some(KeyCode::Tab),
            "Backspace" => Some(KeyCode::Backspace),

            // Modifier keys
            "ShiftLeft" => Some(KeyCode::ShiftLeft),
            "ShiftRight" => Some(KeyCode::ShiftRight),
            "ControlLeft" => Some(KeyCode::ControlLeft),
            "ControlRight" => Some(KeyCode::ControlRight),
            "AltLeft" => Some(KeyCode::AltLeft),
            "AltRight" => Some(KeyCode::AltRight),

            _ => None,
        }
    }

    /// Parse gamepad button string to GamepadButton enum.
    ///
    /// 将手柄按钮字符串解析为 GamepadButton 枚举。
    fn parse_gamepad_button(button: &str) -> Option<GamepadButton> {
        match button {
            // D-Pad
            "DPadUp" => Some(GamepadButton::DPadUp),
            "DPadDown" => Some(GamepadButton::DPadDown),
            "DPadLeft" => Some(GamepadButton::DPadLeft),
            "DPadRight" => Some(GamepadButton::DPadRight),

            // Face buttons (using position names)
            "South" => Some(GamepadButton::South),
            "East" => Some(GamepadButton::East),
            "West" => Some(GamepadButton::West),
            "North" => Some(GamepadButton::North),

            // Triggers and bumpers
            "LeftTrigger" => Some(GamepadButton::LeftTrigger),
            "RightTrigger" => Some(GamepadButton::RightTrigger),
            "LeftTrigger2" => Some(GamepadButton::LeftTrigger2),
            "RightTrigger2" => Some(GamepadButton::RightTrigger2),

            // Stick buttons
            "LeftThumb" => Some(GamepadButton::LeftThumb),
            "RightThumb" => Some(GamepadButton::RightThumb),

            // Meta buttons
            "Select" => Some(GamepadButton::Select),
            "Start" => Some(GamepadButton::Start),

            _ => None,
        }
    }
}
// Note: No Default implementation for InputConfig.
// Actions MUST be defined in the MOD configuration file (config/input.ron).
// 注意：InputConfig 没有 Default 实现。
// Actions 必须在 MOD 配置文件 (config/input.ron) 中定义。
