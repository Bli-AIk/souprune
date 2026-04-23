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
//! 定义 `InputConfig`，一种数据驱动的输入配置资源，允许所有键位绑定和动作定义
//! 通过 RON 文件配置。

use super::actions::{Action, ActionRegistry};
use bevy::prelude::*;
use leafwing_input_manager::prelude::*;
use ron::de::from_str;
use serde::{Deserialize, Serialize};
use souprune_schema::config::{
    InputConfig as SchemaInputConfig, TouchLayoutDef as SchemaTouchLayoutDef,
};
use std::collections::HashMap;
use std::fmt;
use std::fs;
use std::ops::{Deref, DerefMut};
use std::path::{Path, PathBuf};

/// Frame transition duration for button press/release animation (seconds).
pub const TOUCH_FRAME_TRANSITION_SECS: f32 = 0.03;

/// Error type for input configuration loading.
///
/// 输入配置加载的错误类型。
#[derive(Debug)]
pub enum ConfigError {
    /// Configuration file not found.
    /// 配置文件未找到。
    FileNotFound(PathBuf),

    /// Failed to read the configuration file.
    /// 读取配置文件失败。
    ReadError(PathBuf, String),

    /// Failed to parse the configuration file.
    /// 解析配置文件失败。
    ParseError(PathBuf, String),
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConfigError::FileNotFound(path) => write!(
                f,
                "Input config file not found: {:?}\n\
                 Actions must be defined in the MOD configuration file.\n\
                 Please create the file with your action definitions.",
                path
            ),
            ConfigError::ReadError(path, err) => {
                write!(f, "Failed to read input config from {:?}: {}", path, err)
            }
            ConfigError::ParseError(path, err) => {
                write!(f, "Failed to parse input config from {:?}: {}", path, err)
            }
        }
    }
}

impl std::error::Error for ConfigError {}

pub use souprune_schema::config::{
    InputBinding, NavigationConfig, TouchAnchor, TouchButtonDef, TouchControllerDef,
    TouchOverlayConfig, UIConfig,
};

/// Touch layout definition loaded from RON config.
/// Describes all virtual touch buttons and their layout.
///
/// 从 RON 配置加载的触控布局定义。
/// 描述所有虚拟触控按钮及其布局。
#[derive(Debug, Clone, Deserialize, Serialize, Resource)]
#[serde(transparent)]
pub struct TouchLayoutDef(pub SchemaTouchLayoutDef);

