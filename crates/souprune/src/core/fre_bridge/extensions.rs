//! Extensible dispatch for Custom FRE actions within view rule processing.
//!
//! View 规则处理中自定义 FRE 动作的可扩展分发机制。
//!
//! When a rule action is `Custom { action_type, params }`, the view action
//! executor looks up the `action_type` in [`ViewActionExtensions`] and calls
//! the registered handler with full view-level context (local facts, global
//! facts, audio, etc.). This allows game-specific actions (like UseItem) to
//! be dispatched synchronously alongside core actions within the same rule,
//! while keeping `GameActionDef` free of game-specific variants.

use bevy::prelude::*;
use std::collections::HashMap;

/// Execution context passed to extension handlers.
///
/// Contains all the same resources available to built-in actions
/// in `execute_action`, enabling extensions to modify view-local
/// and global facts, play sounds, etc.
pub struct ViewActionExecCtx<'a> {
    pub local_facts: &'a mut bevy_fact_rule_event::FactDatabase,
    pub global_facts: &'a mut bevy_fact_rule_event::LayeredFactDatabase,
    pub audio: &'a bevy_kira_audio::Audio,
    pub asset_server: &'a AssetServer,
    pub audio_cache: &'a mut crate::core::audio::AudioSourceCache,
    pub enum_registry: &'a bevy_fact_rule_event::EnumRegistry,
    pub config: &'a crate::config::SoupruneConfig,
    pub fact_history: &'a mut crate::core::trace::FactChangeHistory,
    pub frame_number: u64,
    pub rule_id: &'a str,
}

type ViewActionHandlerFn =
    Box<dyn Fn(&HashMap<String, String>, &mut ViewActionExecCtx) + Send + Sync>;

/// Registry of handlers for `Custom` action types within view rule processing.
///
/// Handlers are invoked synchronously during rule evaluation, ensuring
/// that fact changes are visible to subsequent actions in the same rule.
#[derive(Resource, Default)]
pub struct ViewActionExtensions {
    handlers: HashMap<String, ViewActionHandlerFn>,
}

impl ViewActionExtensions {
    pub fn register(
        &mut self,
        action_type: impl Into<String>,
        handler: impl Fn(&HashMap<String, String>, &mut ViewActionExecCtx) + Send + Sync + 'static,
    ) {
        self.handlers.insert(action_type.into(), Box::new(handler));
    }

    /// Try to handle a Custom action. Returns `true` if a handler was found.
    pub fn handle(
        &self,
        action_type: &str,
        params: &HashMap<String, String>,
        ctx: &mut ViewActionExecCtx,
    ) -> bool {
        if let Some(handler) = self.handlers.get(action_type) {
            handler(params, ctx);
            true
        } else {
            false
        }
    }
}
