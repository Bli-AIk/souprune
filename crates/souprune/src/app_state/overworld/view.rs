//! Overworld-specific View logic
//!
//! Overworld 特定的 View 逻辑
//!
//! This module contains View systems that are specific to the Overworld state,
//! such as backpack menu handling and input-to-FRE event bridging.
//!
//! 此模块包含特定于 Overworld 状态的 View 系统，
//! 例如背包菜单处理和输入到 FRE 事件的桥接。

use bevy::prelude::*;
use bevy_fact_rule_event::{FactEvent, FactValue};
use leafwing_input_manager::action_state::ActionState;

use crate::app_state::overworld::character::components::PlayerControlled;
use crate::core::audio;
use crate::core::input::{Action, ActionRegistry, ActionStateExt};
use crate::core::state_config::LoadedStateConfig;
use crate::core::view::components::ViewRoot;

use super::OverworldSubState;

/// System that bridges keyboard/gamepad input to FRE events.
///
/// This system converts player input actions (Up, Down, Confirm, Cancel, etc.)
/// into FRE FactEvents that can trigger rules defined in .rules.ron files.
///
/// The events are only sent when the player is in a state that has
/// `player_movable: false` (i.e., UI/menu states).
///
/// 将键盘/手柄输入桥接到 FRE 事件的系统。
///
/// 此系统将玩家输入动作（上、下、确认、取消等）转换为
/// 可以触发 .rules.ron 文件中定义的规则的 FRE FactEvent。
///
/// 仅当玩家处于 `player_movable: false` 的状态（即 UI/菜单状态）时才发送事件。
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

/// System that directly handles navigation input and updates ViewRoot.local_facts.
///
/// This is a temporary solution that bypasses the FRE rule system and directly
/// manipulates ViewRoot.local_facts based on input. This enables basic navigation
/// while the full FRE-View integration is being developed.
///
/// 直接处理导航输入并更新 ViewRoot.local_facts 的系统。
///
/// 这是一个临时解决方案，绕过 FRE 规则系统，根据输入直接操作 ViewRoot.local_facts。
/// 这使得基本导航在完整的 FRE-View 集成开发期间能够工作。
#[cfg(all(feature = "bevy_kira_audio", not(feature = "firewheel")))]
pub(crate) fn view_local_facts_navigation_system(
    audio: Res<bevy_kira_audio::Audio>,
    asset_server: Res<AssetServer>,
    registry: Res<ActionRegistry>,
    state_config: Option<Res<LoadedStateConfig>>,
    current_state: Res<State<OverworldSubState>>,
    input_query: Query<&ActionState<Action>, With<PlayerControlled>>,
    mut view_root_query: Query<&mut ViewRoot>,
    mut next_state: ResMut<NextState<OverworldSubState>>,
) {
    // Only process input in non-movable states (UI states)
    let player_movable = state_config
        .as_ref()
        .and_then(|config| config.0.states.get(&current_state.0))
        .map(|def| def.player_movable)
        .unwrap_or(true);

    if player_movable {
        return;
    }

    let Ok(action_state) = input_query.single() else {
        return;
    };

    for mut view_root in view_root_query.iter_mut() {
        // Get current depth and selection from local_facts
        let depth = view_root.local_facts.get_int("depth").unwrap_or(0);
        let selection = view_root.local_facts.get_int("selection").unwrap_or(0);

        // Get max selection based on depth
        let max_selection = get_max_selection_for_depth(depth);

        // Handle navigation
        if action_state.action_just_pressed(&registry, "Up") && selection > 0 {
            view_root
                .local_facts
                .set("selection", FactValue::Int(selection - 1));
            audio::play_sound(&audio, &asset_server, "choice.wav");
            debug!(
                "View navigation: selection {} -> {}",
                selection,
                selection - 1
            );
        }

        if action_state.action_just_pressed(&registry, "Down") && selection < max_selection {
            view_root
                .local_facts
                .set("selection", FactValue::Int(selection + 1));
            audio::play_sound(&audio, &asset_server, "choice.wav");
            debug!(
                "View navigation: selection {} -> {}",
                selection,
                selection + 1
            );
        }

        // Handle confirm
        if action_state.action_just_pressed(&registry, "Confirm") {
            handle_confirm(
                depth,
                selection,
                &mut view_root.local_facts,
                &audio,
                &asset_server,
            );
        }

        // Handle cancel
        if action_state.action_just_pressed(&registry, "Cancel") {
            handle_cancel(
                depth,
                &mut view_root.local_facts,
                &mut next_state,
                &audio,
                &asset_server,
            );
        }
    }
}

