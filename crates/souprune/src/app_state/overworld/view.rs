//! Overworld-specific View logic
//!
//! Overworld 特定的 View 逻辑
//!
//! This module contains View systems that are specific to the Overworld state,
//! such as input-to-FRE event bridging.
//!
//! 此模块包含特定于 Overworld 状态的 View 系统，
//! 例如输入到 FRE 事件的桥接。
//!
//! NOTE: Navigation logic has been moved to FRE rules in backpack.fre.ron.
//! The old hardcoded navigation systems have been removed.
//!
//! 注意：导航逻辑已移至 backpack.fre.ron 中的 FRE 规则。
//! 旧的硬编码导航系统已被移除。

use bevy::prelude::*;
use bevy_fact_rule_event::FactEvent;
use leafwing_input_manager::action_state::ActionState;

use crate::app_state::overworld::character::components::PlayerControlled;
use crate::core::input::{Action, ActionRegistry, ActionStateExt};
use crate::core::state_config::LoadedStateConfig;

use super::OverworldSubState;

/// System that bridges keyboard/gamepad input to FRE events.
///
/// This system converts player input actions (Up, Down, Confirm, Cancel, etc.)
/// into FRE FactEvents that can trigger rules defined in .fre.ron files.
///
/// The events are only sent when the player is in a state that has
/// `player_movable: false` (i.e., UI/menu states).
///
/// 将键盘/手柄输入桥接到 FRE 事件的系统。
///
/// 此系统将玩家输入动作（上、下、确认、取消等）转换为
/// 可以触发 .fre.ron 文件中定义的规则的 FRE FactEvent。
///
/// 仅当玩家处于 `player_movable: false` 的状态（即 UI/菜单状态）时才发送事件。
///
/// NOTE: This system emits legacy string events (input_nav_up, etc.) for
/// backward compatibility with older .fre.ron files. New rules should use
/// ActionEvent format instead.
///
/// 注意：此系统为了向后兼容旧的 .fre.ron 文件，会发出旧式字符串事件
/// （input_nav_up 等）。新规则应使用 ActionEvent 格式。
pub(crate) fn input_to_fre_event_bridge_system(
    registry: Res<ActionRegistry>,
    state_config: Option<Res<LoadedStateConfig>>,
    current_state: Res<State<OverworldSubState>>,
    query: Query<&ActionState<Action>, With<PlayerControlled>>,
    mut event_writer: MessageWriter<FactEvent>,
) {
    // Only process input in non-movable states (UI states)
    // 仅在不可移动状态（UI 状态）中处理输入
    let player_movable = state_config
        .as_ref()
        .and_then(|config| config.0.states.get(&current_state.0))
        .map(|def| def.player_movable)
        .unwrap_or(true);

    if player_movable {
        return;
    }

    let Ok(action_state) = query.single() else {
        return;
    };

    // Check navigation inputs and emit FRE events
    // 检查导航输入并发出 FRE 事件
    if action_state.action_just_pressed(&registry, "Up") {
        debug!("Input bridge: Up pressed, emitting input_nav_up");
        event_writer.write(FactEvent::new("input_nav_up"));
    }

    if action_state.action_just_pressed(&registry, "Down") {
        debug!("Input bridge: Down pressed, emitting input_nav_down");
        event_writer.write(FactEvent::new("input_nav_down"));
    }

    if action_state.action_just_pressed(&registry, "Left") {
        debug!("Input bridge: Left pressed, emitting input_nav_left");
        event_writer.write(FactEvent::new("input_nav_left"));
    }

    if action_state.action_just_pressed(&registry, "Right") {
        debug!("Input bridge: Right pressed, emitting input_nav_right");
        event_writer.write(FactEvent::new("input_nav_right"));
    }

    // Check UI inputs
    // 检查 UI 输入
    if action_state.action_just_pressed(&registry, "Confirm") {
        debug!("Input bridge: Confirm pressed, emitting input_confirm");
        event_writer.write(FactEvent::new("input_confirm"));
    }

    if action_state.action_just_pressed(&registry, "Cancel") {
        debug!("Input bridge: Cancel pressed, emitting input_cancel");
        event_writer.write(FactEvent::new("input_cancel"));
    }

    if action_state.action_just_pressed(&registry, "Menu") {
        debug!("Input bridge: Menu pressed, emitting input_menu");
        event_writer.write(FactEvent::new("input_menu"));
    }
}
