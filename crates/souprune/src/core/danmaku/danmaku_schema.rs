//! # danmaku_schema.rs
//!
//! # danmaku_schema.rs 文件
//!
//! ## Module Overview
//!
//! ## 模块概述
//!
//! Defines the schema for Danmaku Performance assets (`.performance.ron`) and related data structures.
//! This module provides the data contract for the timeline-based danmaku system.
//!
//! 定义弹幕演出资产 (`.performance.ron`) 及相关数据结构的 Schema。
//! 本模块为基于时间轴的弹幕系统提供数据契约。
//!
//! Defines the DanmakuPerformance asset and related data structures for
//! the timeline-based danmaku system.
//!
//! 定义基于时间轴的弹幕系统的 DanmakuPerformance 资产和相关数据结构。

use crate::core::visual::Visual;
use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ============================================================================
// Core Types: DanmakuPerformance (Timeline & Reference Architecture)
// ============================================================================

/// Danmaku Performance asset - defines a complete bullet pattern sequence.
/// Corresponds to a .performance.ron file.
///
/// 弹幕演出资产 - 定义完整的弹幕模式序列。
/// 对应 .performance.ron 文件。
#[derive(Asset, Debug, Clone, Deserialize, Serialize, Reflect)]
#[reflect(Debug)]
pub struct DanmakuPerformance {
    /// Bullet prototypes: ID -> visual/collision/damage data
    #[serde(default)]
    pub prototypes: HashMap<String, BulletPrototype>,

    /// Behavior definitions: ID -> BulletBehavior
    #[serde(default)]
    pub behaviors: HashMap<String, BulletBehavior>,

    /// Timeline: sequence of timed events
    pub timeline: Vec<TimelineEvent>,
}

// ============================================================================
// Bullet Prototype
// ============================================================================

/// Hit behavior preset for RON configuration.
/// Maps to BulletHitBehavior component at runtime.
///
/// RON 配置的命中行为预设。
/// 在运行时映射到 BulletHitBehavior 组件。
#[derive(Debug, Clone, Deserialize, Serialize, Reflect, Default)]
pub enum HitBehaviorPreset {
    /// Default: despawn on hit, damage always (default)
    #[default]
    Default,
    /// Persistent: doesn't despawn on hit, has i-frames
    Persistent,
    /// Blue soul style: damage only when player is moving
    DamageWhenMoving,
    /// Orange soul style: damage only when player is stationary
    DamageWhenStationary,
    /// Custom configuration
    Custom {
        #[serde(default = "HitBehaviorPreset::default_true")]
        despawn_on_hit: bool,
        #[serde(default)]
        damage_on_player_moving: bool,
        #[serde(default)]
        damage_on_player_stationary: bool,
        #[serde(default)]
        invincibility_duration: f32,
    },
}

impl HitBehaviorPreset {
    fn default_true() -> bool {
        true
    }
}

/// Color tint configuration for bullets.
/// Supports hex color strings like "#FCA600".
///
/// 弹幕的颜色叠加配置。
/// 支持十六进制颜色字符串如 "#FCA600"。
#[derive(Debug, Clone, Deserialize, Serialize, Reflect, Default)]
pub struct ColorTint {
    /// Hex color string (e.g., "#FCA600" for orange, "#40FEFE" for blue)
    #[serde(default)]
    pub hex: String,
    /// RGBA values (0.0-1.0), used if hex is empty
    #[serde(default)]
    pub rgba: Option<(f32, f32, f32, f32)>,
}

impl ColorTint {
    /// Convert to Bevy Color
    pub fn to_color(&self) -> Option<Color> {
        if !self.hex.is_empty() {
            parse_hex_color(&self.hex)
        } else if let Some((r, g, b, a)) = self.rgba {
            Some(Color::srgba(r, g, b, a))
        } else {
            None
        }
    }
}

/// Parse hex color string to Color.
/// Supports formats: "#RGB", "#RGBA", "#RRGGBB", "#RRGGBBAA"
fn parse_hex_color(hex: &str) -> Option<Color> {
    let hex = hex.trim_start_matches('#');
    match hex.len() {
        3 => {
            // #RGB
            let r = u8::from_str_radix(&hex[0..1], 16).ok()? * 17;
            let g = u8::from_str_radix(&hex[1..2], 16).ok()? * 17;
            let b = u8::from_str_radix(&hex[2..3], 16).ok()? * 17;
            Some(Color::srgb_u8(r, g, b))
        }
        4 => {
            // #RGBA
            let r = u8::from_str_radix(&hex[0..1], 16).ok()? * 17;
            let g = u8::from_str_radix(&hex[1..2], 16).ok()? * 17;
            let b = u8::from_str_radix(&hex[2..3], 16).ok()? * 17;
            let a = u8::from_str_radix(&hex[3..4], 16).ok()? * 17;
            Some(Color::srgba_u8(r, g, b, a))
        }
        6 => {
            // #RRGGBB
            let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
            let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
            let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
            Some(Color::srgb_u8(r, g, b))
        }
        8 => {
            // #RRGGBBAA
            let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
            let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
            let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
            let a = u8::from_str_radix(&hex[6..8], 16).ok()?;
            Some(Color::srgba_u8(r, g, b, a))
        }
        _ => None,
    }
}

