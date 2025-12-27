//! # components.rs
//!
//! ## Module Overview
//!
//! Defines ECS components for the danmaku system.
//!
//! 定义弹幕系统的 ECS 组件。

use bevy::prelude::*;

/// Marker component for bullet entities.
///
/// 弹幕实体的标记组件。
#[derive(Component)]
pub struct Bullet;

/// Bullet velocity component.
///
/// 弹幕速度组件。
#[derive(Component, Default)]
pub struct BulletVelocity(pub Vec2);

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

/// Component for bullets that follow a circular motion around a center point.
///
/// 环绕中心点做圆周运动的弹幕组件。
#[derive(Component)]
pub struct CircularMotion {
    pub center: Vec2,
    pub radius: f32,
    pub angular_velocity: f32,
    pub current_angle: f32,
    pub radial_velocity: f32,
}

impl CircularMotion {
    pub fn new(center: Vec2, radius: f32, start_angle: f32, angular_velocity: f32) -> Self {
        Self {
            center,
            radius,
            angular_velocity,
            current_angle: start_angle,
            radial_velocity: 0.0,
        }
    }

    pub fn with_radial_velocity(mut self, radial_velocity: f32) -> Self {
        self.radial_velocity = radial_velocity;
        self
    }
}

/// Component for linear bullet motion.
///
/// 线性弹幕运动组件。
#[derive(Component)]
pub struct LinearMotion {
    pub direction: Vec2,
    pub speed: f32,
}

impl LinearMotion {
    pub fn new(direction: Vec2, speed: f32) -> Self {
        Self {
            direction: direction.normalize_or_zero(),
            speed,
        }
    }
}

/// Component for sweeping motion (like Undyne spears).
///
/// 横扫运动组件（如 Undyne 的矛）。
#[derive(Component)]
pub struct SweepMotion {
    pub start_pos: Vec2,
    pub end_pos: Vec2,
    pub duration: f32,
    pub elapsed: f32,
}

impl SweepMotion {
    pub fn new(start_pos: Vec2, end_pos: Vec2, duration: f32) -> Self {
        Self {
            start_pos,
            end_pos,
            duration,
            elapsed: 0.0,
        }
    }

    pub fn progress(&self) -> f32 {
        (self.elapsed / self.duration).clamp(0.0, 1.0)
    }
}
