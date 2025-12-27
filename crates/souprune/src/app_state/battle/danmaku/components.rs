//! # components.rs
//!
//! ## Module Overview
//!
//! Defines ECS components for the danmaku system.
//!
//! 定义弹幕系统的 ECS 组件。

use super::patterns::{LoopMode, MotionTrack};
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

/// Runtime state for a bullet's motion stack evaluation.
/// Stores the current elapsed time and initial spawn data.
///
/// 弹幕运动堆栈评估的运行时状态。
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
    /// Current velocity direction (for homing/linear tracks)
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

    pub fn with_direction(mut self, direction: Vec2) -> Self {
        self.velocity_direction = direction.normalize_or_zero();
        self
    }
}

/// Component holding the motion tracks for a bullet.
/// This is cloned from the blueprint at spawn time.
///
/// 持有弹幕运动轨道的组件。
/// 在生成时从蓝图克隆。
#[derive(Component, Clone)]
pub struct BulletMotionTracks(pub Vec<MotionTrack>);

/// Component for keyframed track state.
#[derive(Component)]
pub struct KeyframedTrackState {
    pub current_keyframe_index: usize,
    pub loop_count: usize,
    pub loop_mode: LoopMode,
}

impl Default for KeyframedTrackState {
    fn default() -> Self {
        Self {
            current_keyframe_index: 0,
            loop_count: 0,
            loop_mode: LoopMode::Once,
        }
    }
}