/// Bullet prototype - defines the appearance and collision of a bullet type.
///
/// 弹幕原型 - 定义弹幕类型的外观和碰撞。
#[derive(Debug, Clone, Deserialize, Serialize, Reflect)]
#[serde(default)]
pub struct BulletPrototype {
    /// Visual resource path (supports shorthand resolution)
    pub visual: Visual,

    /// Collision shape
    pub collider: ColliderShape,

    /// Base damage
    pub damage: f32,

    /// Lifetime in seconds
    pub lifetime: f32,

    /// Z-index for rendering order
    pub z_index: f32,

    /// Scale factor (default: 1.0)
    pub scale: f32,

    /// Hit behavior configuration (default: despawn on hit)
    pub hit_behavior: HitBehaviorPreset,

    /// Color tint overlay (for blue/orange soul bullets)
    /// Empty hex string means no tint
    pub color_tint: ColorTint,

    /// Horizontal flip for sprite rendering
    #[serde(default)]
    pub flip_x: bool,

    /// Vertical flip for sprite rendering
    #[serde(default)]
    pub flip_y: bool,

    /// Frame duration for frame animations (seconds, default: 0.05)
    #[serde(default)]
    pub frame_duration: Option<f32>,
}

impl Default for BulletPrototype {
    fn default() -> Self {
        Self {
            visual: Visual::default(),
            collider: ColliderShape::default(),
            damage: 1.0,
            lifetime: 5.0,
            z_index: 15.0,
            scale: 1.0,
            hit_behavior: HitBehaviorPreset::Default,
            color_tint: ColorTint::default(),
            flip_x: false,
            flip_y: false,
            frame_duration: None,
        }
    }
}

impl BulletPrototype {
    pub fn new(visual: Visual) -> Self {
        Self {
            visual,
            ..Default::default()
        }
    }
}

/// Collider shape for hit detection.
///
/// 碰撞形状用于命中检测。
#[derive(Debug, Clone, Deserialize, Serialize, Reflect)]
pub enum ColliderShape {
    CircleCollider(f32),
    BoxCollider(f32, f32),
}

impl Default for ColliderShape {
    fn default() -> Self {
        ColliderShape::CircleCollider(4.0)
    }
}

// ============================================================================
// Bullet Behavior
// ============================================================================

/// Bullet behavior definition.
/// Supports both built-in algorithms and custom WASM mod algorithms.
///
/// 弹幕行为定义。
/// 同时支持内置算法和自定义 WASM mod 算法。
#[derive(Debug, Clone, Deserialize, Serialize, Reflect)]
pub enum BulletBehavior {
    // === Built-in Behaviors (内置行为) ===
    /// Linear motion in a direction
    ///
    /// 朝一个方向进行直线运动
    Linear(LinearConfig),

    /// Tween animation (opacity, scale, position, etc.)
    ///
    /// 缓动动画（不透明度、比例、位置等）
    Tween(TweenConfig),

    /// stay still
    ///
    /// 静止不动
    Stationary(),

    /// Orbital motion around spawn center (rotation + radial movement)
    ///
    /// 围绕生成中心的轨道运动（旋转+径向运动）
    Orbital(OrbitalConfig),

    /// Sinusoidal oscillation
    ///
    /// 正弦波震荡
    Sine(SineConfig),

    // === Custom Behavior (自定义行为) ===
    /// Algorithm loaded from mod system via WASM
    ///
    /// 通过 WASM 从 mod 系统加载算法
    Custom {
        /// Algorithm ID registered in DanmakuRegistry
        id: String,
        /// Properties passed to the algorithm
        #[serde(default)]
        props: HashMap<String, f32>,
    },
}

/// Linear motion configuration.
#[derive(Debug, Clone, Deserialize, Serialize, Reflect)]
#[serde(default)]
pub struct LinearConfig {
    pub dir: (f32, f32),
    pub speed: f32,
}

