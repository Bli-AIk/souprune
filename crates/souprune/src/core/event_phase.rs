//! Three-phase event scheduling: Physics → Logic → Presentation.
//!
//! 三阶段事件调度：物理 → 逻辑 → 表现。
//!
//! Systems are assigned to an `EventPhaseSet` to enforce a strict ordering within
//! each frame. Physics produces collision events, Logic consumes them via FRE rules,
//! and Presentation plays sounds/animations. No reverse propagation is allowed.

use bevy::prelude::*;

/// Event processing phase within a frame.
///
/// 帧内事件处理阶段。执行顺序严格为 Physics → Logic → Presentation。
/// 后一层只能消费前一层产生的事件，不允许反向传播。
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum EventPhaseSet {
    /// Physics-layer events (collisions, boundaries) — high frequency, engine-internal.
    ///
    /// 物理层事件（碰撞、边界）— 高频，引擎内部消费。
    Physics,
    /// Logic-layer events (FRE rule evaluation) — medium frequency, rule engine.
    ///
    /// 逻辑层事件（FRE 规则评估）— 中频，规则引擎消费。
    Logic,
    /// Presentation-layer events (sound, VFX, UI updates) — low frequency.
    ///
    /// 表现层事件（音效、特效、UI 更新）— 低频。
    Presentation,
}

/// Event throttle configuration — prevents the same event from triggering rules
/// repeatedly within a short window.
///
/// 事件节流配置 — 防止同一事件在短时间内重复触发规则。
#[derive(Resource)]
pub struct EventThrottleConfig {
    /// Minimum interval between collision events for the same entity pair (seconds).
    ///
    /// 同一实体对碰撞事件的最小间隔（秒）。
    pub collision_cooldown: f32,
    /// Minimum interval between same-name FRE events (seconds). 0 = no throttling.
    ///
    /// 同名 FRE 事件的最小间隔（秒）。0 = 不节流。
    pub fre_event_cooldown: f32,
}

impl Default for EventThrottleConfig {
    fn default() -> Self {
        Self {
            collision_cooldown: 0.1,
            fre_event_cooldown: 0.0,
        }
    }
}

/// Collision cooldown tracker — records last collision time per entity pair.
///
/// 碰撞冷却追踪器 — 记录每对实体的上次碰撞时间。
#[derive(Resource, Default)]
pub struct CollisionCooldowns {
    /// (entity_a.min, entity_b.max) → last collision time in seconds.
    pub last_collision: std::collections::HashMap<(Entity, Entity), f64>,
}

impl CollisionCooldowns {
    /// Check if a collision between two entities is allowed (not in cooldown).
    pub fn is_allowed(&self, a: Entity, b: Entity, now: f64, cooldown: f32) -> bool {
        if cooldown <= 0.0 {
            return true;
        }
        let pair = (a.min(b), a.max(b));
        self.last_collision
            .get(&pair)
            .map_or(true, |&last| now - last >= cooldown as f64)
    }

    /// Record a collision occurrence.
    pub fn record(&mut self, a: Entity, b: Entity, now: f64) {
        let pair = (a.min(b), a.max(b));
        self.last_collision.insert(pair, now);
    }

    /// Prune stale entries older than `max_age` seconds.
    pub fn prune(&mut self, now: f64, max_age: f64) {
        self.last_collision
            .retain(|_, &mut last| now - last < max_age);
    }
}

pub struct EventPhasePlugin;

impl Plugin for EventPhasePlugin {
    fn build(&self, app: &mut App) {
        app.configure_sets(
            Update,
            (
                EventPhaseSet::Physics,
                EventPhaseSet::Logic,
                EventPhaseSet::Presentation,
            )
                .chain(),
        )
        .init_resource::<EventThrottleConfig>()
        .init_resource::<CollisionCooldowns>();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_collision_cooldown_allows_first_collision() {
        let cooldowns = CollisionCooldowns::default();
        let a = Entity::from_raw_u32(1).unwrap();
        let b = Entity::from_raw_u32(2).unwrap();
        assert!(cooldowns.is_allowed(a, b, 1.0, 0.1));
    }

    #[test]
    fn test_collision_cooldown_blocks_repeat() {
        let mut cooldowns = CollisionCooldowns::default();
        let a = Entity::from_raw_u32(1).unwrap();
        let b = Entity::from_raw_u32(2).unwrap();
        cooldowns.record(a, b, 1.0);
        assert!(!cooldowns.is_allowed(a, b, 1.05, 0.1));
        assert!(cooldowns.is_allowed(a, b, 1.15, 0.1));
    }

    #[test]
    fn test_collision_cooldown_symmetric() {
        let mut cooldowns = CollisionCooldowns::default();
        let a = Entity::from_raw_u32(1).unwrap();
        let b = Entity::from_raw_u32(2).unwrap();
        cooldowns.record(a, b, 1.0);
        // Order shouldn't matter.
        assert!(!cooldowns.is_allowed(b, a, 1.05, 0.1));
    }

    #[test]
    fn test_prune_removes_stale() {
        let mut cooldowns = CollisionCooldowns::default();
        let a = Entity::from_raw_u32(1).unwrap();
        let b = Entity::from_raw_u32(2).unwrap();
        cooldowns.record(a, b, 1.0);
        assert_eq!(cooldowns.last_collision.len(), 1);
        cooldowns.prune(10.0, 5.0);
        assert_eq!(cooldowns.last_collision.len(), 0);
    }
}
