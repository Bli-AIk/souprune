//! # components.rs
//!
//! ## Module Overview
//!
//! Defines ECS components for the danmaku system.
//!
//! 定义弹幕系统的 ECS 组件。

use super::patterns::BulletBehavior;
use bevy::prelude::*;

/// Marker component for bullet entities.
///
/// 弹幕实体的标记组件。
#[derive(Component)]
pub struct Bullet;

/// Bullet lifetime component. When timer finishes, bullet is despawned.
///
/// 弹幕生命周期组件。当计时器结束时，弹幕被销毁。
#[derive(Component)]
pub struct BulletLifetime {
    pub timer: Timer,
}

impl BulletLifetime {
    pub fn new(seconds: f32) -> Self {
        Self {
            timer: Timer::from_seconds(seconds, TimerMode::Once),
        }
    }
}

/// Bullet damage component.
///
/// 弹幕伤害组件。
#[derive(Component)]
pub struct BulletDamage(pub f32);

impl Default for BulletDamage {
    fn default() -> Self {
        Self(1.0)
    }
}

/// Marker for bullets that should be despawned.
///
/// 标记需要销毁的弹幕。
#[derive(Component)]
pub struct DespawnBullet;

/// Runtime state for a bullet's motion evaluation.
/// Stores the current elapsed time and initial spawn data.
///
/// 弹幕运动评估的运行时状态。
/// 存储当前经过时间和初始生成数据。
#[derive(Component)]
pub struct BulletMotionState {
    /// Time since spawn
    pub elapsed: f32,
    /// Initial spawn position (center of pattern)
    pub spawn_center: Vec2,
    /// Initial position offset from center
    pub initial_offset: Vec2,
    /// Initial angle (for circular patterns)
    pub initial_angle: f32,
    /// Initial radius (for circular patterns)
    pub initial_radius: f32,
    /// Current velocity direction (for homing/linear behaviors)
    pub velocity_direction: Vec2,
}

impl BulletMotionState {
    pub fn new(spawn_center: Vec2) -> Self {
        Self {
            elapsed: 0.0,
            spawn_center,
            initial_offset: Vec2::ZERO,
            initial_angle: 0.0,
            initial_radius: 0.0,
            velocity_direction: Vec2::NEG_Y,
        }
    }

    pub fn with_offset(mut self, offset: Vec2) -> Self {
        self.initial_offset = offset;
        self
    }

    pub fn with_angle(mut self, angle: f32) -> Self {
        self.initial_angle = angle;
        self.velocity_direction = Vec2::new(angle.cos(), angle.sin());
        self
    }

    pub fn with_radius(mut self, radius: f32) -> Self {
        self.initial_radius = radius;
        self
    }
}

// ============================================================================
// Performance System Components
// ============================================================================

/// Behavior stack component for a bullet.
/// Contains a list of active behaviors that are evaluated each frame.
///
/// 弹幕的行为栈组件。
/// 包含每帧都会评估的活跃行为列表。
#[derive(Component, Clone)]
pub struct BehaviorStack {
    /// List of active behaviors
    pub behaviors: Vec<BulletBehavior>,
    /// Cached parameters for FFI algorithms
    pub cached_params: Vec<f32>,
}

impl BehaviorStack {
    pub fn new(behaviors: Vec<BulletBehavior>) -> Self {
        Self {
            behaviors,
            cached_params: Vec::new(),
        }
    }

    pub fn with_cached_params(mut self, params: Vec<f32>) -> Self {
        self.cached_params = params;
        self
    }
}

impl Default for BehaviorStack {
    fn default() -> Self {
        Self {
            behaviors: Vec::new(),
            cached_params: Vec::new(),
        }
    }
}

/// Tween state for tween behaviors.
/// Tracks the progress of tween animations.
///
/// 补间行为的状态。
/// 追踪补间动画的进度。
#[derive(Component, Default)]
pub struct TweenState {
    /// Current time for each tween (indexed by behavior position in stack)
    pub timers: Vec<f32>,
}

/// Performance player component.
/// Attached to an entity that is playing a DanmakuPerformance timeline.
///
/// 演出播放器组件。
/// 附加到正在播放 DanmakuPerformance 时间轴的实体上。
#[derive(Component)]
pub struct PerformancePlayer {
    /// Current playback time
    pub elapsed: f32,
    /// Index of the next event to process
    pub next_event_index: usize,
    /// Whether the performance has finished
    pub finished: bool,
    /// Center position for spawning
    pub spawn_center: Vec2,
}

impl PerformancePlayer {
    pub fn new(spawn_center: Vec2) -> Self {
        Self {
            elapsed: 0.0,
            next_event_index: 0,
            finished: false,
            spawn_center,
        }
    }
}

/// Component that links a PerformancePlayer to its loaded performance asset.
///
/// 将 PerformancePlayer 与其加载的演出资产关联的组件。
#[derive(Component)]
pub struct PerformanceHandle(pub Handle<super::patterns::DanmakuPerformance>);

/// Marker component for performance player entities.
///
/// 演出播放器实体的标记组件。
#[derive(Component)]
pub struct PerformancePlayerMarker;
