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
use std::fmt;
use std::fs;
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

    /// Touch virtual button binding (e.g., "DPadUp", "ButtonA")
    /// Used for on-screen touch overlay controls on mobile platforms.
    ///
    /// 触屏虚拟按钮绑定（例如 "DPadUp"、"ButtonA"）
    /// 用于移动平台的屏幕触控覆盖层控件。
    Touch(String),
}

/// Touch overlay configuration.
///
/// 触控覆盖层配置。
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct TouchOverlayConfig {
    /// List of platforms where the touch overlay is shown.
    /// Uses OS names from `std::env::consts::OS`: "android", "ios", "linux", "macos", "windows".
    /// If empty or omitted, the touch overlay is disabled on all platforms.
    ///
    /// 显示触控覆盖层的平台列表。
    /// 使用 `std::env::consts::OS` 的系统名："android"、"ios"、"linux"、"macos"、"windows"。
    /// 如果为空或省略，所有平台上均不显示。
    #[serde(default)]
    pub platforms: Vec<String>,

    /// Path to the touch layout configuration file (RON format).
    ///
    /// 触控布局配置文件路径（RON 格式）。
    #[serde(default)]
    pub layout: Option<String>,

    /// Opacity of touch controls (0.0 = transparent, 1.0 = opaque).
    /// If not set, uses the value from the layout RON file.
    ///
    /// 触控控件的透明度（0.0 = 透明，1.0 = 不透明）。
    /// 如果未设置，使用布局 RON 文件中的值。
    #[serde(default)]
    pub opacity: Option<f32>,

    /// Scale factor for touch controls.
    /// If not set, uses the value from the layout RON file.
    ///
    /// 触控控件的缩放系数。
    /// 如果未设置，使用布局 RON 文件中的值。
    #[serde(default)]
    pub scale: Option<f32>,
}

fn default_touch_opacity() -> f32 {
    0.5
}

fn default_touch_scale() -> f32 {
    1.0
}

fn default_mobile_scale() -> f32 {
    0.75
}

/// Screen corner anchor for touch button positioning.
///
/// 触控按钮定位的屏幕锚点。
#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
pub enum TouchAnchor {
    BottomLeft,
    BottomRight,
    TopLeft,
    TopRight,
}

/// Definition of a single touch button in the layout config.
///
/// 布局配置中单个触控按钮的定义。
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TouchButtonDef {
    /// Action name to trigger (must be registered in ActionRegistry).
    /// 要触发的动作名称（必须在 ActionRegistry 中注册）。
    pub action: String,

    /// Texture path for the normal state (relative to mod assets/).
    /// If None, a semi-transparent rectangle with label is used.
    ///
    /// 常态贴图路径（相对于 mod assets/）。
    /// 如果为 None，使用半透明矩形加文字标签。
    #[serde(default)]
    pub texture: Option<String>,

    /// Texture path for the pressed state.
    /// If None, uses a tint of the normal texture.
    ///
    /// 按下状态贴图路径。如果为 None，使用常态贴图的色调变化。
    #[serde(default)]
    pub pressed_texture: Option<String>,

    /// Animation frame textures [idle, pressing, pressed, releasing].
    /// When set, overrides `texture`/`pressed_texture`.
    ///
    /// 动画帧贴图 [空闲, 按下过渡, 按住, 松开过渡]。
    /// 设置后覆盖 `texture`/`pressed_texture`。
    #[serde(default)]
    pub frames: Option<Vec<String>>,

    /// Text label shown on the button (fallback when no texture).
    /// 按钮上显示的文字标签（无贴图时的回退方案）。
    #[serde(default)]
    pub label: Option<String>,

    /// Screen anchor for this button.
    /// 此按钮的屏幕锚点。
    pub anchor: TouchAnchor,

    /// Horizontal offset from the anchor edge (in logical pixels).
    /// 距锚点边缘的水平偏移量（逻辑像素）。
    #[serde(default)]
    pub offset_x: f32,

    /// Vertical offset from the anchor edge (in logical pixels).
    /// 距锚点边缘的垂直偏移量（逻辑像素）。
    #[serde(default)]
    pub offset_y: f32,

    /// Button width (in logical pixels).
    /// 按钮宽度（逻辑像素）。
    #[serde(default = "default_btn_size")]
    pub width: f32,

    /// Button height (in logical pixels).
    /// 按钮高度（逻辑像素）。
    #[serde(default = "default_btn_size")]
    pub height: f32,
}

fn default_btn_size() -> f32 {
    56.0
}

