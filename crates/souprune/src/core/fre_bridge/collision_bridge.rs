//! Bridges collision events from the Physics phase into FRE FactEvents.
//!
//! 将物理阶段产生的碰撞事件桥接为 FRE FactEvent。
//!
//! This system drains the `CollisionEventBuffer` and emits a FRE `FactEvent`
//! for each unique collision, applying cooldown throttling.

use bevy::prelude::*;
use bevy_fact_rule_event::FactEvent;

use crate::core::collision::{CollisionEventBuffer, CollisionType};
use crate::core::event_phase::{CollisionCooldowns, EventThrottleConfig};
use crate::core::trace::{EventPhase, EventTraceLog};

/// Drain buffered collision events, apply throttling, and emit FRE events.
///
/// 消费缓冲的碰撞事件，应用节流，并发出 FRE 事件。
pub fn collision_to_fact_bridge_system(
    mut buffer: ResMut<CollisionEventBuffer>,
    mut fre_events: MessageWriter<FactEvent>,
    mut cooldowns: ResMut<CollisionCooldowns>,
    throttle: Res<EventThrottleConfig>,
    time: Res<Time>,
    mut trace_log: ResMut<EventTraceLog>,
) {
    let events = buffer.drain();
    if events.is_empty() {
        return;
    }

    let now = time.elapsed_secs_f64();
    let mut emitted = 0usize;

    for event in &events {
        // Apply collision cooldown.
        if !cooldowns.is_allowed(
            event.entity_a,
            event.entity_b,
            now,
            throttle.collision_cooldown,
        ) {
            continue;
        }
        cooldowns.record(event.entity_a, event.entity_b, now);

        let event_name = match event.collision_type {
            CollisionType::Trigger => "collision.trigger",
            CollisionType::Boundary => "collision.boundary",
        };

        fre_events.write(FactEvent::new(event_name.to_string()));
        emitted += 1;

        trace_log.record(
            EventPhase::Physics,
            "CollisionEvent",
            format!(
                "{:?} → {:?} ({})",
                event.entity_a, event.entity_b, event_name
            ),
            now,
            None,
        );
    }

    // Periodically prune stale cooldown entries (every ~5 seconds worth of entries).
    if cooldowns.last_collision.len() > 200 {
        cooldowns.prune(now, 5.0);
    }

    if emitted > 0 {
        debug!(
            "CollisionToFactBridge: {} collisions → {} FRE events (of {} buffered)",
            events.len(),
            emitted,
            events.len()
        );
    }
}
