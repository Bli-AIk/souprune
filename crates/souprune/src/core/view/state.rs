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

use super::components::interactive::NavDirection;
use super::ron_view::ViewGlobalTriggerConfig;
use crate::app_state::overworld::{OverworldState, character};
use crate::core::audio;
use crate::core::input::{Action, ActionRegistry, ActionStateExt, InputBehaviorConfig};
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
#[cfg(all(feature = "bevy_kira_audio", not(feature = "firewheel")))]
pub(crate) fn global_trigger_system(
    audio: Res<bevy_kira_audio::Audio>,
    asset_server: Res<AssetServer>,
    mut next_state: ResMut<NextState<OverworldState>>,
    current_state: Res<State<OverworldState>>,
    query: Query<&ActionState<Action>, With<character::components::PlayerControlled>>,
    global_trigger_config: Res<ViewGlobalTriggerConfig>,
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
    global_trigger_config: Res<ViewGlobalTriggerConfig>,
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
#[cfg(all(feature = "bevy_kira_audio", not(feature = "firewheel")))]
pub(crate) fn handle_interactive_layer_navigation_system(
    audio: Res<bevy_kira_audio::Audio>,
    asset_server: Res<AssetServer>,
    registry: Res<ActionRegistry>,
    behavior_config: Res<InputBehaviorConfig>,
    mut layer_query: Query<(Entity, &mut super::components::InteractiveLayer)>,
    input_query: Query<&ActionState<Action>>,
    mut selection_changed_events: MessageWriter<super::components::SelectionChangedEvent>,
) {
    let Some(action_state) = input_query.iter().next() else {
        return;
    };

    for (entity, mut layer) in layer_query.iter_mut() {
        if !layer.is_active {
            continue;
        }

        let previous_index = layer.current_selection;
        let mut changed = false;

        // Check all navigation directions using behavior config
        // 使用行为配置检查所有导航方向
        for direction in NavDirection::all() {
            if let Some(action) = direction.to_action(&registry, &behavior_config) {
                if action_state.just_pressed(&action) && layer.navigate_direction(direction) {
                    changed = true;
                    break;
                }
            }
        }

        if changed {
            if let Some(sound_path) = &layer.sound_on_navigate {
                audio::play_sound(&audio, &asset_server, sound_path);
            }

            selection_changed_events.write(super::components::SelectionChangedEvent {
                layer_id: layer.layer_id.clone(),
                previous_index,
                new_index: layer.current_selection,
                entity,
            });

            debug!(
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
    registry: Res<ActionRegistry>,
    behavior_config: Res<InputBehaviorConfig>,
    mut layer_query: Query<(Entity, &mut super::components::InteractiveLayer)>,
    input_query: Query<&ActionState<Action>>,
    mut selection_changed_events: MessageWriter<super::components::SelectionChangedEvent>,
) {
    let Some(action_state) = input_query.iter().next() else {
        return;
    };

    for (entity, mut layer) in layer_query.iter_mut() {
        if !layer.is_active {
            continue;
        }

        let previous_index = layer.current_selection;
        let mut changed = false;

        // Check all navigation directions using behavior config
        // 使用行为配置检查所有导航方向
        for direction in NavDirection::all() {
            if let Some(action) = direction.to_action(&registry, &behavior_config)
                && action_state.just_pressed(&action)
                && layer.navigate_direction(direction)
            {
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

            debug!(
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
#[cfg(all(feature = "bevy_kira_audio", not(feature = "firewheel")))]
pub(crate) fn handle_interactive_layer_confirm_cancel_system(
    audio: Res<bevy_kira_audio::Audio>,
    asset_server: Res<AssetServer>,
    registry: Res<ActionRegistry>,
    behavior_config: Res<InputBehaviorConfig>,
    layer_query: Query<(Entity, &super::components::InteractiveLayer)>,
    input_query: Query<&ActionState<Action>>,
    mut confirm_events: MessageWriter<super::components::SelectionConfirmedEvent>,
    mut cancel_events: MessageWriter<super::components::SelectionCancelledEvent>,
) {
    let Some(action_state) = input_query.iter().next() else {
        return;
    };

    for (entity, layer) in layer_query.iter() {
        if !layer.is_active {
            continue;
        }

        // Use behavior config to get action names for UI interactions
        // 使用行为配置获取 UI 交互的动作名称
        let confirm_pressed = behavior_config
            .ui_confirm()
            .map(|name| action_state.action_just_pressed(&registry, name))
            .unwrap_or(false);
        let cancel_pressed = behavior_config
            .ui_cancel()
            .map(|name| action_state.action_just_pressed(&registry, name))
            .unwrap_or(false);

        if confirm_pressed {
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
    registry: Res<ActionRegistry>,
    behavior_config: Res<InputBehaviorConfig>,
    layer_query: Query<(Entity, &super::components::InteractiveLayer)>,
    input_query: Query<&ActionState<Action>>,
    mut confirm_events: MessageWriter<super::components::SelectionConfirmedEvent>,
    mut cancel_events: MessageWriter<super::components::SelectionCancelledEvent>,
) {
    let Some(action_state) = input_query.iter().next() else {
        return;
    };

    for (entity, layer) in layer_query.iter() {
        if !layer.is_active {
            continue;
        }

        // Use behavior config to get action names for UI interactions
        // 使用行为配置获取 UI 交互的动作名称
        let confirm_pressed = behavior_config
            .ui_confirm()
            .map(|name| action_state.action_just_pressed(&registry, name))
            .unwrap_or(false);
        let cancel_pressed = behavior_config
            .ui_cancel()
            .map(|name| action_state.action_just_pressed(&registry, name))
            .unwrap_or(false);

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
    mut activated_events: MessageWriter<super::components::LayerActivatedEvent>,
    mut deactivated_events: MessageWriter<super::components::LayerDeactivatedEvent>,
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
                audio::play_sound(&audio, &asset_server, sound);
            }

            execute_layer_transition(
                action,
                &layer_id,
                &mut layer_query,
                &mut next_ow_state,
                &mut activated_events,
                &mut deactivated_events,
            );
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
                audio::play_sound(&audio, &asset_server, sound);
            }

            execute_layer_transition(
                action,
                &layer_id,
                &mut layer_query,
                &mut next_ow_state,
                &mut activated_events,
                &mut deactivated_events,
            );
        }
    }
}

/// Execute a layer transition action.
fn execute_layer_transition(
    action: super::components::LayerTransitionAction,
    current_layer_id: &str,
    layer_query: &mut Query<(Entity, &mut super::components::InteractiveLayer)>,
    next_ow_state: &mut ResMut<NextState<OverworldState>>,
    activated_events: &mut MessageWriter<super::components::LayerActivatedEvent>,
    deactivated_events: &mut MessageWriter<super::components::LayerDeactivatedEvent>,
) {
    match action {
        super::components::LayerTransitionAction::GotoLayer(target_layer_id) => {
            info!(
                "Layer transition: '{}' -> '{}'",
                current_layer_id, target_layer_id
            );

            let mut deactivate_entity = None;
            let mut activate_entity = None;
            let mut activate_selection = 0;

            for (entity, layer) in layer_query.iter() {
                if layer.layer_id == current_layer_id {
                    deactivate_entity = Some((entity, layer.layer_id.clone()));
                } else if layer.layer_id == target_layer_id {
                    activate_entity = Some((entity, layer.layer_id.clone()));
                }
            }

            for (_, mut layer) in layer_query.iter_mut() {
                if layer.layer_id == current_layer_id {
                    layer.is_active = false;
                } else if layer.layer_id == target_layer_id {
                    layer.is_active = true;
                    layer.current_selection = 0;
                    activate_selection = 0;
                }
            }

            if let Some((entity, layer_id)) = deactivate_entity {
                deactivated_events
                    .write(super::components::LayerDeactivatedEvent { layer_id, entity });
            }

            if let Some((entity, layer_id)) = activate_entity {
                activated_events.write(super::components::LayerActivatedEvent {
                    layer_id,
                    current_selection: activate_selection,
                    entity,
                });
            }
        }
        super::components::LayerTransitionAction::PopState => {
            info!("Layer transition: PopState from '{}'", current_layer_id);

            let mut deactivate_entity = None;

            for (entity, layer) in layer_query.iter() {
                if layer.layer_id == current_layer_id {
                    deactivate_entity = Some((entity, layer.layer_id.clone()));
                }
            }

            for (_, mut layer) in layer_query.iter_mut() {
                if layer.layer_id == current_layer_id {
                    layer.is_active = false;
                }
            }

            if let Some((entity, layer_id)) = deactivate_entity {
                deactivated_events
                    .write(super::components::LayerDeactivatedEvent { layer_id, entity });
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
    mut activated_events: MessageWriter<super::components::LayerActivatedEvent>,
    mut deactivated_events: MessageWriter<super::components::LayerDeactivatedEvent>,
    mut next_ow_state: ResMut<NextState<OverworldState>>,
    player_data: Res<crate::core::data::PlayerData>,
) {
    use super::ron_view::parsing::evaluate_transition_condition_unified;

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

            execute_layer_transition(
                action,
                &layer_id,
                &mut layer_query,
                &mut next_ow_state,
                &mut activated_events,
                &mut deactivated_events,
            );
        }
    }

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

            execute_layer_transition(
                action,
                &layer_id,
                &mut layer_query,
                &mut next_ow_state,
                &mut activated_events,
                &mut deactivated_events,
            );
        }
    }
}
