//! # patterns.rs
//!
//! ## Module Overview
//!
//! Defines the DanmakuBlueprint asset and related data structures for
//! the stack-based composite danmaku system.
//!
//! 定义基于堆栈的复合弹幕系统的 DanmakuBlueprint 资产和相关数据结构。

use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Event to spawn a bullet pattern from a blueprint asset.
///
/// 从蓝图资产生成弹幕模式的事件。
#[derive(bevy::ecs::message::Message, Clone)]
pub struct SpawnPatternEvent {
    /// Path to the .danmaku.ron file (e.g., "battle/patterns/flowey_pellet.danmaku.ron")
    pub blueprint_path: String,
    /// Center position for spawning
    pub position: Vec2,
    /// Number of bullets to spawn (for patterns that support it)
    pub count: Option<usize>,
    /// Optional parameters override
    pub params_override: HashMap<String, f32>,
}

impl SpawnPatternEvent {
    pub fn new(blueprint_path: impl Into<String>) -> Self {
        Self {
            blueprint_path: blueprint_path.into(),
            position: Vec2::ZERO,
            count: None,
            params_override: HashMap::new(),
        }
    }

    pub fn at_position(mut self, position: Vec2) -> Self {
        self.position = position;
        self
    }

    pub fn with_count(mut self, count: usize) -> Self {
        self.count = Some(count);
        self
    }

    pub fn with_param(mut self, key: impl Into<String>, value: f32) -> Self {
        self.params_override.insert(key.into(), value);
        self
    }
}

// ============================================================================
// DanmakuBlueprint: The core asset definition
// ============================================================================

/// Danmaku Blueprint: Defines all properties of a bullet type.
/// Corresponds to a .danmaku.ron file.
///
/// 弹幕蓝图：定义一种弹幕的所有属性（对应一个 .danmaku.ron 文件）
#[derive(Asset, TypePath, Debug, Clone, Deserialize, Serialize)]
pub struct DanmakuBlueprint {
    // === Appearance ===
    /// Path to sprite image or animation name
    pub visual: BulletVisual,
    /// Collider radius for hit detection
    #[serde(default = "default_collider_radius")]
    pub collider_radius: f32,
    /// Base damage
    #[serde(default = "default_damage")]
    pub damage: f32,
    /// Lifetime in seconds
    #[serde(default = "default_lifetime")]
    pub lifetime: f32,
    /// Z-index for rendering order
    #[serde(default = "default_z_index")]
    pub z_index: f32,

    // === Motion Stack ===
    /// List of motion tracks that are evaluated and summed
    #[serde(default)]
    pub motion_tracks: Vec<MotionTrack>,

    // === Spawning Configuration ===
    /// How bullets are spawned (single, circle, line, etc.)
    #[serde(default)]
    pub spawn_pattern: SpawnPattern,

    // === Child Spawners ===
    /// Nested bullet spawners triggered during lifetime
    #[serde(default)]
    pub children: Vec<ChildSpawner>,
}

fn default_collider_radius() -> f32 {
    4.0
}
fn default_damage() -> f32 {
    1.0
}
fn default_lifetime() -> f32 {
    5.0
}
fn default_z_index() -> f32 {
    5.0
}

/// Visual representation of a bullet.
///
/// 弹幕的视觉表现。
#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum BulletVisual {
    /// Static sprite image
    Sprite { path: String },
    /// Animated sprite (references animation name in config.toml)
    Animation {
        module: String,
        name: String,
        #[serde(default = "default_frame_duration")]
        frame_duration: f32,
    },
}

fn default_frame_duration() -> f32 {
    0.05
}

/// Spawn pattern configuration.
///
/// 生成模式配置。
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub enum SpawnPattern {
    /// Spawn a single bullet
    #[default]
    Single,
    /// Spawn bullets in a circle around a center point
    Circle {
        #[serde(default = "default_circle_count")]
        count: usize,
        #[serde(default = "default_circle_radius")]
        radius: f32,
        /// Starting angle in radians
        #[serde(default)]
        start_angle: f32,
    },
    /// Spawn bullets in a line
    Line {
        #[serde(default = "default_line_count")]
        count: usize,
        #[serde(default = "default_line_spacing")]
        spacing: f32,
        /// Direction of the line (normalized)
        #[serde(default = "default_line_direction")]
        direction: (f32, f32),
    },
    /// Spawn bullets from screen edge
    Edge {
        #[serde(default = "default_edge_count")]
        count: usize,
        #[serde(default)]
        side: EdgeSide,
        #[serde(default = "default_edge_spacing")]
        spacing: f32,
        #[serde(default = "default_edge_margin")]
        margin: f32,
    },
}

fn default_circle_count() -> usize {
    8
}
fn default_circle_radius() -> f32 {
    100.0
}
fn default_line_count() -> usize {
    5
}
fn default_line_spacing() -> f32 {
    20.0
}
fn default_line_direction() -> (f32, f32) {
    (1.0, 0.0)
}
fn default_edge_count() -> usize {
    5
}
fn default_edge_spacing() -> f32 {
    30.0
}
fn default_edge_margin() -> f32 {
    200.0
}

