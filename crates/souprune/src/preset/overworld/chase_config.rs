//! # chase_config.rs
//!
//! # chase_config.rs 文件
//!
//! Configuration structures for the chase state, loaded from RON files.
//!
//! 追逐战状态的配置结构，从 RON 文件加载。

use bevy::prelude::*;
use serde::Deserialize;
use std::fs;

use crate::config;

// ============================================================================
// Chase configuration structures (loaded from RON file)
// 追逐战配置结构（从 RON 文件加载）
// ============================================================================

/// Configuration for the heart marker effect.
///
/// 心形判定标记效果配置。
#[derive(Debug, Clone, Deserialize)]
pub struct HeartMarkerConfig {
    /// Texture path for the heart marker
    pub texture_path: String,
    /// Offset from player center
    pub offset: Vec2Config,
    /// Z-offset from player
    pub z_offset: f32,
    /// Scale of the heart marker
    #[serde(default = "default_heart_scale")]
    pub scale: f32,
    /// Tint color (RGBA)
    pub color: ColorConfig,
    /// Fade duration in seconds
    pub fade_duration: f32,
}

fn default_heart_scale() -> f32 {
    0.5
}

impl Default for HeartMarkerConfig {
    fn default() -> Self {
        Self {
            // Empty path - requires explicit configuration in chase.ron
            // 空路径 - 需要在 chase.ron 中显式配置
            texture_path: String::new(),
            offset: Vec2Config { x: 0.0, y: -2.0 },
            z_offset: 101.0,
            scale: 0.5,
            color: ColorConfig {
                r: 1.0,
                g: 0.0,
                b: 0.0,
                a: 1.0,
            },
            fade_duration: 0.5,
        }
    }
}

/// Configuration for the outline effect.
///
/// 描边效果配置。
#[derive(Debug, Clone, Deserialize)]
pub struct OutlineConfig {
    /// Outline color (RGB)
    pub color: ColorConfig,
    /// Fade duration in seconds
    pub fade_duration: f32,
    /// Outline padding in pixels (added to sprite size for outline mesh)
    #[serde(default = "default_outline_padding")]
    pub padding: f32,
    /// Z-offset for outline rendering layer
    #[serde(default = "default_outline_z_offset")]
    pub z_offset: f32,
}

fn default_outline_padding() -> f32 {
    2.0
}

fn default_outline_z_offset() -> f32 {
    100.0
}

impl Default for OutlineConfig {
    fn default() -> Self {
        Self {
            color: ColorConfig {
                r: 1.0,
                g: 0.0,
                b: 0.0,
                a: 1.0,
            },
            fade_duration: 0.5,
            padding: default_outline_padding(),
            z_offset: default_outline_z_offset(),
        }
    }
}

/// Configuration for the dark overlay effect.
///
/// 黑色覆盖层效果配置。
#[derive(Debug, Clone, Deserialize)]
pub struct DarkOverlayConfig {
    /// Target alpha value
    pub target_alpha: f32,
    /// Fade duration in seconds
    pub fade_duration: f32,
    /// Size of the overlay quad (should be large enough to cover the screen)
    #[serde(default = "default_overlay_size")]
    pub overlay_size: f32,
    /// Z-offset for overlay rendering layer
    #[serde(default = "default_overlay_z_offset")]
    pub z_offset: f32,
}

fn default_overlay_size() -> f32 {
    10000.0
}