impl Deref for TouchLayoutDef {
    type Target = SchemaTouchLayoutDef;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for TouchLayoutDef {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl TouchLayoutDef {
    /// Load a TouchLayoutDef from a RON file.
    ///
    /// 从 RON 文件加载 TouchLayoutDef。
    pub fn load_from_file(path: impl AsRef<Path>) -> Result<Self, ConfigError> {
        let path = path.as_ref();
        if !path.exists() {
            return Err(ConfigError::FileNotFound(path.to_path_buf()));
        }
        let contents = fs::read_to_string(path)
            .map_err(|e| ConfigError::ReadError(path.to_path_buf(), e.to_string()))?;
        from_str::<Self>(&contents)
            .map_err(|e| ConfigError::ParseError(path.to_path_buf(), e.to_string()))
    }
}

/// Input configuration asset loaded from RON files.
///
/// 从 RON 文件加载的输入配置资源。
///
/// Example RON format:
/// ```ron
/// (
///     actions: {
///         "Up": [Key("ArrowUp"), Key("KeyW"), Gamepad("DPadUp")],
///         "Down": [Key("ArrowDown"), Key("KeyS"), Gamepad("DPadDown")],
///         "Left": [Key("ArrowLeft"), Key("KeyA"), Gamepad("DPadLeft")],
///         "Right": [Key("ArrowRight"), Key("KeyD"), Gamepad("DPadRight")],
///         "Confirm": [Key("KeyZ"), Key("Enter"), Gamepad("South")],
///         "Cancel": [Key("KeyX"), Key("ShiftLeft"), Gamepad("East")],
///     },
///
///     // Optional: Navigation configuration (flat, not nested)
///     navigation: (
///         up: "Up",
///         down: "Down",
///         left: "Left",
///         right: "Right",
///     ),
///
///     // Optional: UI configuration (flat, not nested)
///     ui: (
///         confirm: "Confirm",
///         cancel: "Cancel",
///     ),
/// )
/// ```
#[derive(Asset, TypePath, Debug, Clone, Deserialize, Serialize)]
#[serde(transparent)]
pub struct InputConfig(pub SchemaInputConfig);

impl Deref for InputConfig {
    type Target = SchemaInputConfig;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for InputConfig {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
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
        Self::try_load_from_file(path.as_ref()).unwrap_or_else(|e| panic!("{}", e))
    }

    /// Try to load InputConfig from a RON file path.
    /// Returns an error if the file doesn't exist or cannot be parsed.
    ///
    /// 尝试从 RON 文件路径加载 InputConfig。
    /// 如果文件不存在或无法解析则返回错误。
    pub fn try_load_from_file(path: impl AsRef<Path>) -> Result<Self, ConfigError> {
        let path = path.as_ref();

        if !path.exists() {
            return Err(ConfigError::FileNotFound(path.to_path_buf()));
        }

        let contents = fs::read_to_string(path)
            .map_err(|e| ConfigError::ReadError(path.to_path_buf(), e.to_string()))?;

        from_str::<Self>(&contents)
            .map_err(|e| ConfigError::ParseError(path.to_path_buf(), e.to_string()))
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
        for action_name in self.0.actions.keys() {
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

        for (action_name, bindings) in &self.0.actions {
            let Some(slot) = registry.get(action_name) else {
                warn!("Unknown action name in input config: {}", action_name);
                continue;
            };

            for binding in bindings {
                self.insert_binding(&mut map, slot, binding);
            }
        }

        map
    }

    /// Insert a single binding into the input map for the given action slot.
    fn insert_binding(&self, map: &mut InputMap<Action>, slot: Action, binding: &InputBinding) {
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
            // Touch bindings are handled by the touch overlay system, not leafwing InputMap.
            InputBinding::Touch(_) => {}
        }
    }

    /// Build a reverse mapping from `KeyCode` to action name.
    ///
    /// 构建从 `KeyCode` 到动作名称的反向映射。
    pub fn build_keycode_to_action_map(&self) -> HashMap<KeyCode, String> {
        let mut map = HashMap::new();
        for (name, bindings) in &self.0.actions {
            Self::collect_key_bindings(name, bindings, &mut map);
        }
        map
    }

    fn collect_key_bindings(
        name: &str,
        bindings: &[InputBinding],
        map: &mut HashMap<KeyCode, String>,
    ) {
        for binding in bindings {
            let InputBinding::Key(key_str) = binding else {
                continue;
            };
            if let Some(kc) = Self::parse_keycode(key_str) {
                map.insert(kc, name.to_owned());
            }
        }
    }

    /// Parse key code string to KeyCode enum.
    ///
    /// 将按键代码字符串解析为 KeyCode 枚举。
    fn parse_keycode(key: &str) -> Option<KeyCode> {
        match key {
            // Arrow keys
            // 方向键
            "ArrowUp" => Some(KeyCode::ArrowUp),
            "ArrowDown" => Some(KeyCode::ArrowDown),
            "ArrowLeft" => Some(KeyCode::ArrowLeft),
            "ArrowRight" => Some(KeyCode::ArrowRight),

            // Letter keys
            // 字母键
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

            // Number keys (main keyboard)
            // 数字键（主键盘）
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

            // Numpad keys
            // 小键盘
            "Numpad0" => Some(KeyCode::Numpad0),
            "Numpad1" => Some(KeyCode::Numpad1),
            "Numpad2" => Some(KeyCode::Numpad2),
            "Numpad3" => Some(KeyCode::Numpad3),
            "Numpad4" => Some(KeyCode::Numpad4),
            "Numpad5" => Some(KeyCode::Numpad5),
            "Numpad6" => Some(KeyCode::Numpad6),
            "Numpad7" => Some(KeyCode::Numpad7),
            "Numpad8" => Some(KeyCode::Numpad8),
            "Numpad9" => Some(KeyCode::Numpad9),
            "NumpadAdd" => Some(KeyCode::NumpadAdd),
            "NumpadSubtract" => Some(KeyCode::NumpadSubtract),
            "NumpadMultiply" => Some(KeyCode::NumpadMultiply),
            "NumpadDivide" => Some(KeyCode::NumpadDivide),
            "NumpadDecimal" => Some(KeyCode::NumpadDecimal),
            "NumpadEnter" => Some(KeyCode::NumpadEnter),

            // Function keys
            // 功能键
            "F1" => Some(KeyCode::F1),
            "F2" => Some(KeyCode::F2),
            "F3" => Some(KeyCode::F3),
            "F4" => Some(KeyCode::F4),
            "F5" => Some(KeyCode::F5),
            "F6" => Some(KeyCode::F6),
            "F7" => Some(KeyCode::F7),
            "F8" => Some(KeyCode::F8),
            "F9" => Some(KeyCode::F9),
            "F10" => Some(KeyCode::F10),
            "F11" => Some(KeyCode::F11),
            "F12" => Some(KeyCode::F12),

            // Navigation keys
            // 导航键
            "Insert" => Some(KeyCode::Insert),
            "Delete" => Some(KeyCode::Delete),
            "Home" => Some(KeyCode::Home),
            "End" => Some(KeyCode::End),
            "PageUp" => Some(KeyCode::PageUp),
            "PageDown" => Some(KeyCode::PageDown),

            // Special keys
            // 特殊键
            "Enter" => Some(KeyCode::Enter),
            "Escape" => Some(KeyCode::Escape),
            "Space" => Some(KeyCode::Space),
            "Tab" => Some(KeyCode::Tab),
            "Backspace" => Some(KeyCode::Backspace),
            "CapsLock" => Some(KeyCode::CapsLock),
            "NumLock" => Some(KeyCode::NumLock),
            "ScrollLock" => Some(KeyCode::ScrollLock),
            "Pause" => Some(KeyCode::Pause),
            "PrintScreen" => Some(KeyCode::PrintScreen),

            // Modifier keys
            // 修饰键
            "ShiftLeft" => Some(KeyCode::ShiftLeft),
            "ShiftRight" => Some(KeyCode::ShiftRight),
            "ControlLeft" => Some(KeyCode::ControlLeft),
            "ControlRight" => Some(KeyCode::ControlRight),
            "AltLeft" => Some(KeyCode::AltLeft),
            "AltRight" => Some(KeyCode::AltRight),
            "SuperLeft" => Some(KeyCode::SuperLeft),
            "SuperRight" => Some(KeyCode::SuperRight),

            // Punctuation and symbols
            // 标点符号
            "Minus" => Some(KeyCode::Minus),
            "Equal" => Some(KeyCode::Equal),
            "BracketLeft" => Some(KeyCode::BracketLeft),
            "BracketRight" => Some(KeyCode::BracketRight),
            "Backslash" => Some(KeyCode::Backslash),
            "Semicolon" => Some(KeyCode::Semicolon),
            "Quote" => Some(KeyCode::Quote),
            "Backquote" => Some(KeyCode::Backquote),
            "Comma" => Some(KeyCode::Comma),
            "Period" => Some(KeyCode::Period),
            "Slash" => Some(KeyCode::Slash),

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_input_config_parsing() {
        let ron = r#"
            #![enable(implicit_some)]
            (
                actions: {
                    "TestUp": [Key("ArrowUp"), Key("KeyW")],
                    "TestConfirm": [Key("Enter"), Gamepad("South")],
                },
                navigation: (
                    up: "TestUp",
                ),
                ui: (
                    confirm: "TestConfirm",
                ),
            )
        "#;

        let config: InputConfig = ron::de::from_str(ron).unwrap();

        // Check actions
        assert!(config.actions.contains_key("TestUp"));
        assert!(config.actions.contains_key("TestConfirm"));
        assert_eq!(config.actions["TestUp"].len(), 2);

        // Check navigation
        assert_eq!(config.navigation.up, Some("TestUp".to_string()));
        assert_eq!(config.navigation.down, None);

        // Check UI
        assert_eq!(config.ui.confirm, Some("TestConfirm".to_string()));
        assert_eq!(config.ui.cancel, None);
    }

    #[test]
    fn test_input_config_build_registry() {
        let config = InputConfig(SchemaInputConfig {
            actions: [
                ("Action1".to_string(), vec![]),
                ("Action2".to_string(), vec![]),
            ]
            .into_iter()
            .collect(),
            navigation: NavigationConfig::default(),
            ui: UIConfig::default(),
            touch_overlay: None,
        });

        let registry = config.build_registry();

        assert!(registry.is_registered("Action1"));
        assert!(registry.is_registered("Action2"));
        assert!(!registry.is_registered("Action3"));
    }

    #[test]
    fn test_parse_keycode_extended() {
        // Function keys
        assert_eq!(InputConfig::parse_keycode("F1"), Some(KeyCode::F1));
        assert_eq!(InputConfig::parse_keycode("F12"), Some(KeyCode::F12));

        // Numpad
        assert_eq!(
            InputConfig::parse_keycode("Numpad0"),
            Some(KeyCode::Numpad0)
        );
        assert_eq!(
            InputConfig::parse_keycode("NumpadEnter"),
            Some(KeyCode::NumpadEnter)
        );

        // Navigation
        assert_eq!(InputConfig::parse_keycode("Insert"), Some(KeyCode::Insert));
        assert_eq!(InputConfig::parse_keycode("Delete"), Some(KeyCode::Delete));
        assert_eq!(InputConfig::parse_keycode("Home"), Some(KeyCode::Home));
        assert_eq!(InputConfig::parse_keycode("End"), Some(KeyCode::End));
        assert_eq!(InputConfig::parse_keycode("PageUp"), Some(KeyCode::PageUp));
        assert_eq!(
            InputConfig::parse_keycode("PageDown"),
            Some(KeyCode::PageDown)
        );

        // Unknown key
        assert_eq!(InputConfig::parse_keycode("UnknownKey"), None);
    }

    #[test]
    fn test_parse_gamepad_button() {
        assert_eq!(
            InputConfig::parse_gamepad_button("South"),
            Some(GamepadButton::South)
        );
        assert_eq!(
            InputConfig::parse_gamepad_button("DPadUp"),
            Some(GamepadButton::DPadUp)
        );
        assert_eq!(InputConfig::parse_gamepad_button("Unknown"), None);
    }

    #[test]
    fn test_config_error_display() {
        let err = ConfigError::FileNotFound("/path/to/file.ron".into());
        let msg = format!("{}", err);
        assert!(msg.contains("not found"));

        let err = ConfigError::ParseError("/path/to/file.ron".into(), "syntax error".to_string());
        let msg = format!("{}", err);
        assert!(msg.contains("parse"));
    }
}
