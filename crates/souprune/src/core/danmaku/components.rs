//! # components.rs
//!
//! ## Module Overview
//!
//! Defines ECS components for the danmaku system.
//!
//! 定义弹幕系统的 ECS 组件。

use super::danmaku_schema::BulletBehavior;
use bevy::prelude::*;

/// Marker component for bullet entities.
///
/// 弹幕实体的标记组件。
#[derive(Component)]
pub struct Bullet;

/// Container for all bullets spawned by a PerformancePlayer.
/// This helps organize the entity hierarchy in the inspector.
///
/// 由 PerformancePlayer 生成的所有弹幕的容器。
/// 这有助于在检查器中组织实体层级结构。
#[derive(Component, Debug)]
pub struct BulletContainer {
    /// Center position for this bullet container
    pub center: Vec2,
}

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

/// Bullet hit behavior configuration.
/// Controls what happens when a bullet hits the player.
///
/// 弹幕命中行为配置。
/// 控制弹幕击中玩家时的行为。
#[derive(Component, Debug, Clone, Default)]
pub struct BulletHitBehavior {
    /// Whether the bullet should despawn on hit (default: true)
    ///
    /// 弹幕命中后是否销毁（默认：true）
    pub despawn_on_hit: bool,

    /// Whether damage is only dealt when player is moving (default: false)
    /// If true, stationary player won't take damage
    ///
    /// 是否仅在玩家移动时造成伤害（默认：false）
    /// 如果为 true，静止的玩家不会受到伤害
    pub damage_on_player_moving: bool,

    /// Whether damage is only dealt when player is stationary (default: false)
    /// If true, moving player won't take damage
    ///
    /// 是否仅在玩家静止时造成伤害（默认：false）
    /// 如果为 true，移动的玩家不会受到伤害
    pub damage_on_player_stationary: bool,

    /// Invincibility frames after being hit (in seconds, default: 0.0)
    /// During this time, this bullet won't deal damage again
    ///
    /// 被击中后的无敌帧时间（秒，默认：0.0）
    /// 在此期间，该弹幕不会再次造成伤害
    pub invincibility_duration: f32,
}

impl BulletHitBehavior {
    /// Create default hit behavior (despawn on hit, damage always)
    pub fn default_despawn() -> Self {
        Self {
            despawn_on_hit: true,
            damage_on_player_moving: false,
            damage_on_player_stationary: false,
            invincibility_duration: 0.0,
        }
    }

    /// Create persistent bullet (doesn't despawn on hit)
    pub fn persistent() -> Self {
        Self {
            despawn_on_hit: false,
            damage_on_player_moving: false,
            damage_on_player_stationary: false,
            invincibility_duration: 0.5, // Default i-frames for persistent bullets
        }
    }

    /// Create "blue soul" style (damage only when moving)
    pub fn damage_when_moving() -> Self {
        Self {
            despawn_on_hit: true,
            damage_on_player_moving: true,
            damage_on_player_stationary: false,
            invincibility_duration: 0.0,
        }
    }

    /// Create "orange soul" style (damage only when stationary)
    pub fn damage_when_stationary() -> Self {
        Self {
            despawn_on_hit: true,
            damage_on_player_moving: false,
            damage_on_player_stationary: true,
            invincibility_duration: 0.0,
        }
    }
}

/// Tracks the last time this bullet dealt damage to the player.
/// Used for invincibility frame calculations.
///
/// 追踪该弹幕最后一次对玩家造成伤害的时间。
/// 用于无敌帧计算。
#[derive(Component, Debug, Clone, Default)]
pub struct BulletLastHitTime(pub f32);

/// Base scale for bullets, used as the base for Tween scale modifications.
/// Stores the initial scale from the prototype so Tween can multiply on top of it.
///
/// 弹幕的基础缩放，用作 Tween 缩放修改的基础。
/// 存储来自原型的初始缩放，以便 Tween 可以在其之上进行乘法。
#[derive(Component, Debug, Clone, Copy)]
pub struct BulletBaseScale(pub f32);

impl Default for BulletBaseScale {
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
    /// Current velocity direction (for aimed/linear behaviors)
    pub velocity_dir: Vec2,
}

impl BulletMotionState {
    pub fn new(spawn_center: Vec2) -> Self {
        Self {
            elapsed: 0.0,
            spawn_center,
            initial_offset: Vec2::ZERO,
            initial_angle: 0.0,
            initial_radius: 0.0,
            velocity_dir: Vec2::NEG_Y,
        }
    }

    pub fn with_offset(mut self, offset: Vec2) -> Self {
        self.initial_offset = offset;
        self
    }

    pub fn with_angle(mut self, angle: f32) -> Self {
        self.initial_angle = angle;
        self.velocity_dir = Vec2::new(angle.cos(), angle.sin());
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
/// Contains the declarative behavior list (used for to_wasm_call mapping).
///
/// 弹幕的行为栈组件。
/// 包含声明式行为列表（用于 to_wasm_call 映射）。
#[derive(Component, Clone, Default)]
pub struct BehaviorStack {
    /// List of active behaviors
    pub behaviors: Vec<BulletBehavior>,
}

impl BehaviorStack {
    pub fn new(behaviors: Vec<BulletBehavior>) -> Self {
        Self { behaviors }
    }
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
    /// Entity ID of the bullet container for this performance
    pub container_entity: Option<Entity>,
}

impl PerformancePlayer {
    pub fn new(spawn_center: Vec2) -> Self {
        Self {
            elapsed: 0.0,
            next_event_index: 0,
            finished: false,
            spawn_center,
            container_entity: None,
        }
    }
}

/// Component that links a PerformancePlayer to its loaded performance asset.
///
/// 将 PerformancePlayer 与其加载的演出资产关联的组件。
#[derive(Component)]
pub struct PerformanceHandle(pub Handle<super::danmaku_schema::DanmakuPerformance>);

/// Marker component for performance player entities.
///
/// 演出播放器实体的标记组件。
#[derive(Component)]
pub struct PerformancePlayerMarker;

// ============================================================================
// Active Danmaku Behavior Component
// ============================================================================

/// Component that holds an active danmaku behavior instance from a WASM mod.
/// Manages the lifecycle of the WASM instance (on_enter/on_update/on_exit).
///
/// 持有来自 WASM 模组的活跃弹幕行为实例的组件。
/// 管理 WASM 实例的生命周期（on_enter/on_update/on_exit）。
pub use crate::core::mod_system::ActiveDanmaku;

/// Stack of active WASM danmaku instances.
/// Each bullet can have multiple behaviors running simultaneously.
///
/// 活跃 WASM 弹幕实例栈。
/// 每个弹幕可以同时运行多个行为。
pub use crate::core::mod_system::ActiveDanmakuStack;
