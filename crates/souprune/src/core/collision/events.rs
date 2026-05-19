//! Generic collision events and per-frame deduplication buffer.
//!
//! 通用碰撞事件和帧内去重缓冲区。
//!
//! Collision systems (battle, top_down) report collisions into the
//! `CollisionEventBuffer`. At the end of the Physics phase, the buffer
//! is drained and deduped events are emitted as `CollisionEvent` messages.

use bevy::ecs::message::Message;
use bevy::prelude::*;
use std::collections::HashSet;

/// Type of collision detected.
///
/// 检测到的碰撞类型。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CollisionType {
    /// Trigger collision (e.g. bullet hitting player).
    Trigger,
    /// Boundary collision (e.g. entity hitting arena wall).
    Boundary,
}

/// A collision event — the framework's generic collision report.
///
/// 碰撞事件 — 框架核心的通用碰撞报告。
#[derive(Message, Debug, Clone)]
pub struct CollisionEvent {
    pub entity_a: Entity,
    pub entity_b: Entity,
    pub collision_type: CollisionType,
}

/// Per-frame collision event aggregator with entity-pair deduplication.
///
/// 帧内碰撞事件聚合器，按实体对去重。
/// 同一对实体在同一帧内只报告一次碰撞。
#[derive(Resource, Default)]
pub struct CollisionEventBuffer {
    events: Vec<CollisionEvent>,
    seen_pairs: HashSet<(Entity, Entity)>,
}

impl CollisionEventBuffer {
    /// Report a collision. Duplicates (same entity pair, any order) are ignored.
    pub fn report(&mut self, event: CollisionEvent) {
        let pair = (
            event.entity_a.min(event.entity_b),
            event.entity_a.max(event.entity_b),
        );
        if self.seen_pairs.insert(pair) {
            self.events.push(event);
        }
    }

    /// Drain all buffered events, clearing the buffer for the next frame.
    pub fn drain(&mut self) -> Vec<CollisionEvent> {
        self.seen_pairs.clear();
        std::mem::take(&mut self.events)
    }

    /// Number of unique collisions buffered this frame.
    pub fn len(&self) -> usize {
        self.events.len()
    }

    /// Whether the buffer is empty.
    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entity(id: u32) -> Entity {
        Entity::from_raw_u32(id).unwrap()
    }

    #[test]
    fn test_deduplication() {
        let mut buffer = CollisionEventBuffer::default();
        buffer.report(CollisionEvent {
            entity_a: entity(1),
            entity_b: entity(2),
            collision_type: CollisionType::Trigger,
        });
        // Same pair, same order.
        buffer.report(CollisionEvent {
            entity_a: entity(1),
            entity_b: entity(2),
            collision_type: CollisionType::Trigger,
        });
        // Same pair, reversed order.
        buffer.report(CollisionEvent {
            entity_a: entity(2),
            entity_b: entity(1),
            collision_type: CollisionType::Trigger,
        });
        assert_eq!(buffer.len(), 1);
        let events = buffer.drain();
        assert_eq!(events.len(), 1);
        assert!(buffer.is_empty());
    }

    #[test]
    fn test_different_pairs() {
        let mut buffer = CollisionEventBuffer::default();
        buffer.report(CollisionEvent {
            entity_a: entity(1),
            entity_b: entity(2),
            collision_type: CollisionType::Trigger,
        });
        buffer.report(CollisionEvent {
            entity_a: entity(1),
            entity_b: entity(3),
            collision_type: CollisionType::Boundary,
        });
        assert_eq!(buffer.len(), 2);
    }
}
