//! # state.rs
//!
//! # state.rs 文件
//!
//! ## Module Overview
//!
//! ## 模块概述
//!
//! This module handles UI layer transitions and navigation using the unified
//! InteractiveLayer system for both OW and Battle contexts.
//!
//! 本模块使用统一的 InteractiveLayer 系统处理 OW 和 Battle 场景下
//! UI 层级转换和导航。
//!
//! ## Source File Overview
//!
//! ## 源文件概述
//!
//! It manages state changes triggered by user input such as confirm and cancel actions,
//! providing a unified experience for both Overworld and Battle modes.
//!
//! 管理由用户输入触发的状态变化（如确认和取消操作），
//! 为 Overworld 和 Battle 模式提供统一的体验。

use super::ron_view::UIGlobalTriggerConfig;
use crate::app_state::overworld::{OverworldState, character};
use crate::core::audio;
use crate::core::input::Action;
use bevy::ecs::message::MessageWriter;
use bevy::prelude::*;
use leafwing_input_manager::action_state::ActionState;

// ============================================================================
// Global Trigger System
// 全局触发器系统
// ============================================================================

/// Handle global triggers that can activate from any overworld state.
///
/// 处理可以从任何 Overworld 状态激活的全局触发器。
///
/// This system checks for global input triggers (like opening the backpack menu)
/// that should work regardless of the current state.
///
/// 该系统检查全局输入触发器（如打开背包菜单），
/// 这些触发器应该在任何当前状态下都能工作。
#[cfg(all(feature = "bevy_kira_audio", not(feature = "firewheel")))]
pub(crate) fn global_trigger_system(
    audio: Res<bevy_kira_audio::Audio>,
    asset_server: Res<AssetServer>,
    mut next_state: ResMut<NextState<OverworldState>>,
    current_state: Res<State<OverworldState>>,
    query: Query<&ActionState<Action>, With<character::components::PlayerControlled>>,
    global_trigger_config: Res<UIGlobalTriggerConfig>,
) {
    let Ok(action_state) = query.single() else {
        return;
    };

    for (action, rules) in &global_trigger_config.triggers {
        if action_state.just_pressed(action) {
            debug!(
                "Action pressed: {:?}, current state: {:?}, rules count: {}",
                action,
                current_state.get(),
                rules.len()
            );
            for rule in rules {
                debug!(
                    "Checking rule: target={:?}, allowed={:?}",
                    rule.target_state, rule.allowed_states
                );
                if rule.allowed_states.contains(current_state.get()) {
                    info!(
                        "Global trigger activated: {:?} -> {:?} via {:?}",
                        current_state.get(),
                        rule.target_state,
                        action
                    );

                    if let Some(sound_path) = &rule.sound {
                        audio::play_sound(&audio, &asset_server, sound_path);
                    }

                    next_state.set(rule.target_state);
                    return;
                }
            }
        }
    }
}

/// Firewheel audio backend variant of global trigger system.
#[cfg(feature = "firewheel")]
pub(crate) fn global_trigger_system(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut next_state: ResMut<NextState<OverworldState>>,
    current_state: Res<State<OverworldState>>,
    query: Query<&ActionState<Action>, With<character::components::PlayerControlled>>,
    global_trigger_config: Res<UIGlobalTriggerConfig>,
) {
    let Ok(action_state) = query.single() else {
        return;
    };

    for (action, rules) in &global_trigger_config.triggers {
        if action_state.just_pressed(action) {
            debug!(
                "Action pressed: {:?}, current state: {:?}, rules count: {}",
                action,
                current_state.get(),
                rules.len()
            );
            for rule in rules {
                debug!(
                    "Checking rule: target={:?}, allowed={:?}",
                    rule.target_state, rule.allowed_states
                );
                if rule.allowed_states.contains(current_state.get()) {
                    info!(
                        "Global trigger activated: {:?} -> {:?} via {:?}",
                        current_state.get(),
                        rule.target_state,
                        action
                    );

                    if let Some(sound_path) = &rule.sound {
                        audio::play_sound(&mut commands, &asset_server, sound_path);
                    }

                    next_state.set(rule.target_state);
                    return;
                }
            }
        }
    }
}

