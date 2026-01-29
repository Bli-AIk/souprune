//! # config.rs
//!
//! # config.rs 文件
//!
//! ## Module Overview
//!
//! ## 模块概述
//!
//! Defines `InputConfig`, a data-driven input configuration asset that allows
//! all key bindings to be defined in RON files instead of hardcoded in source.
//!
//! 定义 `InputConfig`，一种数据驱动的输入配置资产，允许所有键位绑定通过 RON 文件定义，
//! 而非硬编码在源代码中。

use super::actions::Action;
use bevy::prelude::*;
use leafwing_input_manager::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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
/// Example RON format:
/// ```ron
/// (
///     actions: {
///         "Up": [Key("ArrowUp"), Key("KeyW"), Gamepad("DPadUp")],
///         "Down": [Key("ArrowDown"), Key("KeyS"), Gamepad("DPadDown")],
///         "Confirm": [Key("KeyZ"), Key("Enter"), Gamepad("South")],
///     },
/// )
/// ```
#[derive(Asset, TypePath, Debug, Clone, Deserialize, Serialize)]
pub struct InputConfig {
    /// Mapping from action names to their input bindings.
    ///
    /// 动作名称到输入绑定的映射。
    pub actions: HashMap<String, Vec<InputBinding>>,
}

impl InputConfig {
    /// Build an `InputMap<Action>` from this configuration.
    ///
    /// 从此配置构建 `InputMap<Action>`。
    pub fn build_input_map(&self) -> InputMap<Action> {
        let mut map = InputMap::default();

        for (action_name, bindings) in &self.actions {
            let Some(action) = Self::parse_action(action_name) else {
                warn!("Unknown action name in input config: {}", action_name);
                continue;
            };

            for binding in bindings {
                match binding {
                    InputBinding::Key(key_str) => {
                        if let Some(keycode) = Self::parse_keycode(key_str) {
                            map.insert(action, keycode);
                        } else {
                            warn!("Unknown key code in input config: {}", key_str);
                        }
                    }
                    InputBinding::Gamepad(button_str) => {
                        if let Some(button) = Self::parse_gamepad_button(button_str) {
                            map.insert(action, button);
                        } else {
                            warn!("Unknown gamepad button in input config: {}", button_str);
                        }
                    }
                }
            }
        }

        map
    }

    /// Parse action name string to Action enum.
    ///
    /// 将动作名称字符串解析为 Action 枚举。
    fn parse_action(name: &str) -> Option<Action> {
        match name {
            "Up" => Some(Action::Up),
            "Down" => Some(Action::Down),
            "Left" => Some(Action::Left),
            "Right" => Some(Action::Right),
            "Confirm" => Some(Action::Confirm),
            "Cancel" => Some(Action::Cancel),
            "Menu" => Some(Action::Menu),
            _ => None,
        }
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

impl Default for InputConfig {
    fn default() -> Self {
        let mut actions = HashMap::new();

        // Default bindings matching the existing PlayerInputSettings
        actions.insert(
            "Up".to_string(),
            vec![
                InputBinding::Key("ArrowUp".to_string()),
                InputBinding::Key("KeyW".to_string()),
                InputBinding::Gamepad("DPadUp".to_string()),
            ],
        );
        actions.insert(
            "Down".to_string(),
            vec![
                InputBinding::Key("ArrowDown".to_string()),
                InputBinding::Key("KeyS".to_string()),
                InputBinding::Gamepad("DPadDown".to_string()),
            ],
        );
        actions.insert(
            "Left".to_string(),
            vec![
                InputBinding::Key("ArrowLeft".to_string()),
                InputBinding::Key("KeyA".to_string()),
                InputBinding::Gamepad("DPadLeft".to_string()),
            ],
        );
        actions.insert(
            "Right".to_string(),
            vec![
                InputBinding::Key("ArrowRight".to_string()),
                InputBinding::Key("KeyD".to_string()),
                InputBinding::Gamepad("DPadRight".to_string()),
            ],
        );
        actions.insert(
            "Confirm".to_string(),
            vec![
                InputBinding::Key("KeyZ".to_string()),
                InputBinding::Key("Enter".to_string()),
                InputBinding::Gamepad("South".to_string()),
            ],
        );
        actions.insert(
            "Cancel".to_string(),
            vec![
                InputBinding::Key("KeyX".to_string()),
                InputBinding::Key("ShiftLeft".to_string()),
                InputBinding::Key("ShiftRight".to_string()),
                InputBinding::Gamepad("East".to_string()),
            ],
        );
        actions.insert(
            "Menu".to_string(),
            vec![
                InputBinding::Key("KeyC".to_string()),
                InputBinding::Key("ControlLeft".to_string()),
                InputBinding::Key("ControlRight".to_string()),
                InputBinding::Gamepad("North".to_string()),
            ],
        );

        Self { actions }
    }
}