/// Firewheel audio backend variant
#[cfg(feature = "firewheel")]
pub(crate) fn view_local_facts_navigation_system(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    registry: Res<ActionRegistry>,
    state_config: Option<Res<LoadedStateConfig>>,
    current_state: Res<State<OverworldSubState>>,
    input_query: Query<&ActionState<Action>, With<PlayerControlled>>,
    mut view_root_query: Query<&mut ViewRoot>,
    mut next_state: ResMut<NextState<OverworldSubState>>,
) {
    // Only process input in non-movable states (UI states)
    let player_movable = state_config
        .as_ref()
        .and_then(|config| config.0.states.get(&current_state.0))
        .map(|def| def.player_movable)
        .unwrap_or(true);

    if player_movable {
        return;
    }

    let Ok(action_state) = input_query.single() else {
        return;
    };

    for mut view_root in view_root_query.iter_mut() {
        // Get current depth and selection from local_facts
        let depth = view_root.local_facts.get_int("depth").unwrap_or(0);
        let selection = view_root.local_facts.get_int("selection").unwrap_or(0);

        // Get max selection based on depth
        let max_selection = get_max_selection_for_depth(depth);

        // Handle navigation
        if action_state.action_just_pressed(&registry, "Up") && selection > 0 {
            view_root
                .local_facts
                .set("selection", FactValue::Int(selection - 1));
            audio::play_sound(&mut commands, &asset_server, "choice.wav");
            debug!(
                "View navigation: selection {} -> {}",
                selection,
                selection - 1
            );
        }

        if action_state.action_just_pressed(&registry, "Down") && selection < max_selection {
            view_root
                .local_facts
                .set("selection", FactValue::Int(selection + 1));
            audio::play_sound(&mut commands, &asset_server, "choice.wav");
            debug!(
                "View navigation: selection {} -> {}",
                selection,
                selection + 1
            );
        }

        // Handle confirm
        if action_state.action_just_pressed(&registry, "Confirm") {
            handle_confirm_firewheel(
                depth,
                selection,
                &mut view_root.local_facts,
                &mut commands,
                &asset_server,
            );
        }

        // Handle cancel
        if action_state.action_just_pressed(&registry, "Cancel") {
            handle_cancel_firewheel(
                depth,
                &mut view_root.local_facts,
                &mut next_state,
                &mut commands,
                &asset_server,
            );
        }
    }
}

/// Get the maximum selection index for a given depth level.
///
/// DEPTH MAPPING:
/// - depth 0: Main menu (ITEM/STAT) - max 1
/// - depth 1: Item list - max depends on inventory (simplified to 7)
/// - depth 2: Item options (USE/INFO/DROP) - max 2
/// - depth 3: Status page - max 0 (no navigation)
fn get_max_selection_for_depth(depth: i64) -> i64 {
    match depth {
        0 => 1, // ITEM, STAT
        1 => 7, // Item list (simplified)
        2 => 2, // USE, INFO, DROP
        3 => 0, // Status page
        _ => 0,
    }
}

