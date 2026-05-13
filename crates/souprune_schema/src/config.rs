//! # config.rs
//!
//! Configuration schema types for various RON config files:
//! - `input.ron` — InputConfig
//! - `flow.ron` — StateConfig
//! - `touch_layout.ron` — TouchLayoutDef
//! - `alight_motion_config.ron` — AlightMotionBattleConfig
//!
//! 各种 RON 配置文件的 Schema 类型。

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ============================================================================
// Input Configuration (input.ron)
// ============================================================================

/// Keyboard key used by an input binding.
///
/// 输入绑定使用的键盘按键。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
pub enum KeyboardKey {
    ArrowUp,
    ArrowDown,
    ArrowLeft,
    ArrowRight,
    KeyA,
    KeyB,
    KeyC,
    KeyD,
    KeyE,
    KeyF,
    KeyG,
    KeyH,
    KeyI,
    KeyJ,
    KeyK,
    KeyL,
    KeyM,
    KeyN,
    KeyO,
    KeyP,
    KeyQ,
    KeyR,
    KeyS,
    KeyT,
    KeyU,
    KeyV,
    KeyW,
    KeyX,
    KeyY,
    KeyZ,
    Digit0,
    Digit1,
    Digit2,
    Digit3,
    Digit4,
    Digit5,
    Digit6,
    Digit7,
    Digit8,
    Digit9,
    Numpad0,
    Numpad1,
    Numpad2,
    Numpad3,
    Numpad4,
    Numpad5,
    Numpad6,
    Numpad7,
    Numpad8,
    Numpad9,
    NumpadAdd,
    NumpadSubtract,
    NumpadMultiply,
    NumpadDivide,
    NumpadDecimal,
    NumpadEnter,
    F1,
    F2,
    F3,
    F4,
    F5,
    F6,
    F7,
    F8,
    F9,
    F10,
    F11,
    F12,
    Insert,
    Delete,
    Home,
    End,
    PageUp,
    PageDown,
    Enter,
    Escape,
    Space,
    Tab,
    Backspace,
    CapsLock,
    NumLock,
    ScrollLock,
    Pause,
    PrintScreen,
    ShiftLeft,
    ShiftRight,
    ControlLeft,
    ControlRight,
    AltLeft,
    AltRight,
    SuperLeft,
    SuperRight,
    Minus,
    Equal,
    BracketLeft,
    BracketRight,
    Backslash,
    Semicolon,
    Quote,
    Backquote,
    Comma,
    Period,
    Slash,
}

/// Input binding.
///
/// 输入绑定。
#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum InputBinding {
    Key(KeyboardKey),
    Gamepad(String),
    Touch(String),
}

/// Navigation behavior configuration.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct NavigationConfig {
    #[serde(default)]
    pub up: Option<String>,
    #[serde(default)]
    pub down: Option<String>,
    #[serde(default)]
    pub left: Option<String>,
    #[serde(default)]
    pub right: Option<String>,
}

/// UI interaction behavior configuration.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct UIConfig {
    #[serde(default)]
    pub confirm: Option<String>,
    #[serde(default)]
    pub cancel: Option<String>,
    #[serde(default)]
    pub menu: Option<String>,
}

/// Touch overlay configuration.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct TouchOverlayConfig {
    #[serde(default)]
    pub platforms: Vec<String>,
    #[serde(default)]
    pub layout: Option<String>,
    #[serde(default)]
    pub opacity: Option<f32>,
    #[serde(default)]
    pub scale: Option<f32>,
}

/// Input configuration - top-level `input.ron` schema.
///
/// 输入配置 - 顶层 `input.ron` schema。
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct InputConfig {
    #[serde(serialize_with = "crate::ordered_map::serialize_ordered_map")]
    pub actions: HashMap<String, Vec<InputBinding>>,
    #[serde(default)]
    pub navigation: NavigationConfig,
    #[serde(default)]
    pub ui: UIConfig,
    #[serde(default)]
    pub touch_overlay: Option<TouchOverlayConfig>,
}

// ============================================================================
// State Configuration (flow.ron)
// ============================================================================

/// Screen-space fact projection owned by a state.
///
/// 由状态拥有的屏幕空间 fact 投影配置。
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ScreenFactProjectionDef {
    /// Fact key receiving the player screen-space X coordinate.
    ///
    /// 接收玩家屏幕空间 X 坐标的 fact key。
    #[serde(default)]
    pub player_x_fact: Option<String>,
    /// Fact key receiving the player screen-space Y coordinate.
    ///
    /// 接收玩家屏幕空间 Y 坐标的 fact key。
    #[serde(default)]
    pub player_y_fact: Option<String>,
    /// FRE event emitted after projection facts are refreshed.
    ///
    /// 投影 facts 刷新后发出的 FRE 事件。
    #[serde(default)]
    pub updated_event: Option<String>,
}