// ============================================================================
// Unified Interactive Layer Navigation System
// 统一的交互层导航系统
// ============================================================================

/// System that handles navigation input for all active InteractiveLayers.
///
/// 处理所有活跃 InteractiveLayer 导航输入的系统。
///
/// This is the unified navigation handler that works for both OW and Battle.
/// It tries to use ActionState from PlayerControlled entity first, then falls
/// back to raw keyboard input for Battle mode where no PlayerControlled exists.
///
/// 这是统一的导航处理器，适用于 OW 和 Battle。
/// 它首先尝试使用 PlayerControlled 实体的 ActionState，
/// 如果不存在则回退到原始键盘输入（用于 Battle 模式）。
#[cfg(all(feature = "bevy_kira_audio", not(feature = "firewheel")))]
pub(crate) fn handle_interactive_layer_navigation_system(
    audio: Res<bevy_kira_audio::Audio>,
    asset_server: Res<AssetServer>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut layer_query: Query<(Entity, &mut super::components::InteractiveLayer)>,
    player_query: Query<&ActionState<Action>, With<character::components::PlayerControlled>>,
    mut selection_changed_events: MessageWriter<super::components::SelectionChangedEvent>,
) {
    // Determine which input method to use
    let action_state = player_query.single().ok();

    for (entity, mut layer) in layer_query.iter_mut() {
        if !layer.is_active {
            continue;
        }

        let previous_index = layer.current_selection;
        let mut changed = false;

        // Check all navigation actions using ActionState or keyboard fallback
        for action in [Action::Up, Action::Down, Action::Left, Action::Right] {
            let pressed = if let Some(state) = action_state {
                state.just_pressed(&action)
            } else {
                // Fallback to raw keyboard input for Battle mode
                match action {
                    Action::Up => {
                        keyboard.just_pressed(KeyCode::ArrowUp)
                            || keyboard.just_pressed(KeyCode::KeyW)
                    }
                    Action::Down => {
                        keyboard.just_pressed(KeyCode::ArrowDown)
                            || keyboard.just_pressed(KeyCode::KeyS)
                    }
                    Action::Left => {
                        keyboard.just_pressed(KeyCode::ArrowLeft)
                            || keyboard.just_pressed(KeyCode::KeyA)
                    }
                    Action::Right => {
                        keyboard.just_pressed(KeyCode::ArrowRight)
                            || keyboard.just_pressed(KeyCode::KeyD)
                    }
                    _ => false,
                }
            };

            if pressed && layer.navigate(&action) {
                changed = true;
                break;
            }
        }

        if changed {
            // Play navigation sound if configured
            if let Some(sound_path) = &layer.sound_on_navigate {
                audio::play_sound(&audio, &asset_server, sound_path);
            }

            // Fire selection changed message
            selection_changed_events.write(super::components::SelectionChangedEvent {
                layer_id: layer.layer_id.clone(),
                previous_index,
                new_index: layer.current_selection,
                entity,
            });

            info!(
                "InteractiveLayer '{}' selection changed: {} -> {}",
                layer.layer_id, previous_index, layer.current_selection
            );
        }
    }
}

/// Firewheel audio backend variant
#[cfg(feature = "firewheel")]
pub(crate) fn handle_interactive_layer_navigation_system(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut layer_query: Query<(Entity, &mut super::components::InteractiveLayer)>,
    player_query: Query<&ActionState<Action>, With<character::components::PlayerControlled>>,
    mut selection_changed_events: MessageWriter<super::components::SelectionChangedEvent>,
) {
    let action_state = player_query.single().ok();

    for (entity, mut layer) in layer_query.iter_mut() {
        if !layer.is_active {
            continue;
        }

        let previous_index = layer.current_selection;
        let mut changed = false;

        for action in [Action::Up, Action::Down, Action::Left, Action::Right] {
            let pressed = if let Some(state) = action_state {
                state.just_pressed(&action)
            } else {
                match action {
                    Action::Up => {
                        keyboard.just_pressed(KeyCode::ArrowUp)
                            || keyboard.just_pressed(KeyCode::KeyW)
                    }
                    Action::Down => {
                        keyboard.just_pressed(KeyCode::ArrowDown)
                            || keyboard.just_pressed(KeyCode::KeyS)
                    }
                    Action::Left => {
                        keyboard.just_pressed(KeyCode::ArrowLeft)
                            || keyboard.just_pressed(KeyCode::KeyA)
                    }
                    Action::Right => {
                        keyboard.just_pressed(KeyCode::ArrowRight)
                            || keyboard.just_pressed(KeyCode::KeyD)
                    }
                    _ => false,
                }
            };

            if pressed && layer.navigate(&action) {
                changed = true;
                break;
            }
        }

        if changed {
            if let Some(sound_path) = &layer.sound_on_navigate {
                audio::play_sound(&mut commands, &asset_server, sound_path);
            }

            selection_changed_events.write(super::components::SelectionChangedEvent {
                layer_id: layer.layer_id.clone(),
                previous_index,
                new_index: layer.current_selection,
                entity,
            });

            info!(
                "InteractiveLayer '{}' selection changed: {} -> {}",
                layer.layer_id, previous_index, layer.current_selection
            );
        }
    }
}

