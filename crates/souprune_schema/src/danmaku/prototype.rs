//! # Danmaku Prototype Schema
//!
//! Bullet prototype, collider, hit behavior, and tint schema types.
//!
//! # 弹幕原型 Schema
//!
//! 弹幕原型、碰撞体、命中行为与色调 Schema 类型。

use serde::{Deserialize, Serialize};

/// Hit behavior preset.
///
/// 命中行为预设。
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub enum HitBehaviorPreset {
    #[default]
    Default,
    Persistent,
    DamageWhenMoving,
    DamageWhenStationary,
    Custom {
        #[serde(default = "default_true")]
        despawn_on_hit: bool,
        #[serde(default)]
        damage_on_player_moving: bool,
        #[serde(default)]
        damage_on_player_stationary: bool,
        #[serde(default)]
        invincibility_duration: f32,
    },
}

/// Color tint configuration for bullets.
///
/// 弹幕的色调配置。
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct ColorTint {
    #[serde(default)]
    pub hex: String,
    #[serde(default)]
    pub rgba: Option<(f32, f32, f32, f32)>,
}

impl ColorTint {
    /// Create a hex-only tint definition.
    ///
    /// 创建仅使用十六进制字符串的色调定义。
    pub fn hex(hex: impl Into<String>) -> Self {
        Self {
            hex: hex.into(),
            rgba: None,
        }
    }

    /// Create an RGBA tint definition.
    ///
    /// 创建 RGBA 色调定义。
    pub fn rgba(red: f32, green: f32, blue: f32, alpha: f32) -> Self {
        Self {
            hex: String::new(),
            rgba: Some((red, green, blue, alpha)),
        }
    }
}

/// Bullet prototype — appearance and collision definition.
///
/// 弹幕原型 — 外观与碰撞定义。
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct BulletPrototype {
    /// Visual resource path (e.g., "bullet/small").
    ///
    /// 视觉资源路径（如 "bullet/small"）。
    pub visual: String,
    /// Collision shape and size.
    ///
    /// 碰撞形状与尺寸。
    pub collider: ColliderShape,
    /// Damage dealt to the player on hit.
    ///
    /// 命中时对玩家造成的伤害。
    pub damage: f32,
    /// Maximum time the bullet exists in world units (seconds).
    ///
    /// 弹幕在世界中存在的最大时间（秒）。
    pub lifetime: f32,
    /// Visual layering priority.
    ///
    /// 视觉图层优先级。
    pub z_index: f32,
    /// Uniform scale factor.
    ///
    /// 统一缩放比例。
    pub scale: f32,
    /// Initial rotation in radians.
    ///
    /// 初始旋转角度（弧度）。
    #[serde(default)]
    pub rotation: f32,
    /// Preset behavior when hitting the player.
    ///
    /// 命中玩家时的预设行为。
    pub hit_behavior: HitBehaviorPreset,
    /// Color tint applied to the visual.
    ///
    /// 应用于视觉资源的色调。
    pub color_tint: ColorTint,
    /// Whether to flip the visual horizontally.
    ///
    /// 是否水平翻转视觉资源。
    #[serde(default)]
    pub flip_x: bool,
    /// Whether to flip the visual vertically.
    ///
    /// 是否垂直翻转视觉资源。
    #[serde(default)]
    pub flip_y: bool,
    /// Duration of each animation frame (if applicable).
    ///
    /// 每个动画帧的持续时间（如果适用）。
    #[serde(default)]
    pub frame_duration: Option<f32>,
}

impl Default for BulletPrototype {
    fn default() -> Self {
        Self {
            visual: String::new(),
            collider: ColliderShape::default(),
            damage: 1.0,
            lifetime: 5.0,
            z_index: 15.0,
            scale: 1.0,
            rotation: 0.0,
            hit_behavior: HitBehaviorPreset::Default,
            color_tint: ColorTint::default(),
            flip_x: false,
            flip_y: false,
            frame_duration: None,
        }
    }
}

impl BulletPrototype {
    /// Create a bullet prototype with the given visual and schema defaults.
    ///
    /// 使用指定视觉资源和 Schema 默认值创建弹幕原型。
    pub fn new(visual: impl Into<String>) -> Self {
        Self {
            visual: visual.into(),
            ..Default::default()
        }
    }
}

/// Collider shape for hit detection.
///
/// 用于命中检测的碰撞形状。
#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum ColliderShape {
    CircleCollider(f32),
    BoxCollider(f32, f32),
}

impl Default for ColliderShape {
    fn default() -> Self {
        ColliderShape::CircleCollider(4.0)
    }
}

impl ColliderShape {
    /// Create a circular collider.
    ///
    /// 创建圆形碰撞体。
    pub fn circle(radius: f32) -> Self {
        Self::CircleCollider(radius)
    }

    /// Create a rectangular collider.
    ///
    /// 创建矩形碰撞体。
    pub fn rectangle(width: f32, height: f32) -> Self {
        Self::BoxCollider(width, height)
    }
}

fn default_true() -> bool {
    true
}