impl Default for LinearConfig {
    fn default() -> Self {
        Self {
            dir: (0.0, -1.0),
            speed: 100.0,
        }
    }
}

/// Orbital motion configuration (rotation around spawn center).
///
/// 轨道运动配置（围绕生成中心旋转）。
#[derive(Debug, Clone, Deserialize, Serialize, Reflect)]
#[serde(default)]
pub struct OrbitalConfig {
    pub angular_velocity: f32,
    pub radial_velocity: f32,
}

impl Default for OrbitalConfig {
    fn default() -> Self {
        Self {
            angular_velocity: 1.0,
            radial_velocity: 0.0,
        }
    }
}

/// Sine wave configuration.
#[derive(Debug, Clone, Deserialize, Serialize, Reflect)]
#[serde(default)]
pub struct SineConfig {
    pub axis: (f32, f32),
    pub amplitude: f32,
    pub frequency: f32,
    pub phase: f32,
}

impl Default for SineConfig {
    fn default() -> Self {
        Self {
            axis: (1.0, 0.0),
            amplitude: 20.0,
            frequency: 2.0,
            phase: 0.0,
        }
    }
}

/// Tween animation configuration.
#[derive(Debug, Clone, Deserialize, Serialize, Reflect)]
pub struct TweenConfig {
    /// Target property to animate
    pub target: TweenTarget,
    /// Duration in seconds
    pub duration: f32,
    /// Easing function
    #[serde(default)]
    pub ease: Easing,
    /// Value range (start, end)
    pub range: (f32, f32),
    /// Delay before starting
    #[serde(default)]
    pub delay: f32,
}

/// Tween target properties.
#[derive(Debug, Clone, Copy, Deserialize, Serialize, Reflect, Default)]
pub enum TweenTarget {
    #[default]
    Opacity,
    Scale,
    ScaleX,
    ScaleY,
    PositionX,
    PositionY,
    Rotation,
}

/// Easing functions for interpolation.
#[derive(Debug, Clone, Copy, Deserialize, Serialize, Default, Reflect)]
pub enum Easing {
    #[default]
    Linear,
    QuadIn,
    QuadOut,
    QuadInOut,
    CubicIn,
    CubicOut,
    CubicInOut,
    SineIn,
    SineOut,
    SineInOut,
}

impl Easing {
    pub fn apply(self, t: f32) -> f32 {
        match self {
            Easing::Linear => t,
            Easing::QuadIn => t * t,
            Easing::QuadOut => 1.0 - (1.0 - t) * (1.0 - t),
            Easing::QuadInOut => {
                if t < 0.5 {
                    2.0 * t * t
                } else {
                    1.0 - (-2.0 * t + 2.0).powi(2) / 2.0
                }
            }
            Easing::CubicIn => t * t * t,
            Easing::CubicOut => 1.0 - (1.0 - t).powi(3),
            Easing::CubicInOut => {
                if t < 0.5 {
                    4.0 * t * t * t
                } else {
                    1.0 - (-2.0 * t + 2.0).powi(3) / 2.0
                }
            }
            Easing::SineIn => 1.0 - (t * std::f32::consts::FRAC_PI_2).cos(),
            Easing::SineOut => (t * std::f32::consts::FRAC_PI_2).sin(),
            Easing::SineInOut => -(t * std::f32::consts::PI).cos() / 2.0 + 0.5,
        }
    }
}

// ============================================================================
// Timeline Event
// ============================================================================

/// Timeline event - describes what happens at a specific time.
///
/// 时间轴事件 - 描述在特定时间发生的事情。
#[derive(Debug, Clone, Deserialize, Serialize, Reflect)]
pub struct TimelineEvent {
    /// Time in seconds from performance start.
    /// By default this is relative time (delta from previous event).
    /// Set `absolute: true` to use absolute time from performance start.
    ///
    /// 距演出开始的时间（秒）。
    /// 默认为相对时间（与上一事件的时间差）。
    /// 设置 `absolute: true` 使用距演出开始的绝对时间。
    pub t: f32,

    /// Whether t is absolute time (from performance start) or relative (from previous event).
    /// Default is false (relative time).
    ///
    /// t 是否为绝对时间（从演出开始）还是相对时间（从上一事件）。
    /// 默认为 false（相对时间）。
    #[serde(default)]
    pub absolute: bool,

    /// Prototype ID to spawn
    pub spawn: String,

    /// Spawn pattern (built-in geometric patterns)
    #[serde(default)]
    pub pattern: SpawnPattern,

    /// List of behavior IDs to apply (references to `behaviors` map)
    #[serde(default)]
    pub apply: Vec<String>,