fn default_overlay_z_offset() -> f32 {
    50.0
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

/// Simple Vec2 config for RON deserialization.
///
/// 用于 RON 反序列化的简单 Vec2 配置。
#[derive(Debug, Clone, Deserialize, Default)]
pub struct Vec2Config {
    #[serde(default)]
    pub x: f32,
    #[serde(default)]
    pub y: f32,
}

impl Vec2Config {
    pub fn to_vec2(&self) -> Vec2 {
        Vec2::new(self.x, self.y)
    }
}

/// Simple color config for RON deserialization.
///
/// 用于 RON 反序列化的简单颜色配置。
#[derive(Debug, Clone, Deserialize)]
pub struct ColorConfig {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    #[serde(default = "default_alpha")]
    pub a: f32,
}

fn default_alpha() -> f32 {
    1.0
}

/// Hitbox shape configuration.
///
/// 判定框形状配置。
#[derive(Debug, Clone, Deserialize)]
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
///
/// 用于伤害检测的判定框配置。
#[derive(Debug, Clone, Deserialize)]
pub struct HitboxConfig {
    /// Shape of the hitbox
    pub shape: HitboxShapeConfig,
    /// Offset from player center
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
///
/// 受伤UI配置。
#[derive(Debug, Clone, Deserialize)]
pub struct DamageUIConfig {
    /// View layout file path
    pub layout_path: String,
    /// Display duration in seconds
    pub display_duration: f32,
    /// Sound to play when taking damage (full path, e.g., "assets/audios/sfx/hurtsound.wav")
    /// If None, no sound is played.
    ///
    /// 受伤时播放的音效（完整路径，如 "assets/audios/sfx/hurtsound.wav"）
    /// 如果为 None，则不播放音效。
    #[serde(default)]
    pub damage_sound: Option<String>,
}

impl Default for DamageUIConfig {
    fn default() -> Self {
        Self {
            // Empty path - requires explicit configuration in chase.ron
            // 空路径 - 需要在 chase.ron 中显式配置
            layout_path: String::new(),
            display_duration: 0.5,
            damage_sound: None,
        }
    }
}

/// Complete chase configuration loaded from RON file.
///
/// 从 RON 文件加载的完整追逐战配置。
#[derive(Debug, Clone, Deserialize, Resource, Default)]
pub struct ChaseConfig {
    pub heart_marker: HeartMarkerConfig,
    pub outline: OutlineConfig,
    pub dark_overlay: DarkOverlayConfig,
    #[serde(default)]
    pub hitbox: HitboxConfig,
    #[serde(default)]
    pub damage_ui: DamageUIConfig,
}

impl ChaseConfig {
    /// Load chase configuration from RON file at the given path.
    /// Returns None if the path is None or the file cannot be loaded.
    ///
    /// 从给定路径的 RON 文件加载追逐战配置。
    /// 如果路径为 None 或文件无法加载，则返回 None。
    pub fn load_from_path(chase_config_path: Option<&str>) -> Option<Self> {
        let path_str = chase_config_path?;

        if let Some(path) = config::resolve_path(path_str)
            && let Ok(contents) = fs::read_to_string(&path)
        {
            match ron::de::from_str(&contents) {
                Ok(config) => {
                    info!("Chase: Loaded config from {}", path.display());
                    return Some(config);
                }
                Err(e) => {
                    warn!("Chase: Failed to parse config at {}: {}", path.display(), e);
                    return None;
                }
            }
        }

        warn!("Chase: Config file not found at {}", path_str);
        None
    }

    /// Get transition duration (uses heart marker fade duration as reference).
    ///
    /// 获取过渡持续时间（使用心形标记的渐变持续时间作为参考）。
    pub fn transition_duration(&self) -> f32 {
        self.heart_marker.fade_duration
    }
}

/// Resource indicating whether chase functionality is enabled.
/// This is set based on whether a valid chase_config path exists in mod.toml.
///
/// 指示追逐战功能是否启用的资源。
/// 基于 mod.toml 中是否存在有效的 chase_config 路径来设置。
#[derive(Resource, Default)]
pub struct ChaseEnabled(pub bool);

/// Resource to track the chase state name (configured in flow.ron).
/// This allows the chase system to work with any state name that has chase_config.
#[derive(Resource, Default)]
pub struct ChaseStateName(pub Option<String>);

/// Resource to track if we've entered/exited chase state.
#[derive(Resource, Default)]
pub struct ChaseStateTracker {
    pub was_in_chase: bool,
}
