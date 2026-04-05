//! Bridges Souprune's runtime state and UI systems into the FRE rule engine.
//!
//! 把 Souprune 的运行时状态与界面系统桥接进 FRE 规则引擎。
//!
//! This module is where input, state transitions, active views, and custom game
//! actions become FRE-readable facts or FRE-triggered side effects. It is not a
//! gameplay feature by itself; it is the translation layer that lets rules
//! observe the running game and push decisions back into the runtime.
//!
//! 这个模块负责把输入、状态切换、激活中的 View 以及自定义游戏动作转成
//! FRE 可读的事实或 FRE 触发的副作用。它本身不是玩法功能，而是一层翻译器：
//! 让规则系统看见正在运行的游戏，也让规则计算结果能够反过来驱动运行时。

mod collision_bridge;
mod custom_dispatch;
mod eval;
pub mod extensions;
mod item_actions;
mod state_sync;
mod view_actions;

pub use custom_dispatch::dispatch_custom_actions_system;
pub use eval::evaluate_single_condition;
use eval::{evaluate_conditions, evaluate_local_fact_value, register_condition_evaluator_system};
pub use extensions::ViewActionExtensions;
pub use view_actions::process_view_actions_system;

use bevy::prelude::*;
use std::collections::HashMap;

/// Event emitted for each FRE Custom action encountered during rule evaluation.
/// Systems can read this event to handle specific action types.
///
/// 在规则评估期间遇到的 FRE Custom action 会发出此事件。
/// 系统可以读取此事件来处理特定的 action 类型。
#[derive(Message, Debug, Clone)]
pub struct FreCustomActionEvent {
    pub action_type: String,
    pub params: HashMap<String, String>,
}

/// Plugin for FRE-View bridge systems.
///
/// FRE-View 桥接系统的插件。
pub struct FREBridgePlugin;

impl Plugin for FREBridgePlugin {
    fn build(&self, app: &mut App) {
        let schedule = crate::game_schedule(app);

        let mut extensions = ViewActionExtensions::default();
        register_item_action_extensions(&mut extensions);
        app.insert_resource(extensions);

        app.add_message::<FreCustomActionEvent>()
            .add_systems(Startup, register_condition_evaluator_system)
            .add_systems(
                schedule,
                (
                    state_sync::sync_state_to_facts_system
                        .run_if(state_sync::state_facts_need_sync),
                    state_sync::action_to_fre_event_system,
                    state_sync::mode_change_to_fre_event_system,
                    collision_bridge::collision_to_fact_bridge_system,
                    view_actions::process_view_actions_system,
                    custom_dispatch::dispatch_custom_actions_system,
                    view_actions::handle_switch_state_system,
                )
                    .chain(),
            );
    }
}

/// Register item-specific Custom action handlers (UseItem, CheckItem, DropItem).
///
/// These handlers are invoked synchronously during view rule processing,
/// maintaining fact visibility for subsequent actions in the same rule.
fn register_item_action_extensions(extensions: &mut ViewActionExtensions) {
    extensions.register("UseItem", |params, ctx| {
        let index_expr = params.get("index_expr").map(|s| s.as_str()).unwrap_or("");
        let start_dialogue = params
            .get("start_dialogue")
            .map_or(false, |v| v == "true");
        item_actions::execute_use_item(
            index_expr,
            ctx.local_facts,
            ctx.global_facts,
            ctx.audio,
            ctx.asset_server,
            ctx.enum_registry,
            ctx.item_registry,
            start_dialogue,
            &ctx.config.game.dialogue_view_default,
            &ctx.config.game.dialogue_voice_default,
        );
    });

    extensions.register("CheckItem", |params, ctx| {
        let index_expr = params.get("index_expr").map(|s| s.as_str()).unwrap_or("");
        item_actions::execute_check_item(
            index_expr,
            ctx.local_facts,
            ctx.global_facts,
            ctx.enum_registry,
            ctx.item_registry,
            &ctx.config.game.dialogue_view_default,
            &ctx.config.game.dialogue_voice_default,
        );
    });

    extensions.register("DropItem", |params, ctx| {
        let index_expr = params.get("index_expr").map(|s| s.as_str()).unwrap_or("");
        item_actions::execute_drop_item(
            index_expr,
            ctx.local_facts,
            ctx.global_facts,
            ctx.enum_registry,
            ctx.item_registry,
            &ctx.config.game.dialogue_view_default,
            &ctx.config.game.dialogue_voice_default,
        );
    });
}