// ============================================================================
// Unified Interactive Layer Confirm/Cancel System
// 统一的交互层确认/取消系统
// ============================================================================

/// System that handles confirm/cancel input for active InteractiveLayers.
///
/// 处理活跃 InteractiveLayer 确认/取消输入的系统。
///
/// Supports both ActionState (OW mode) and raw keyboard input (Battle mode).
///
/// 支持 ActionState（OW 模式）和原始键盘输入（Battle 模式）。
#[cfg(all(feature = "bevy_kira_audio", not(feature = "firewheel")))]
pub(crate) fn handle_interactive_layer_confirm_cancel_system(
    audio: Res<bevy_kira_audio::Audio>,
    asset_server: Res<AssetServer>,
    keyboard: Res<ButtonInput<KeyCode>>,
    layer_query: Query<(Entity, &super::components::InteractiveLayer)>,
    player_query: Query<&ActionState<Action>, With<character::components::PlayerControlled>>,
    mut confirm_events: MessageWriter<super::components::SelectionConfirmedEvent>,
    mut cancel_events: MessageWriter<super::components::SelectionCancelledEvent>,
) {
    let action_state = player_query.single().ok();

    for (entity, layer) in layer_query.iter() {
        if !layer.is_active {
            continue;
        }

        let confirm_pressed = if let Some(state) = action_state {
            state.just_pressed(&Action::Confirm)
        } else {
            // Fallback to raw keyboard input (Z or Enter for Confirm)
            keyboard.just_pressed(KeyCode::KeyZ) || keyboard.just_pressed(KeyCode::Enter)
        };

        let cancel_pressed = if let Some(state) = action_state {
            state.just_pressed(&Action::Cancel)
        } else {
            // Fallback to raw keyboard input (X or Escape for Cancel)
            keyboard.just_pressed(KeyCode::KeyX) || keyboard.just_pressed(KeyCode::Escape)
        };

        if confirm_pressed {
            // Play confirm sound
            audio::play_sound(&audio, &asset_server, "confirm.wav");

            info!(
                "InteractiveLayer '{}' confirmed at index {}",
                layer.layer_id, layer.current_selection
            );

            confirm_events.write(super::components::SelectionConfirmedEvent {
                layer_id: layer.layer_id.clone(),
                selected_index: layer.current_selection,
                selected_element: layer.current_element_name().map(|s| s.to_string()),
                entity,
            });
        }

        if cancel_pressed {
            info!("InteractiveLayer '{}' cancelled", layer.layer_id);

            cancel_events.write(super::components::SelectionCancelledEvent {
                layer_id: layer.layer_id.clone(),
                entity,
            });
        }
    }
}