/// Which screen edge to spawn from.
#[derive(Debug, Clone, Copy, Deserialize, Serialize, Default)]
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

    pub fn rotation_angle(self) -> f32 {
        match self {
            EdgeSide::Left => 0.0,
            EdgeSide::Right => std::f32::consts::PI,
            EdgeSide::Top => -std::f32::consts::FRAC_PI_2,
            EdgeSide::Bottom => std::f32::consts::FRAC_PI_2,
        }
    }
}

// ============================================================================
// Motion Tracks: The stack-based motion system
// ============================================================================

/// Motion Track: Defines a single dimension of motion logic.
/// Multiple tracks are evaluated and their results summed.
///
/// 运动轨道：定义单一维度的运动逻辑。
/// 多个轨道的结果会被计算并叠加。
#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum MotionTrack {
    /// Keyframed motion using bezier curves
    Keyframed {
        target: PropertyTarget,
        keyframes: Vec<Keyframe>,
        #[serde(default)]
        loop_mode: LoopMode,
    },

    /// Linear motion in a direction
    Linear {
        #[serde(default = "default_linear_direction")]
        direction: (f32, f32),
        #[serde(default = "default_linear_speed")]
        speed: f32,
    },

    /// Circular/orbital motion around spawn point
    Circular {
        #[serde(default = "default_angular_velocity")]
        angular_velocity: f32,
        /// Radial velocity (positive = expand, negative = contract)
        #[serde(default)]
        radial_velocity: f32,
    },

    /// Sinusoidal oscillation
    Sine {
        #[serde(default = "default_sine_axis")]
        axis: (f32, f32),
        #[serde(default = "default_sine_amplitude")]
        amplitude: f32,
        #[serde(default = "default_sine_frequency")]
        frequency: f32,
        #[serde(default)]
        phase: f32,
    },

    /// Homing towards player
    Homing {
        #[serde(default = "default_homing_strength")]
        strength: f32,
        #[serde(default = "default_homing_max_turn")]
        max_turn_rate: f32,
    },

    /// Algorithm-driven track (for complex custom behaviors)
    /// References a registered algorithm by ID
    Algorithmic {
        algorithm_id: String,
        #[serde(default)]
        parameters: HashMap<String, f32>,
    },
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

/// Property target for keyframed animation.
#[derive(Debug, Clone, Copy, Deserialize, Serialize, Default)]
pub enum PropertyTarget {
    #[default]
    Position,
    PositionX,
    PositionY,
    Rotation,
    Scale,
    Alpha,
}

/// A single keyframe in an animation.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Keyframe {
    /// Time in seconds
    pub t: f32,
    /// Value at this keyframe
    pub value: KeyframeValue,
    /// Easing function
    #[serde(default)]
    pub ease: EaseFunction,
}

/// Keyframe value types.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum KeyframeValue {
    Float(f32),
    Vec2(f32, f32),
}

/// Easing functions for keyframe interpolation.
#[derive(Debug, Clone, Copy, Deserialize, Serialize, Default)]
pub enum EaseFunction {
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

impl EaseFunction {
    pub fn apply(self, t: f32) -> f32 {
        match self {
            EaseFunction::Linear => t,
            EaseFunction::QuadIn => t * t,
            EaseFunction::QuadOut => 1.0 - (1.0 - t) * (1.0 - t),
            EaseFunction::QuadInOut => {
                if t < 0.5 {
                    2.0 * t * t
                } else {
                    1.0 - (-2.0 * t + 2.0).powi(2) / 2.0
                }
            }
            EaseFunction::CubicIn => t * t * t,
            EaseFunction::CubicOut => 1.0 - (1.0 - t).powi(3),
            EaseFunction::CubicInOut => {
                if t < 0.5 {
                    4.0 * t * t * t
                } else {
                    1.0 - (-2.0 * t + 2.0).powi(3) / 2.0
                }
            }
            EaseFunction::SineIn => 1.0 - (t * std::f32::consts::FRAC_PI_2).cos(),
            EaseFunction::SineOut => (t * std::f32::consts::FRAC_PI_2).sin(),
            EaseFunction::SineInOut => -(t * std::f32::consts::PI).cos() / 2.0 + 0.5,
        }
    }
}

/// Loop mode for keyframed animations.
#[derive(Debug, Clone, Copy, Deserialize, Serialize, Default)]
pub enum LoopMode {
    #[default]
    Once,
    Loop,
    PingPong,
}

// ============================================================================
// Child Spawners: Nested bullet spawning
// ============================================================================

/// Child Spawner: Defines nested spawning logic.
///
/// 子发射器：定义嵌套生成逻辑。
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ChildSpawner {
    /// Trigger condition
    pub trigger: SpawnTrigger,
    /// Path to child blueprint
    pub blueprint_path: String,
    /// Whether to inherit parent's transform
    #[serde(default = "default_true")]
    pub inherit_transform: bool,
    /// Local offset from parent
    #[serde(default)]
    pub offset: (f32, f32),
}

fn default_true() -> bool {
    true
}

/// Spawn trigger conditions.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum SpawnTrigger {
    /// Spawn at a specific time
    AtTime(f32),
    /// Spawn when bullet is destroyed
    OnDeath,
    /// Spawn periodically
    Interval { start: f32, interval: f32 },
}

// ============================================================================
// Runtime Resources
// ============================================================================

/// Resource tracking pending blueprint loads.
#[derive(Resource, Default)]
pub struct PendingBlueprintLoads {
    pub pending: Vec<(Handle<DanmakuBlueprint>, SpawnPatternEvent)>,
}