/// Definition of a single state's configuration.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct StateDefinition {
    #[serde(default)]
    pub view_interactive: bool,
    #[serde(default)]
    pub player_movable: bool,
    #[serde(default)]
    pub player_can_interact: Option<bool>,
    #[serde(default = "default_true")]
    pub camera_follow_player: bool,
    #[serde(default)]
    pub view_layout: Option<String>,
    /// FRE events to apply before spawning this state's view.
    ///
    /// 生成此状态的 View 前要先应用的 FRE 事件。
    #[serde(default)]
    pub pre_spawn_events: Vec<String>,
    /// Screen-space facts to synchronize while this state is active.
    ///
    /// 当前状态激活时需要同步的屏幕空间 facts。
    #[serde(default)]
    pub screen_fact_projection: Option<ScreenFactProjectionDef>,
    #[serde(default)]
    pub initial_layer: Option<String>,
    #[serde(default)]
    pub on_enter_sound: Option<String>,
    #[serde(default)]
    pub on_exit_sound: Option<String>,
    #[serde(default)]
    pub chase_config: Option<String>,
}

impl Default for StateDefinition {
    fn default() -> Self {
        Self {
            view_interactive: false,
            player_movable: true,
            player_can_interact: None,
            camera_follow_player: true,
            view_layout: None,
            pre_spawn_events: Vec::new(),
            screen_fact_projection: None,
            initial_layer: None,
            on_enter_sound: None,
            on_exit_sound: None,
            chase_config: None,
        }
    }
}

/// State configuration — top-level `flow.ron` schema.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct StateConfig {
    #[serde(serialize_with = "crate::ordered_map::serialize_ordered_map")]
    pub states: HashMap<String, StateDefinition>,
}

// ============================================================================
// Touch Layout Configuration (touch_layout.ron)
// ============================================================================

/// Screen corner anchor for touch button positioning.
#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
pub enum TouchAnchor {
    BottomLeft,
    BottomRight,
    TopLeft,
    TopRight,
}

/// Definition of a single touch button.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TouchButtonDef {
    pub action: String,
    #[serde(default)]
    pub texture: Option<String>,
    #[serde(default)]
    pub pressed_texture: Option<String>,
    #[serde(default)]
    pub frames: Option<Vec<String>>,
    #[serde(default)]
    pub label: Option<String>,
    pub anchor: TouchAnchor,
    #[serde(default)]
    pub offset_x: f32,
    #[serde(default)]
    pub offset_y: f32,
    #[serde(default = "default_btn_size")]
    pub width: f32,
    #[serde(default = "default_btn_size")]
    pub height: f32,
}

/// Definition of a touch controller (D-pad).
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TouchControllerDef {
    pub anchor: TouchAnchor,
    #[serde(default)]
    pub offset_x: f32,
    #[serde(default)]
    pub offset_y: f32,
    #[serde(default = "default_controller_size")]
    pub size: f32,
    pub base_texture: String,
    #[serde(serialize_with = "crate::ordered_map::serialize_ordered_map")]
    pub overlays: HashMap<String, String>,
}

/// Touch layout definition — top-level `touch_layout.ron` schema.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TouchLayoutDef {
    #[serde(default = "default_touch_opacity")]
    pub opacity: f32,
    #[serde(default = "default_touch_scale")]
    pub scale: f32,
    #[serde(default = "default_mobile_scale")]
    pub mobile_scale: f32,
    #[serde(default)]
    pub controller: Option<TouchControllerDef>,
    pub buttons: Vec<TouchButtonDef>,
}

// ============================================================================
// AM Battle Configuration (alight_motion_config.ron)
// ============================================================================

/// AM battle configuration — top-level `alight_motion_config.ron` schema.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct AlightMotionBattleConfig {
    pub scale: f32,
    pub offset: (f32, f32),
    pub bullet_pattern: String,
    pub battle_box_pattern: String,
    pub hidden_pattern: String,
    pub bullet_damage: f32,
    pub collision_scale: f32,
    #[serde(default = "default_battle_box_size")]
    pub default_battle_box_size: (f32, f32),
}

impl Default for AlightMotionBattleConfig {
    fn default() -> Self {
        Self {
            scale: 1.0,
            offset: (0.0, 0.0),
            bullet_pattern: "^#B".to_string(),
            battle_box_pattern: "^#C".to_string(),
            hidden_pattern: String::new(),
            bullet_damage: 1.0,
            collision_scale: 0.05,
            default_battle_box_size: default_battle_box_size(),
        }
    }
}

// ============================================================================
// Default helpers
// ============================================================================

fn default_true() -> bool {
    true
}

fn default_btn_size() -> f32 {
    56.0
}

fn default_controller_size() -> f32 {
    120.0
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

fn default_battle_box_size() -> (f32, f32) {
    (566.0, 130.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn touch_layout_defaults_match_runtime_expectations() {
        let ron = r#"(
            buttons: [(
                action: "Confirm",
                anchor: BottomRight,
            )],
        )"#;

        let layout: TouchLayoutDef = ron::from_str(ron).expect("touch layout should parse");
        let button = &layout.buttons[0];

        assert!((layout.opacity - 0.5).abs() < f32::EPSILON);
        assert!((layout.scale - 1.0).abs() < f32::EPSILON);
        assert!((layout.mobile_scale - 0.75).abs() < f32::EPSILON);
        assert!((button.width - 56.0).abs() < f32::EPSILON);
        assert!((button.height - 56.0).abs() < f32::EPSILON);
    }
}
