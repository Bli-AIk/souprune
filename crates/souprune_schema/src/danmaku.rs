//! # danmaku.rs
//!
//! DanmakuPerformance schema types for `.performance.ron` files.
//! Mirrors `souprune::core::danmaku::danmaku_schema` without Bevy dependency.
//!
//! `.performance.ron` 文件的弹幕演出 Schema 类型。

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ============================================================================
// Core Types
// ============================================================================

/// Danmaku Performance asset — top-level `.performance.ron` schema.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DanmakuPerformance {
    #[serde(default)]
    pub prototypes: HashMap<String, BulletPrototype>,
    #[serde(default)]
    pub behaviors: HashMap<String, BulletBehavior>,
    pub timeline: Vec<TimelineEvent>,
}

// ============================================================================
// Bullet Prototype
// ============================================================================

/// Hit behavior preset.
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
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct ColorTint {
    #[serde(default)]
    pub hex: String,
    #[serde(default)]
    pub rgba: Option<(f32, f32, f32, f32)>,
}

/// Bullet prototype — appearance and collision definition.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct BulletPrototype {
    pub visual: String,
    pub collider: ColliderShape,
    pub damage: f32,
    pub lifetime: f32,
    pub z_index: f32,
    pub scale: f32,
    #[serde(default)]
    pub rotation: f32,
    pub hit_behavior: HitBehaviorPreset,
    pub color_tint: ColorTint,
    #[serde(default)]
    pub flip_x: bool,
    #[serde(default)]
    pub flip_y: bool,
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

/// Collider shape for hit detection.
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

// ============================================================================
// Bullet Behavior
// ============================================================================

/// Bullet behavior definition.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum BulletBehavior {
    Linear(LinearConfig),
    Tween(TweenConfig),
    Stationary(),
    Orbital(OrbitalConfig),
    Sine(SineConfig),
    Aimed(AimedConfig),
    Custom {
        id: String,
        #[serde(default)]
        props: HashMap<String, f32>,
    },
}

/// Linear motion configuration.
#[derive(Debug, Clone, Deserialize, Serialize)]
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

/// Orbital motion configuration.
#[derive(Debug, Clone, Deserialize, Serialize)]
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
#[derive(Debug, Clone, Deserialize, Serialize)]
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

/// Aimed bullet configuration (自机狙).
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct AimedConfig {
    pub speed: f32,
    pub angle_offset: f32,
}

impl Default for AimedConfig {
    fn default() -> Self {
        Self {
            speed: 120.0,
            angle_offset: 0.0,
        }
    }
}

/// Tween animation configuration.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TweenConfig {
    pub target: DanmakuTweenTarget,
    pub duration: f32,
    #[serde(default)]
    pub ease: Easing,
    pub range: (f32, f32),
    #[serde(default)]
    pub delay: f32,
}

/// Tween target properties (danmaku-specific).
#[derive(Debug, Clone, Copy, Deserialize, Serialize, Default)]
pub enum DanmakuTweenTarget {
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
#[derive(Debug, Clone, Copy, Deserialize, Serialize, Default)]
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

// ============================================================================
// Timeline Event
// ============================================================================

/// Timeline event — describes what happens at a specific time.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TimelineEvent {
    pub t: f32,
    #[serde(default)]
    pub absolute: bool,
    pub spawn: String,
    #[serde(default)]
    pub pattern: SpawnPattern,
    #[serde(default)]
    pub offset: (f32, f32),
    #[serde(default)]
    pub apply: Vec<String>,
    #[serde(default)]
    pub behaviors: Vec<BulletBehavior>,
}

/// Spawn pattern for timeline events.
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub enum SpawnPattern {
    #[default]
    Single,
    RingGenerator {
        count: usize,
        #[serde(default)]
        radius: f32,
        #[serde(default)]
        start_angle: f32,
    },
    LineGenerator {
        count: usize,
        #[serde(default = "default_line_spacing")]
        spacing: f32,
        #[serde(default = "default_linear_direction")]
        direction: (f32, f32),
    },
    EdgeGenerator {
        count: usize,
        #[serde(default)]
        side: EdgeSide,
        #[serde(default = "default_edge_spacing")]
        spacing: f32,
        #[serde(default = "default_edge_margin")]
        margin: f32,
    },
    CustomGenerator {
        id: String,
        #[serde(default)]
        params: HashMap<String, f32>,
    },
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

// ============================================================================
// Default helper functions
// ============================================================================

fn default_true() -> bool {
    true
}

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
