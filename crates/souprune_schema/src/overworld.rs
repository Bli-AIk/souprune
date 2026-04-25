//! # overworld.rs
//!
//! Overworld configuration schema types for `player_behavior.ron` and `chase_config.ron`.
//! Mirrors overworld config types without Bevy dependency.
//!
//! Overworld 配置 Schema 类型。

use serde::{Deserialize, Serialize};

// ============================================================================
// Shared helper types
// ============================================================================

/// Simple Vec2 config for RON deserialization.
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct Vec2Config {
    #[serde(default)]
    pub x: f32,
    #[serde(default)]
    pub y: f32,
}

/// Simple RGBA color config.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ColorConfig {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    #[serde(default = "default_alpha")]
    pub a: f32,
}

impl Default for ColorConfig {
    fn default() -> Self {
        Self {
            r: 1.0,
            g: 0.0,
            b: 0.0,
            a: 1.0,
        }
    }
}

/// Direction enum for facing.
#[derive(Default, Clone, Copy, Serialize, Deserialize, Debug, PartialEq, Eq)]
pub enum Direction {
    Up,
    #[default]
    Down,
    Left,
    Right,
    UpLeft,
    UpRight,
    DownLeft,
    DownRight,
}

// ============================================================================
// Player Behavior (player_behavior.ron)
// ============================================================================

/// Run action configuration.
///
/// 跑动动作配置。
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RunConfig {
    /// Identifier for the input action that triggers running.
    ///
    /// 触发跑动的输入动作标识符。
    pub action: String,
    /// Movement speed multiplier when running.
    ///
    /// 跑动时的移动速度倍率。
    #[serde(default = "default_run_speed_multiplier")]
    pub speed_multiplier: f32,
}

/// File-level invincibility configuration for overworld.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct OverworldInvincibilityConfig {
    #[serde(default = "default_invincibility_duration")]
    pub duration: f32,
    #[serde(default = "default_flash_interval")]
    pub flash_interval: f32,
    #[serde(default = "default_normal_color_hex")]
    pub normal_color: String,
    #[serde(default = "default_flash_color_hex")]
    pub flash_color: String,
}

impl Default for OverworldInvincibilityConfig {
    fn default() -> Self {
        Self {
            duration: default_invincibility_duration(),
            flash_interval: default_flash_interval(),
            normal_color: default_normal_color_hex(),
            flash_color: default_flash_color_hex(),
        }
    }
}

/// Player behavior file — top-level `player_behavior.ron` schema.
///
/// 玩家行为文件 — `player_behavior.ron` 的顶层 Schema。
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PlayerBehaviorFile {
    /// Path to the `.character.ron` file for the player.
    ///
    /// 玩家的 `.character.ron` 文件路径。
    pub character_asset: String,
    /// Initial spawn position in world coordinates.
    ///
    /// 初始生成的世界坐标位置。
    #[serde(default)]
    pub spawn_position: Vec2Config,
    /// Initial direction the player is facing.
    ///
    /// 玩家初始面向的方向。
    #[serde(default)]
    pub initial_facing: Direction,
    /// Initial animation state (e.g., "Idle").
    ///
    /// 初始动画状态（如 "Idle"）。
    #[serde(default = "default_initial_state")]
    pub initial_state: String,
    /// Running configuration.
    ///
    /// 跑动配置。
    #[serde(default)]
    pub run: Option<RunConfig>,
    /// Invincibility effect settings for overworld.
    ///
    /// Overworld 中的无敌效果设置。
    #[serde(default)]
    pub invincibility: Option<OverworldInvincibilityConfig>,
}

// ============================================================================
// Chase Configuration (chase_config.ron)
// ============================================================================

/// Heart marker effect configuration.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct HeartMarkerConfig {
    pub texture_path: String,
    pub offset: Vec2Config,
    pub z_offset: f32,
    #[serde(default = "default_heart_scale")]
    pub scale: f32,
    pub color: ColorConfig,
    pub fade_duration: f32,
}

impl Default for HeartMarkerConfig {
    fn default() -> Self {
        Self {
            texture_path: String::new(),
            offset: Vec2Config { x: 0.0, y: -2.0 },
            z_offset: 101.0,
            scale: 0.5,
            color: ColorConfig::default(),
            fade_duration: 0.5,
        }
    }
}

/// Outline effect configuration.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct OutlineConfig {
    pub color: ColorConfig,
    pub fade_duration: f32,
    #[serde(default = "default_outline_padding")]
    pub padding: f32,
    #[serde(default = "default_outline_z_offset")]
    pub z_offset: f32,
}

impl Default for OutlineConfig {
    fn default() -> Self {
        Self {
            color: ColorConfig::default(),
            fade_duration: 0.5,
            padding: default_outline_padding(),
            z_offset: default_outline_z_offset(),
        }
    }
}

/// Dark overlay effect configuration.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DarkOverlayConfig {
    pub target_alpha: f32,
    pub fade_duration: f32,
    #[serde(default = "default_overlay_size")]
    pub overlay_size: f32,
    #[serde(default = "default_overlay_z_offset")]
    pub z_offset: f32,
}

impl Default for DarkOverlayConfig {
    fn default() -> Self {
        Self {
            target_alpha: 0.5,
            fade_duration: 0.5,
            overlay_size: default_overlay_size(),
            z_offset: default_overlay_z_offset(),
        }
    }
}

/// Hitbox shape configuration.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum HitboxShapeConfig {
    Circle { radius: f32 },
    Box { half_width: f32, half_height: f32 },
}

impl Default for HitboxShapeConfig {
    fn default() -> Self {
        HitboxShapeConfig::Circle { radius: 4.0 }
    }
}

/// Hitbox configuration for damage detection.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct HitboxConfig {
    pub shape: HitboxShapeConfig,
    #[serde(default)]
    pub offset: Vec2Config,
}

impl Default for HitboxConfig {
    fn default() -> Self {
        Self {
            shape: HitboxShapeConfig::default(),
            offset: Vec2Config { x: 0.0, y: -5.0 },
        }
    }
}

/// Damage UI configuration.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DamageUIConfig {
    pub layout_path: String,
    pub display_duration: f32,
    #[serde(default)]
    pub damage_sound: Option<String>,
}

impl Default for DamageUIConfig {
    fn default() -> Self {
        Self {
            layout_path: String::new(),
            display_duration: 0.5,
            damage_sound: None,
        }
    }
}

/// Chase configuration — top-level `chase_config.ron` schema.
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct ChaseConfig {
    pub heart_marker: HeartMarkerConfig,
    pub outline: OutlineConfig,
    pub dark_overlay: DarkOverlayConfig,
    #[serde(default)]
    pub hitbox: HitboxConfig,
    #[serde(default)]
    pub damage_ui: DamageUIConfig,
}

// ============================================================================
// Default helpers
// ============================================================================

fn default_alpha() -> f32 {
    1.0
}

fn default_heart_scale() -> f32 {
    0.5
}

fn default_outline_padding() -> f32 {
    2.0
}

fn default_outline_z_offset() -> f32 {
    100.0
}

fn default_overlay_size() -> f32 {
    10000.0
}

fn default_overlay_z_offset() -> f32 {
    50.0
}

fn default_run_speed_multiplier() -> f32 {
    1.5
}

fn default_invincibility_duration() -> f32 {
    1.0
}

fn default_flash_interval() -> f32 {
    0.25
}

fn default_normal_color_hex() -> String {
    "#FF0000".to_string()
}

fn default_flash_color_hex() -> String {
    "#800000".to_string()
}

fn default_initial_state() -> String {
    "idle".to_string()
}