/// Firewheel audio backend variant
#[cfg(feature = "firewheel")]
pub(crate) fn handle_interactive_layer_confirm_cancel_system(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    keyboard: Res<ButtonInput<KeyCode>>,
    layer_query: Query<(Entity, &super::components::InteractiveLayer)>,
    player_query: Query<&ActionState<Action>, With<character::components::PlayerControlled>>,
    mut confirm_events: MessageWriter<super::components::SelectionConfirmedEvent>,
    mut cancel_events: MessageWriter<super::components::SelectionCancelledEvent>,
) {
    let action_state = player_query.single().ok();

    for (entity, layer) in layer_query.iter() {
        if !layer.is_active {
            continue;
        }

        let confirm_pressed = if let Some(state) = action_state {
            state.just_pressed(&Action::Confirm)
        } else {
            keyboard.just_pressed(KeyCode::KeyZ) || keyboard.just_pressed(KeyCode::Enter)
        };

        let cancel_pressed = if let Some(state) = action_state {
            state.just_pressed(&Action::Cancel)
        } else {
            keyboard.just_pressed(KeyCode::KeyX) || keyboard.just_pressed(KeyCode::Escape)
        };

        if confirm_pressed {
            audio::play_sound(&mut commands, &asset_server, "confirm.wav");

            info!(
                "InteractiveLayer '{}' confirmed at index {}",
                layer.layer_id, layer.current_selection
            );

            confirm_events.write(super::components::SelectionConfirmedEvent {
                layer_id: layer.layer_id.clone(),
                selected_index: layer.current_selection,
                selected_element: layer.current_element_name().map(|s| s.to_string()),
                entity,
            });
        }

        if cancel_pressed {
            info!("InteractiveLayer '{}' cancelled", layer.layer_id);

            cancel_events.write(super::components::SelectionCancelledEvent {
                layer_id: layer.layer_id.clone(),
                entity,
            });
        }
    }
}

// ============================================================================
// Interactive Layer Transition System
// 交互层转换系统
// ============================================================================

/// System that processes SelectionConfirmedEvent and SelectionCancelledEvent
/// to perform layer transitions defined in InteractiveLayer.
///
/// 处理 SelectionConfirmedEvent 和 SelectionCancelledEvent
/// 以执行 InteractiveLayer 中定义的层级转换。
///
/// This system enables OW and Battle to share the same transition logic.
///
/// 该系统使 OW 和 Battle 可以共享相同的转换逻辑。
#[cfg(all(feature = "bevy_kira_audio", not(feature = "firewheel")))]
pub(crate) fn handle_interactive_layer_transitions_system(
    audio: Res<bevy_kira_audio::Audio>,
    asset_server: Res<AssetServer>,
    mut layer_query: Query<(Entity, &mut super::components::InteractiveLayer)>,
    mut confirm_events: bevy::ecs::message::MessageReader<
        super::components::SelectionConfirmedEvent,
    >,
    mut cancel_events: bevy::ecs::message::MessageReader<
        super::components::SelectionCancelledEvent,
    >,
    mut next_ow_state: ResMut<NextState<OverworldState>>,
    player_data: Res<crate::core::data::PlayerData>,
) {
    use super::ron_view::parsing::evaluate_transition_condition_unified;

    // Process confirm events
    for event in confirm_events.read() {
        // Find the layer that was confirmed
        let Some((_, layer)) = layer_query
            .iter()
            .find(|(_, l)| l.layer_id == event.layer_id && l.is_active)
        else {
            continue;
        };

        // Find matching transition rule
        let mut matched_action: Option<super::components::LayerTransitionAction> = None;

        for rule in &layer.on_confirm {
            let condition_met = if let Some(condition) = &rule.condition {
                evaluate_transition_condition_unified(condition, event.selected_index, &player_data)
            } else {
                true
            };

            if condition_met {
                matched_action = Some(rule.action.clone());
                break;
            }
        }

        if let Some(action) = matched_action {
            // Clone sound path before mutable borrow
            let sound_path = layer.sound_on_confirm.clone();
            let layer_id = event.layer_id.clone();

            // Play confirm sound if configured
            if let Some(sound) = &sound_path {
                audio::play_sound(&audio, &asset_server, sound);
            }

            execute_layer_transition(action, &layer_id, &mut layer_query, &mut next_ow_state);
        }
    }

    // Process cancel events
    for event in cancel_events.read() {
        // Find the layer that was cancelled
        let Some((_, layer)) = layer_query
            .iter()
            .find(|(_, l)| l.layer_id == event.layer_id && l.is_active)
        else {
            continue;
        };

        if let Some(action) = &layer.on_cancel {
            // Clone sound path and action before mutable borrow
            let sound_path = layer.sound_on_cancel.clone();
            let action = action.clone();
            let layer_id = event.layer_id.clone();

            // Play cancel sound if configured
            if let Some(sound) = &sound_path {
                audio::play_sound(&audio, &asset_server, sound);
            }

            execute_layer_transition(action, &layer_id, &mut layer_query, &mut next_ow_state);
        }
    }
}

