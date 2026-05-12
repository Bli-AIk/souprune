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
    InputBinding, KeyboardKey, NavigationConfig, TouchAnchor, TouchButtonDef, TouchControllerDef,
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
///         "Up": [Key(ArrowUp), Key(KeyW), Gamepad("DPadUp")],
///         "Down": [Key(ArrowDown), Key(KeyS), Gamepad("DPadDown")],
///         "Left": [Key(ArrowLeft), Key(KeyA), Gamepad("DPadLeft")],
///         "Right": [Key(ArrowRight), Key(KeyD), Gamepad("DPadRight")],
///         "Confirm": [Key(KeyZ), Key(Enter), Gamepad("South")],
///         "Cancel": [Key(KeyX), Key(ShiftLeft), Gamepad("East")],
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
            InputBinding::Key(key) => {
                map.insert(slot, Self::keycode_from_keyboard_key(*key));
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
            let InputBinding::Key(key) = binding else {
                continue;
            };
            map.insert(Self::keycode_from_keyboard_key(*key), name.to_owned());
        }
    }

    /// Convert a schema keyboard key to Bevy's KeyCode.
    ///
    /// 将 schema 键盘按键转换为 Bevy 的 KeyCode。
    fn keycode_from_keyboard_key(key: KeyboardKey) -> KeyCode {
        use KeyboardKey::*;

        match key {
            // Arrow keys
            // 方向键
            ArrowUp => KeyCode::ArrowUp,
            ArrowDown => KeyCode::ArrowDown,
            ArrowLeft => KeyCode::ArrowLeft,
            ArrowRight => KeyCode::ArrowRight,

            // Letter keys
            // 字母键
            KeyA => KeyCode::KeyA,
            KeyB => KeyCode::KeyB,
            KeyC => KeyCode::KeyC,
            KeyD => KeyCode::KeyD,
            KeyE => KeyCode::KeyE,
            KeyF => KeyCode::KeyF,
            KeyG => KeyCode::KeyG,
            KeyH => KeyCode::KeyH,
            KeyI => KeyCode::KeyI,
            KeyJ => KeyCode::KeyJ,
            KeyK => KeyCode::KeyK,
            KeyL => KeyCode::KeyL,
            KeyM => KeyCode::KeyM,
            KeyN => KeyCode::KeyN,
            KeyO => KeyCode::KeyO,
            KeyP => KeyCode::KeyP,
            KeyQ => KeyCode::KeyQ,
            KeyR => KeyCode::KeyR,
            KeyS => KeyCode::KeyS,
            KeyT => KeyCode::KeyT,
            KeyU => KeyCode::KeyU,
            KeyV => KeyCode::KeyV,
            KeyW => KeyCode::KeyW,
            KeyX => KeyCode::KeyX,
            KeyY => KeyCode::KeyY,
            KeyZ => KeyCode::KeyZ,

            // Number keys (main keyboard)
            // 数字键（主键盘）
            Digit0 => KeyCode::Digit0,
            Digit1 => KeyCode::Digit1,
            Digit2 => KeyCode::Digit2,
            Digit3 => KeyCode::Digit3,
            Digit4 => KeyCode::Digit4,
            Digit5 => KeyCode::Digit5,
            Digit6 => KeyCode::Digit6,
            Digit7 => KeyCode::Digit7,
            Digit8 => KeyCode::Digit8,
            Digit9 => KeyCode::Digit9,

            // Numpad keys
            // 小键盘
            Numpad0 => KeyCode::Numpad0,
            Numpad1 => KeyCode::Numpad1,
            Numpad2 => KeyCode::Numpad2,
            Numpad3 => KeyCode::Numpad3,
            Numpad4 => KeyCode::Numpad4,
            Numpad5 => KeyCode::Numpad5,
            Numpad6 => KeyCode::Numpad6,
            Numpad7 => KeyCode::Numpad7,
            Numpad8 => KeyCode::Numpad8,
            Numpad9 => KeyCode::Numpad9,
            NumpadAdd => KeyCode::NumpadAdd,
            NumpadSubtract => KeyCode::NumpadSubtract,
            NumpadMultiply => KeyCode::NumpadMultiply,
            NumpadDivide => KeyCode::NumpadDivide,
            NumpadDecimal => KeyCode::NumpadDecimal,
            NumpadEnter => KeyCode::NumpadEnter,

            // Function keys
            // 功能键
            F1 => KeyCode::F1,
            F2 => KeyCode::F2,
            F3 => KeyCode::F3,
            F4 => KeyCode::F4,
            F5 => KeyCode::F5,
            F6 => KeyCode::F6,
            F7 => KeyCode::F7,
            F8 => KeyCode::F8,
            F9 => KeyCode::F9,
            F10 => KeyCode::F10,
            F11 => KeyCode::F11,
            F12 => KeyCode::F12,

            // Navigation keys
            // 导航键
            Insert => KeyCode::Insert,
            Delete => KeyCode::Delete,
            Home => KeyCode::Home,
            End => KeyCode::End,
            PageUp => KeyCode::PageUp,
            PageDown => KeyCode::PageDown,

            // Special keys
            // 特殊键
            Enter => KeyCode::Enter,
            Escape => KeyCode::Escape,
            Space => KeyCode::Space,
            Tab => KeyCode::Tab,
            Backspace => KeyCode::Backspace,
            CapsLock => KeyCode::CapsLock,
            NumLock => KeyCode::NumLock,
            ScrollLock => KeyCode::ScrollLock,
            Pause => KeyCode::Pause,
            PrintScreen => KeyCode::PrintScreen,

            // Modifier keys
            // 修饰键
            ShiftLeft => KeyCode::ShiftLeft,
            ShiftRight => KeyCode::ShiftRight,
            ControlLeft => KeyCode::ControlLeft,
            ControlRight => KeyCode::ControlRight,
            AltLeft => KeyCode::AltLeft,
            AltRight => KeyCode::AltRight,
            SuperLeft => KeyCode::SuperLeft,
            SuperRight => KeyCode::SuperRight,

            // Punctuation and symbols
            // 标点符号
            Minus => KeyCode::Minus,
            Equal => KeyCode::Equal,
            BracketLeft => KeyCode::BracketLeft,
            BracketRight => KeyCode::BracketRight,
            Backslash => KeyCode::Backslash,
            Semicolon => KeyCode::Semicolon,
            Quote => KeyCode::Quote,
            Backquote => KeyCode::Backquote,
            Comma => KeyCode::Comma,
            Period => KeyCode::Period,
            Slash => KeyCode::Slash,
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
                    "TestUp": [Key(ArrowUp), Key(KeyW)],
                    "TestConfirm": [Key(Enter), Gamepad("South")],
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
    fn test_input_config_rejects_unknown_keyboard_key_variant() {
        let ron = r#"
            #![enable(implicit_some)]
            (
                actions: {
                    "TestUp": [Key(NotARealKey)],
                },
            )
        "#;

        let err = ron::de::from_str::<InputConfig>(ron);
        assert!(err.is_err());
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
    fn test_keyboard_key_to_keycode_mapping() {
        assert_eq!(
            InputConfig::keycode_from_keyboard_key(KeyboardKey::F1),
            KeyCode::F1
        );
        assert_eq!(
            InputConfig::keycode_from_keyboard_key(KeyboardKey::F12),
            KeyCode::F12
        );
        assert_eq!(
            InputConfig::keycode_from_keyboard_key(KeyboardKey::Numpad0),
            KeyCode::Numpad0
        );
        assert_eq!(
            InputConfig::keycode_from_keyboard_key(KeyboardKey::NumpadEnter),
            KeyCode::NumpadEnter
        );
        assert_eq!(
            InputConfig::keycode_from_keyboard_key(KeyboardKey::Insert),
            KeyCode::Insert
        );
        assert_eq!(
            InputConfig::keycode_from_keyboard_key(KeyboardKey::PageDown),
            KeyCode::PageDown
        );
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