    /// Inline behavior definitions (applied after referenced behaviors)
    /// Use this for one-off behaviors that don't need to be reused.
    ///
    /// 内联行为定义（在引用行为之后应用）
    /// 用于不需要复用的一次性行为。
    #[serde(default)]
    pub behaviors: Vec<BulletBehavior>,
}

/// Spawn pattern for timeline events.
/// Built-in geometric patterns for bullet arrangement.
///
/// 时间轴事件的生成模式。
/// 内置的几何图案用于弹幕排列。
#[derive(Debug, Clone, Deserialize, Serialize, Reflect)]
pub enum SpawnPattern {
    /// Spawn a single bullet at center + offset.
    ///
    /// 在中心 + 偏移处生成单个弹幕。
    /// offset 默认为 (0, 0)，即中心。
    Single {
        #[serde(default)]
        offset: (f32, f32),
    },

    /// Spawn bullets in a ring/circle
    ///
    /// 生成环状/圆形弹幕
    RingGenerator {
        count: usize,
        #[serde(default)]
        radius: f32,
        #[serde(default)]
        start_angle: f32,
    },

    /// Spawn bullets in a line
    ///
    /// 生成线形弹幕
    LineGenerator {
        count: usize,
        #[serde(default = "SpawnPattern::default_line_spacing")]
        spacing: f32,
        #[serde(default = "SpawnPattern::default_linear_direction")]
        direction: (f32, f32),
    },

    /// Generate a line of bullets along a screen edge.
    /// Bullets are evenly spaced and fired perpendicular to that edge.
    ///
    /// 在屏幕边缘生成一排等距弹幕，
    /// 子弹将沿该边缘的垂直方向发射。
    EdgeGenerator {
        count: usize,
        #[serde(default)]
        side: EdgeSide,
        #[serde(default = "SpawnPattern::default_edge_spacing")]
        spacing: f32,
        #[serde(default = "SpawnPattern::default_edge_margin")]
        margin: f32,
    },

    /// Custom spawn pattern from mod system
    ///
    /// 通过 mod 系统自定义弹幕样式
    CustomGenerator {
        id: String,
        #[serde(default)]
        params: HashMap<String, f32>,
    },
}

impl Default for SpawnPattern {
    fn default() -> Self {
        Self::Single {
            offset: (0.0, 0.0),
        }
    }
}

impl SpawnPattern {
    fn default_line_spacing() -> f32 {
        20.0
    }
    fn default_linear_direction() -> (f32, f32) {
        (0.0, -1.0)
    }
    fn default_edge_spacing() -> f32 {
        30.0
    }
    fn default_edge_margin() -> f32 {
        200.0
    }
}

/// Which screen edge to spawn from.
#[derive(Debug, Clone, Copy, Deserialize, Serialize, Default, Reflect)]
pub enum EdgeSide {
    #[default]
    Left,
    Right,
    Top,
    Bottom,
}

impl EdgeSide {
    pub fn to_direction(self) -> Vec2 {
        match self {
            EdgeSide::Left => Vec2::new(1.0, 0.0),
            EdgeSide::Right => Vec2::new(-1.0, 0.0),
            EdgeSide::Top => Vec2::new(0.0, -1.0),
            EdgeSide::Bottom => Vec2::new(0.0, 1.0),
        }
    }

    pub fn to_offset(self, margin: f32) -> Vec2 {
        match self {
            EdgeSide::Left => Vec2::new(-margin, 0.0),
            EdgeSide::Right => Vec2::new(margin, 0.0),
            EdgeSide::Top => Vec2::new(0.0, margin),
            EdgeSide::Bottom => Vec2::new(0.0, -margin),
        }
    }
}

// ============================================================================
// Runtime Resources and Events
// ============================================================================

/// Resource tracking pending performance loads.
#[derive(Resource, Default)]
pub struct PendingPerformanceLoads {
    pub pending: Vec<(Handle<DanmakuPerformance>, PlayPerformanceEvent)>,
}

/// Event to play a danmaku performance.
///
/// 播放弹幕演出的事件。
#[derive(bevy::ecs::message::Message, Clone)]
pub struct PlayPerformanceEvent {
    /// Path to the .performance.ron file
    pub performance_path: String,
    /// Center position for the performance
    pub position: Vec2,
}

impl PlayPerformanceEvent {
    pub fn new(performance_path: impl Into<String>) -> Self {
        Self {
            performance_path: performance_path.into(),
            position: Vec2::ZERO,
        }
    }

    pub fn at_position(mut self, position: Vec2) -> Self {
        self.position = position;
        self
    }
}