/// Execute a layer transition action.
///
/// 执行层级转换动作。
fn execute_layer_transition(
    action: super::components::LayerTransitionAction,
    current_layer_id: &str,
    layer_query: &mut Query<(Entity, &mut super::components::InteractiveLayer)>,
    next_ow_state: &mut ResMut<NextState<OverworldState>>,
) {
    match action {
        super::components::LayerTransitionAction::GotoLayer(target_layer_id) => {
            info!(
                "Layer transition: '{}' -> '{}'",
                current_layer_id, target_layer_id
            );

            // Deactivate current layer, activate target layer
            for (_, mut layer) in layer_query.iter_mut() {
                if layer.layer_id == current_layer_id {
                    layer.is_active = false;
                } else if layer.layer_id == target_layer_id {
                    layer.is_active = true;
                    layer.current_selection = 0; // Reset selection
                }
            }
        }
        super::components::LayerTransitionAction::PopState => {
            info!("Layer transition: PopState from '{}'", current_layer_id);

            // Deactivate current layer and return to Normal state
            for (_, mut layer) in layer_query.iter_mut() {
                if layer.layer_id == current_layer_id {
                    layer.is_active = false;
                }
            }

            next_ow_state.set(OverworldState::Normal);
        }
        super::components::LayerTransitionAction::PushState(state_name) => {
            info!(
                "Layer transition: PushState({}) from '{}'",
                state_name, current_layer_id
            );
            // TODO: Implement state push if needed
        }
    }
}

/// Firewheel audio backend variant of the transition system.
#[cfg(feature = "firewheel")]
pub(crate) fn handle_interactive_layer_transitions_system(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut layer_query: Query<(Entity, &mut super::components::InteractiveLayer)>,
    mut confirm_events: bevy::ecs::message::MessageReader<
        super::components::SelectionConfirmedEvent,
    >,
    mut cancel_events: bevy::ecs::message::MessageReader<
        super::components::SelectionCancelledEvent,
    >,
    mut next_ow_state: ResMut<NextState<OverworldState>>,
    player_data: Res<crate::core::data::PlayerData>,
) {
    use super::ron_view::parsing::evaluate_transition_condition_unified;

    // Process confirm events
    for event in confirm_events.read() {
        let Some((_, layer)) = layer_query
            .iter()
            .find(|(_, l)| l.layer_id == event.layer_id && l.is_active)
        else {
            continue;
        };

        let mut matched_action: Option<super::components::LayerTransitionAction> = None;

        for rule in &layer.on_confirm {
            let condition_met = if let Some(condition) = &rule.condition {
                evaluate_transition_condition_unified(condition, event.selected_index, &player_data)
            } else {
                true
            };

            if condition_met {
                matched_action = Some(rule.action.clone());
                break;
            }
        }

        if let Some(action) = matched_action {
            let sound_path = layer.sound_on_confirm.clone();
            let layer_id = event.layer_id.clone();

            if let Some(sound) = &sound_path {
                audio::play_sound(&mut commands, &asset_server, sound);
            }

            execute_layer_transition(action, &layer_id, &mut layer_query, &mut next_ow_state);
        }
    }

    // Process cancel events
    for event in cancel_events.read() {
        let Some((_, layer)) = layer_query
            .iter()
            .find(|(_, l)| l.layer_id == event.layer_id && l.is_active)
        else {
            continue;
        };

        if let Some(action) = &layer.on_cancel {
            let sound_path = layer.sound_on_cancel.clone();
            let action = action.clone();
            let layer_id = event.layer_id.clone();

            if let Some(sound) = &sound_path {
                audio::play_sound(&mut commands, &asset_server, sound);
            }

            execute_layer_transition(action, &layer_id, &mut layer_query, &mut next_ow_state);
        }
    }
}
