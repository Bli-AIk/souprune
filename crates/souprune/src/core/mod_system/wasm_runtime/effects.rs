//! Pending side-effect types produced by WASM callbacks.
//!
//! Defines the host-side work queued while a WASM function runs and drained
//! after control returns to Bevy.
//!
//! WASM 回调产生的待提交副作用类型。
//! 定义 WASM 函数运行期间排队、返回 Bevy 后再提交的宿主侧工作。

use bevy::prelude::*;
use bevy_fact_rule_event::FactValue;

use crate::core::collision::{PhysicsCollider, TriggerCollider};

/// Host-owned side effects requested by a WASM callback.
///
/// WASM 回调请求的宿主侧副作用。
#[derive(Debug, Clone, PartialEq)]
pub enum PendingHostEffect {
    /// Spawn a visible ViewBox primitive and bind it to the opaque handle.
    ///
    /// 生成可见 ViewBox primitive，并绑定到不透明句柄。
    SpawnViewBox {
        handle: u64,
        center: Vec2,
        size: Vec2,
        border_width: f32,
    },
    /// Spawn a visible sprite entity primitive and bind it to the opaque handle.
    ///
    /// 生成可见 sprite 实体 primitive，并绑定到不透明句柄。
    SpawnSpriteEntity {
        handle: u64,
        texture: String,
        position: Vec2,
        z: f32,
        color: Color,
        physics_collider: Option<PhysicsCollider>,
        trigger_collider: Option<TriggerCollider>,
        behavior_id: Option<String>,
        behavior_context: Option<String>,
        bullet_target: bool,
        mode_scope: Option<String>,
        name: Option<String>,
    },
    /// Update a ViewBox primitive's center and size.
    ///
    /// 更新 ViewBox primitive 的中心点和尺寸。
    SetViewBoxBounds {
        handle: u64,
        center: Vec2,
        size: Vec2,
    },
    /// Tween a ViewBox primitive's center and size on the host.
    ///
    /// 在宿主侧对 ViewBox primitive 的中心点和尺寸执行 tween。
    TweenViewBoxBounds {
        handle: u64,
        center: Vec2,
        size: Vec2,
        duration_secs: f32,
    },
    /// Update a ViewBox primitive's visibility.
    ///
    /// 更新 ViewBox primitive 的可见性。
    SetViewBoxVisible { handle: u64, visible: bool },
    /// Remove a host-owned entity primitive.
    ///
    /// 移除宿主拥有的实体 primitive。
    RemoveEntity { handle: u64 },
}

/// Audio playback requested by a WASM callback.
///
/// WASM 回调请求的音频播放。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PendingSoundEffect {
    /// Resolve the value through the configured audio resource roots.
    ///
    /// 通过配置的音频资源根目录解析此值。
    SoundKey(String),
    /// Load the value as a full Bevy asset path.
    ///
    /// 将此值作为完整 Bevy 资源路径加载。
    FullPath(String),
}

/// Pending side effects collected after a WASM callback returns.
///
/// WASM 回调返回后收集的待提交副作用。
#[derive(Default, Debug, Clone, PartialEq)]
pub struct PendingSideEffects {
    pub fact_mutations: Vec<(String, FactValue)>,
    pub global_fact_mutations: Vec<(String, FactValue)>,
    pub events: Vec<String>,
    pub sounds: Vec<PendingSoundEffect>,
    pub host_effects: Vec<PendingHostEffect>,
}

impl PendingSideEffects {
    /// Return whether this bundle has no queued side effects.
    ///
    /// 返回此副作用集合是否为空。
    pub fn is_empty(&self) -> bool {
        self.fact_mutations.is_empty()
            && self.global_fact_mutations.is_empty()
            && self.events.is_empty()
            && self.sounds.is_empty()
            && self.host_effects.is_empty()
    }
}
