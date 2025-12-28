//! # patterns.rs
//!
//! ## Module Overview
//!
//! Defines the DanmakuPerformance asset and related data structures for
//! the timeline-based danmaku system.
//!
//! 定义基于时间轴的弹幕系统的 DanmakuPerformance 资产和相关数据结构。

use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ============================================================================
// Default Value Functions
// ============================================================================

fn default_damage() -> f32 {
    1.0
}

fn default_lifetime() -> f32 {
    5.0
}

fn default_z_index() -> f32 {
    15.0
}

fn default_frame_duration() -> f32 {
    0.05
}

fn default_linear_direction() -> (f32, f32) {
    (0.0, -1.0)
}

fn default_linear_speed() -> f32 {
    100.0
}

fn default_angular_velocity() -> f32 {
    1.0
}

fn default_sine_axis() -> (f32, f32) {
    (1.0, 0.0)
}

fn default_sine_amplitude() -> f32 {
    20.0
}

fn default_sine_frequency() -> f32 {
    2.0
}

fn default_homing_strength() -> f32 {
    1.0
}

fn default_homing_max_turn() -> f32 {
    3.0
}

fn default_line_spacing() -> f32 {
    20.0
}

fn default_edge_spacing() -> f32 {
    30.0
}

fn default_edge_margin() -> f32 {
    200.0
}

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

/// Bullet prototype - defines the appearance and collision of a bullet type.
///
/// 弹幕原型 - 定义弹幕类型的外观和碰撞。
#[derive(Debug, Clone, Deserialize, Serialize, Reflect)]
pub struct BulletPrototype {
    /// Visual representation
    pub visual: BulletVisual,

    /// Collision shape
    #[serde(default)]
    pub collider: ColliderShape,

    /// Base damage
    #[serde(default = "default_damage")]
    pub damage: f32,

    /// Lifetime in seconds
    #[serde(default = "default_lifetime")]
    pub lifetime: f32,

    /// Z-index for rendering order
    #[serde(default = "default_z_index")]
    pub z_index: f32,
}

/// Visual representation of a bullet.
///
/// 弹幕的视觉表现。
#[derive(Debug, Clone, Deserialize, Serialize, Reflect)]
pub enum BulletVisual {
    /// Static sprite image by direct path (legacy)
    Sprite { path: String },
    /// Static sprite by reference to config.toml sprite name
    SpriteRef { module: String, name: String },
    /// Animated sprite (references animation name in config.toml)
    Animation {
        module: String,
        name: String,
        #[serde(default = "default_frame_duration")]
        frame_duration: f32,
    },
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
/// Supports both built-in algorithms and custom FFI algorithms.
///
/// 弹幕行为定义。
/// 同时支持内置算法和自定义 FFI 算法。
#[derive(Debug, Clone, Deserialize, Serialize, Reflect)]
pub enum BulletBehavior {
    // === Built-in Behaviors (内置行为) ===
    /// Linear motion in a direction
    Linear(LinearConfig),

    /// Homing towards player
    Homing(HomingConfig),

    /// Tween animation (opacity, scale, position, etc.)
    Tween(TweenConfig),

    /// Orbital motion around spawn center (rotation + radial movement)
    Orbital(OrbitalConfig),

    /// Sinusoidal oscillation
    Sine(SineConfig),

    // === Custom Behavior (自定义行为) ===
    /// Algorithm loaded from mod system via FFI
    Algo {
        /// Algorithm ID registered in DanmakuRegistry
        id: String,
        /// Parameters passed to the algorithm
        #[serde(default)]
        params: Vec<f32>,
    },
}

/// Linear motion configuration.
#[derive(Debug, Clone, Deserialize, Serialize, Reflect)]
pub struct LinearConfig {
    #[serde(default = "default_linear_direction")]
    pub dir: (f32, f32),
    #[serde(default = "default_linear_speed")]
    pub speed: f32,
}

impl Default for LinearConfig {
    fn default() -> Self {
        Self {
            dir: default_linear_direction(),
            speed: default_linear_speed(),
        }
    }
}

/// Homing behavior configuration.
#[derive(Debug, Clone, Deserialize, Serialize, Reflect)]
pub struct HomingConfig {
    #[serde(default = "default_homing_strength")]
    pub strength: f32,
    #[serde(default = "default_homing_max_turn")]
    pub max_turn_rate: f32,
    #[serde(default = "default_linear_speed")]
    pub speed: f32,
}

impl Default for HomingConfig {
    fn default() -> Self {
        Self {
            strength: default_homing_strength(),
            max_turn_rate: default_homing_max_turn(),
            speed: default_linear_speed(),
        }
    }
}

/// Orbital motion configuration (rotation around spawn center).
///
/// 轨道运动配置（围绕生成中心旋转）。
#[derive(Debug, Clone, Deserialize, Serialize, Reflect)]
pub struct OrbitalConfig {
    #[serde(default = "default_angular_velocity")]
    pub angular_velocity: f32,
    #[serde(default)]
    pub radial_velocity: f32,
}

impl Default for OrbitalConfig {
    fn default() -> Self {
        Self {
            angular_velocity: default_angular_velocity(),
            radial_velocity: 0.0,
        }
    }
}

/// Sine wave configuration.
#[derive(Debug, Clone, Deserialize, Serialize, Reflect)]
pub struct SineConfig {
    #[serde(default = "default_sine_axis")]
    pub axis: (f32, f32),
    #[serde(default = "default_sine_amplitude")]
    pub amplitude: f32,
    #[serde(default = "default_sine_frequency")]
    pub frequency: f32,
    #[serde(default)]
    pub phase: f32,
}

impl Default for SineConfig {
    fn default() -> Self {
        Self {
            axis: default_sine_axis(),
            amplitude: default_sine_amplitude(),
            frequency: default_sine_frequency(),
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
    /// Spawn a single bullet at center
    Single,

    /// Spawn bullets in a ring/circle
    RingGenerator {
        count: usize,
        #[serde(default)]
        radius: f32,
        #[serde(default)]
        start_angle: f32,
    },

    /// Spawn bullets in a line
    LineGenerator {
        count: usize,
        #[serde(default = "default_line_spacing")]
        spacing: f32,
        #[serde(default = "default_linear_direction")]
        direction: (f32, f32),
    },

    /// Spawn bullets from a screen edge
    EdgeGenerator {
        count: usize,
        #[serde(default)]
        side: EdgeSide,
        #[serde(default = "default_edge_spacing")]
        spacing: f32,
        #[serde(default = "default_edge_margin")]
        margin: f32,
    },

    /// Custom spawn pattern from mod system
    CustomGenerator {
        id: String,
        #[serde(default)]
        params: HashMap<String, f32>,
    },
}

impl Default for SpawnPattern {
    fn default() -> Self {
        SpawnPattern::Single
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