/// Definition of a touch controller (D-pad) with direction overlays.
///
/// 触控控制器（方向键）定义，带方向叠加层。
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TouchControllerDef {
    /// Screen anchor for the controller.
    pub anchor: TouchAnchor,

    /// Horizontal offset from anchor (logical pixels).
    #[serde(default)]
    pub offset_x: f32,

    /// Vertical offset from anchor (logical pixels).
    #[serde(default)]
    pub offset_y: f32,

    /// Controller display size (logical pixels, square).
    #[serde(default = "default_controller_size")]
    pub size: f32,

    /// Base texture (always shown).
    pub base_texture: String,

    /// Direction overlay textures. Keys are action names (e.g., "Up", "Down").
    pub overlays: HashMap<String, String>,
}

fn default_controller_size() -> f32 {
    120.0
}

/// Touch layout definition loaded from RON config.
/// Describes all virtual touch buttons and their layout.
///
/// 从 RON 配置加载的触控布局定义。
/// 描述所有虚拟触控按钮及其布局。
#[derive(Debug, Clone, Deserialize, Serialize, Resource)]
pub struct TouchLayoutDef {
    /// Global opacity for all touch buttons (0.0–1.0).
    /// 所有触控按钮的全局透明度（0.0–1.0）。
    #[serde(default = "default_touch_opacity")]
    pub opacity: f32,

    /// Global scale factor applied to all button sizes.
    /// 应用于所有按钮大小的全局缩放系数。
    #[serde(default = "default_touch_scale")]
    pub scale: f32,

    /// Additional scale factor for mobile platforms (Android/iOS).
    /// Applied on top of the auto-scale calculation.
    /// 移动平台（Android/iOS）的额外缩放系数。在自动缩放基础上应用。
    #[serde(default = "default_mobile_scale")]
    pub mobile_scale: f32,

    /// Optional controller (D-pad) definition.
    #[serde(default)]
    pub controller: Option<TouchControllerDef>,

    /// Button definitions.
    /// 按钮定义。
    pub buttons: Vec<TouchButtonDef>,
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

/// Navigation behavior configuration.
/// Maps direction names to action names.
/// All fields are optional - if not configured, the corresponding functionality is disabled.
///
/// 导航行为配置。
/// 将方向名称映射到动作名称。
/// 所有字段都是可选的 - 如果未配置，对应的功能将被禁用。
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct NavigationConfig {
    /// Action name for moving/navigating up
    /// 向上移动/导航的动作名称
    #[serde(default)]
    pub up: Option<String>,

    /// Action name for moving/navigating down
    /// 向下移动/导航的动作名称
    #[serde(default)]
    pub down: Option<String>,

    /// Action name for moving/navigating left
    /// 向左移动/导航的动作名称
    #[serde(default)]
    pub left: Option<String>,

    /// Action name for moving/navigating right
    /// 向右移动/导航的动作名称
    #[serde(default)]
    pub right: Option<String>,
}

/// UI interaction behavior configuration.
/// Maps UI actions to action names.
/// All fields are optional - if not configured, the corresponding functionality is disabled.
///
/// UI 交互行为配置。
/// 将 UI 动作映射到动作名称。
/// 所有字段都是可选的 - 如果未配置，对应的功能将被禁用。
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct UIConfig {
    /// Action name for confirm/select
    /// 确认/选择的动作名称
    #[serde(default)]
    pub confirm: Option<String>,

    /// Action name for cancel/back
    /// 取消/返回的动作名称
    #[serde(default)]
    pub cancel: Option<String>,

    /// Action name for opening menu
    /// 打开菜单的动作名称
    #[serde(default)]
    pub menu: Option<String>,
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
pub struct InputConfig {
    /// Action definitions with their bindings.
    /// Keys are action names, values are binding lists.
    ///
    /// 动作定义及其绑定。
    /// 键是动作名称，值是绑定列表。
    pub actions: HashMap<String, Vec<InputBinding>>,

    /// Navigation configuration (optional).
    /// If not provided, navigation features will be disabled with a warning.
    ///
    /// 导航配置（可选）。
    /// 如果未提供，导航功能将被禁用并发出警告。
    #[serde(default)]
    pub navigation: NavigationConfig,

    /// UI configuration (optional).
    /// If not provided, UI interaction features will be disabled with a warning.
    ///
    /// UI 配置（可选）。
    /// 如果未提供，UI 交互功能将被禁用并发出警告。
    #[serde(default)]
    pub ui: UIConfig,

    /// Touch overlay configuration (optional).
    /// Enables on-screen virtual controls for touch/mobile platforms.
    ///
    /// 触控覆盖层配置（可选）。
    /// 为触屏/移动平台启用屏幕虚拟控件。
    #[serde(default)]
    pub touch_overlay: Option<TouchOverlayConfig>,
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
        for (name, bindings) in &self.actions {
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
        let config = InputConfig {
            actions: [
                ("Action1".to_string(), vec![]),
                ("Action2".to_string(), vec![]),
            ]
            .into_iter()
            .collect(),
            navigation: NavigationConfig::default(),
            ui: UIConfig::default(),
            touch_overlay: None,
        };

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
