//! Per-call state passed through the WASM host interface.
//!
//! Stores input snapshots, world facts, and side effects for the duration of a
//! single callback into a mod component.
//!
//! WASM 宿主接口使用的单次调用状态。
//! 保存一次 mod 回调期间的输入快照、世界事实与待提交副作用。

use bevy::prelude::*;
use bevy_fact_rule_event::FactValue;
use std::collections::HashMap;
use std::sync::Arc;

use super::effects::{PendingHostEffect, PendingSideEffects, PendingSoundEffect};

/// Per-call context: set by the host before invoking a mod callback.
#[derive(Default)]
pub struct CallContext {
    pub input_pressed: [bool; 7],
    pub input_just_pressed: [bool; 7],
    pub velocity: Vec2,
    pub entity_position: Vec2,
    pub delta_time: f32,
    /// Snapshot of the current FRE fact database (key → typed value).
    pub fact_snapshot: Arc<HashMap<String, FactValue>>,
    /// Fact mutations queued by the mod during a callback; applied afterwards.
    pub pending_fact_mutations: Vec<(String, FactValue)>,
    /// Persistent fact mutations queued by the mod during a callback.
    pub pending_global_fact_mutations: Vec<(String, FactValue)>,
    /// FRE events queued by the mod during a callback; emitted afterwards.
    pub pending_events: Vec<String>,
    /// Named entity positions for `get-entity-position-by-tag` queries.
    pub entity_positions_by_tag: HashMap<String, Vec2>,
    /// Current mode name for `get-current-mode`.
    pub current_mode: Option<String>,
    /// Current sub-state name for `get-current-sub-state`.
    pub current_sub_state: String,
    /// Pending view open requests (view-id strings).
    pub pending_view_opens: Vec<String>,
    /// Whether a close-view was requested.
    pub pending_view_close: bool,
    /// Pending sound play requests.
    pub pending_sounds: Vec<PendingSoundEffect>,
    /// Pending emitter spawn requests: (pattern_id, position) → assigned handle.
    pub pending_emitter_spawns: Vec<(String, Vec2)>,
    /// Pending emitter despawn requests (handles).
    pub pending_emitter_despawns: Vec<u64>,
    /// Host-owned entity primitive effects queued by the mod during a callback.
    pub pending_host_effects: Vec<PendingHostEffect>,
    /// Counter for assigning emitter handles within a single callback invocation.
    pub emitter_handle_counter: u64,
    /// Counter for assigning host-owned entity primitive handles.
    pub host_entity_handle_counter: u64,
}

impl CallContext {
    /// Clear callback-scoped pending side effects before entering WASM.
    ///
    /// 进入 WASM 前清空回调范围内的待提交副作用。
    pub fn clear_pending_side_effects(&mut self) {
        self.pending_fact_mutations.clear();
        self.pending_global_fact_mutations.clear();
        self.pending_events.clear();
        self.pending_sounds.clear();
        self.pending_host_effects.clear();
    }

    /// Take all pending side effects after a WASM callback returns.
    ///
    /// WASM 回调返回后取出所有待提交副作用。
    pub fn take_pending_side_effects(&mut self) -> PendingSideEffects {
        PendingSideEffects {
            fact_mutations: std::mem::take(&mut self.pending_fact_mutations),
            global_fact_mutations: std::mem::take(&mut self.pending_global_fact_mutations),
            events: std::mem::take(&mut self.pending_events),
            sounds: std::mem::take(&mut self.pending_sounds),
            host_effects: std::mem::take(&mut self.pending_host_effects),
        }
    }

    pub(super) fn next_host_entity_handle(&mut self) -> u64 {
        self.host_entity_handle_counter = self.host_entity_handle_counter.saturating_add(1).max(1);
        self.host_entity_handle_counter
    }
}
