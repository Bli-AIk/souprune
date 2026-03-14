//! # config.rs
//!
//! Configuration schema types for various RON config files:
//! - `input.ron` — InputConfig
//! - `states.ron` — StateConfig
//! - `touch_layout.ron` — TouchLayoutDef
//! - `alight_motion_config.ron` — AlightMotionBattleConfig
//!
//! 各种 RON 配置文件的 Schema 类型。

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ============================================================================
// Input Configuration (input.ron)
// ============================================================================

/// Input binding.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum InputBinding {
    Key(String),
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

/// Input configuration — top-level `input.ron` schema.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct InputConfig {
    pub actions: HashMap<String, Vec<InputBinding>>,
    #[serde(default)]
    pub navigation: NavigationConfig,
    #[serde(default)]
    pub ui: UIConfig,
    #[serde(default)]
    pub touch_overlay: Option<TouchOverlayConfig>,
}

// ============================================================================
// State Configuration (states.ron)
// ============================================================================

/// Definition of a single state's configuration.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct StateDefinition {
    #[serde(default)]
    #[serde(alias = "ui_interactive")]
    pub view_interactive: bool,
    #[serde(default)]
    pub player_movable: bool,
    #[serde(default)]
    pub player_can_interact: Option<bool>,
    #[serde(default = "default_true")]
    pub camera_follow_player: bool,
    #[serde(default)]
    pub view_layout: Option<String>,
    #[serde(default)]
    pub initial_layer: Option<String>,
    #[serde(default)]
    pub on_enter_sound: Option<String>,
    #[serde(default)]
    pub on_exit_sound: Option<String>,
    #[serde(default)]
    pub chase_config: Option<String>,
}

/// State configuration — top-level `states.ron` schema.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct StateConfig {
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
    64.0
}

fn default_controller_size() -> f32 {
    128.0
}

fn default_touch_opacity() -> f32 {
    0.7
}

fn default_touch_scale() -> f32 {
    1.0
}

fn default_mobile_scale() -> f32 {
    1.0
}

fn default_battle_box_size() -> (f32, f32) {
    (566.0, 130.0)
}
