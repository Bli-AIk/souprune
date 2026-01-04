//! # components.rs
//!
//! ## Module Overview
//!
//! Defines ECS components for the danmaku system.
//!
//! 定义弹幕系统的 ECS 组件。

use super::patterns::BulletBehavior;
use bevy::prelude::*;
use souprune_api::{DanmakuInstance, PropC};
use std::collections::HashMap;
use std::ffi::CString;

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
/// Contains a list of active behaviors that are evaluated each frame.
///
/// 弹幕的行为栈组件。
/// 包含每帧都会评估的活跃行为列表。
#[derive(Component, Clone, Default)]
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
pub struct PerformanceHandle(pub Handle<super::patterns::DanmakuPerformance>);

/// Marker component for performance player entities.
///
/// 演出播放器实体的标记组件。
#[derive(Component)]
pub struct PerformancePlayerMarker;

// ============================================================================
// Active Danmaku Behavior Component
// ============================================================================

/// Component that holds an active danmaku behavior instance from a mod.
/// Manages the lifecycle of the FFI instance (on_enter/on_update/on_exit).
///
/// 持有来自模组的活跃弹幕行为实例的组件。
/// 管理 FFI 实例的生命周期（on_enter/on_update/on_exit）。
#[derive(Component)]
pub struct ActiveDanmaku {
    instance: DanmakuInstance,
    /// Whether on_enter has been called
    initialized: bool,
    /// Properties from RON config (named)
    pub props: HashMap<String, f32>,
    /// Legacy parameters from RON config (indexed)
    pub params: Vec<f32>,

    /// Cached FFI-compatible properties
    cached_props: Vec<PropC>,
    /// Keep CStrings alive for the pointers in cached_props
    #[allow(dead_code)]
    cached_names: Vec<CString>,
}

// SAFETY: The DanmakuInstance is accessed only from the main thread
unsafe impl Send for ActiveDanmaku {}
unsafe impl Sync for ActiveDanmaku {}

impl ActiveDanmaku {
    pub fn new(instance: DanmakuInstance, props: HashMap<String, f32>, params: Vec<f32>) -> Self {
        let mut cached_names = Vec::with_capacity(props.len());
        let mut cached_props = Vec::with_capacity(props.len());

        for (name, value) in &props {
            let c_name = CString::new(name.as_str()).unwrap_or_default();
            cached_props.push(PropC {
                name: c_name.as_ptr() as *const u8,
                name_len: name.len(),
                value: *value,
            });
            cached_names.push(c_name);
        }

        Self {
            instance,
            initialized: false,
            props,
            params,
            cached_props,
            cached_names,
        }
    }

    /// Check if on_enter has been called
    pub fn is_initialized(&self) -> bool {
        self.initialized
    }

    /// Call on_enter (should only be called once)
    pub fn call_on_enter(&mut self, ctx: &souprune_api::BulletContextC) {
        if !self.initialized {
            if let Some(on_enter) = self.instance.vtable.on_enter {
                on_enter(self.instance.instance, ctx);
            }
            self.initialized = true;
        }
    }

    /// Get cached props for FFI
    pub fn ffi_props(&self) -> (*const PropC, usize) {
        (self.cached_props.as_ptr(), self.cached_props.len())
    }

    /// Call on_update and return the output
    pub fn call_on_update(
        &mut self,
        ctx: &souprune_api::BulletContextC,
    ) -> souprune_api::BulletOutputC {
        if let Some(on_update) = self.instance.vtable.on_update {
            on_update(self.instance.instance, ctx)
        } else {
            souprune_api::BulletOutputC::default()
        }
    }
}

impl Drop for ActiveDanmaku {
    fn drop(&mut self) {
        // Call on_exit if initialized
        if self.initialized
            && let Some(on_exit) = self.instance.vtable.on_exit
        {
            on_exit(self.instance.instance);
        }
        // Critical: Call destroy to free memory on the guest side
        if let Some(destroy) = self.instance.vtable.destroy {
            destroy(self.instance.instance);
        }
    }
}