#[cfg(all(feature = "bevy_kira_audio", not(feature = "firewheel")))]
fn handle_confirm(
    depth: i64,
    selection: i64,
    local_facts: &mut bevy_fact_rule_event::FactDatabase,
    audio: &bevy_kira_audio::Audio,
    asset_server: &AssetServer,
) {
    match depth {
        0 => {
            // Main menu - go to sub menu
            match selection {
                0 => {
                    // ITEM -> Item list
                    local_facts.set("depth", FactValue::Int(1));
                    local_facts.set("selection", FactValue::Int(0));
                }
                1 => {
                    // STAT -> Status page
                    local_facts.set("depth", FactValue::Int(3));
                    local_facts.set("selection", FactValue::Int(0));
                }
                _ => {}
            }
            audio::play_sound(audio, asset_server, "confirm.wav");
        }
        1 => {
            // Item list - go to options
            local_facts.set("depth", FactValue::Int(2));
            local_facts.set("selection", FactValue::Int(0));
            audio::play_sound(audio, asset_server, "confirm.wav");
        }
        2 => {
            // Item options - execute action (simplified: just go back)
            local_facts.set("depth", FactValue::Int(1));
            audio::play_sound(audio, asset_server, "confirm.wav");
        }
        _ => {}
    }
    debug!("View confirm: depth {} selection {}", depth, selection);
}

#[cfg(all(feature = "bevy_kira_audio", not(feature = "firewheel")))]
fn handle_cancel(
    depth: i64,
    local_facts: &mut bevy_fact_rule_event::FactDatabase,
    next_state: &mut ResMut<NextState<OverworldSubState>>,
    _audio: &bevy_kira_audio::Audio,
    _asset_server: &AssetServer,
) {
    match depth {
        0 => {
            // Main menu - close backpack
            next_state.set(OverworldSubState::new("Normal"));
        }
        1 => {
            // Item list - back to menu
            local_facts.set("depth", FactValue::Int(0));
            local_facts.set("selection", FactValue::Int(0));
        }
        2 => {
            // Item options - back to item list
            local_facts.set("depth", FactValue::Int(1));
        }
        3 => {
            // Status page - back to menu
            local_facts.set("depth", FactValue::Int(0));
            local_facts.set("selection", FactValue::Int(1));
        }
        _ => {}
    }
    debug!("View cancel: depth {}", depth);
}

#[cfg(feature = "firewheel")]
fn handle_confirm_firewheel(
    depth: i64,
    selection: i64,
    local_facts: &mut bevy_fact_rule_event::FactDatabase,
    commands: &mut Commands,
    asset_server: &AssetServer,
) {
    match depth {
        0 => {
            match selection {
                0 => {
                    local_facts.set("depth", FactValue::Int(1));
                    local_facts.set("selection", FactValue::Int(0));
                }
                1 => {
                    local_facts.set("depth", FactValue::Int(3));
                    local_facts.set("selection", FactValue::Int(0));
                }
                _ => {}
            }
            audio::play_sound(commands, asset_server, "confirm.wav");
        }
        1 => {
            local_facts.set("depth", FactValue::Int(2));
            local_facts.set("selection", FactValue::Int(0));
            audio::play_sound(commands, asset_server, "confirm.wav");
        }
        2 => {
            local_facts.set("depth", FactValue::Int(1));
            audio::play_sound(commands, asset_server, "confirm.wav");
        }
        _ => {}
    }
    debug!("View confirm: depth {} selection {}", depth, selection);
}

#[cfg(feature = "firewheel")]
fn handle_cancel_firewheel(
    depth: i64,
    local_facts: &mut bevy_fact_rule_event::FactDatabase,
    next_state: &mut ResMut<NextState<OverworldSubState>>,
    _commands: &mut Commands,
    _asset_server: &AssetServer,
) {
    match depth {
        0 => {
            next_state.set(OverworldSubState::new("Normal"));
        }
        1 => {
            local_facts.set("depth", FactValue::Int(0));
            local_facts.set("selection", FactValue::Int(0));
        }
        2 => {
            local_facts.set("depth", FactValue::Int(1));
        }
        3 => {
            local_facts.set("depth", FactValue::Int(0));
            local_facts.set("selection", FactValue::Int(1));
        }
        _ => {}
    }
    debug!("View cancel: depth {}", depth);
}
