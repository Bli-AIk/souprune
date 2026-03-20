//! FRE (Fact-Rule-Event) Bridge Module
//!
//! This module provides the bridge between game systems and the FRE rule engine.
//! It handles:
//! - Converting input actions to FRE events (ActionEvent)
//! - Processing FRE actions (PlaySound, SetLocalFact, CloseView, etc.) on ViewRoot
//! - Managing ActiveView markers
//!
//! FRE（Fact-Rule-Event）桥接模块
//!
//! 此模块提供游戏系统和 FRE 规则引擎之间的桥接。
//! 它处理：
//! - 将输入动作转换为 FRE 事件（ActionEvent）
//! - 在 ViewRoot 上处理 FRE 动作（PlaySound、SetLocalFact、CloseView 等）
//! - 管理 ActiveView 标记

mod custom_dispatch;
mod eval;
mod item_actions;
mod state_sync;
mod view_actions;

pub use eval::evaluate_single_condition;
use eval::{evaluate_conditions, evaluate_local_fact_value, register_condition_evaluator_system};
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
        app.add_message::<FreCustomActionEvent>()
            .add_systems(Startup, register_condition_evaluator_system)
            .add_systems(
                schedule,
                (
                    state_sync::sync_state_to_facts_system
                        .run_if(state_sync::state_facts_need_sync),
                    state_sync::action_to_fre_event_system,
                    view_actions::process_view_actions_system,
                    custom_dispatch::dispatch_custom_actions_system,
                    view_actions::handle_switch_state_system,
                )
                    .chain(),
            );
    }
}
