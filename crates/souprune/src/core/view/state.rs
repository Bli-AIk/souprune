//! # state.rs
//!
//! # state.rs 文件
//!
//! ## Module Overview
//!
//! ## 模块概述
//!
//! This module handles UI layer transitions and navigation between menu items.
//!
//! 本模块处理 UI 层级转换和菜单项之间的导航。
//!
//! ## Source File Overview
//!
//! ## 源文件概述
//!
//! It manages state changes triggered by user input such as confirm and cancel actions.
//!
//! 管理由用户输入触发的状态变化，例如确认和取消操作。

use super::components::{
    RonUI, TransitionAction, UILayer, UILayerNavigationConfig, UILayerTransitionConfig,
};
use super::ron_view::UIGlobalTriggerConfig;
use crate::app_state::overworld::{OverworldState, character};
use crate::core::audio;
use crate::core::input::Action;
use bevy::ecs::message::MessageWriter;
use bevy::prelude::*;
use leafwing_input_manager::action_state::ActionState;

/// Handle transitions between overworld sub-states driven by menu actions.
///
/// 处理菜单输入驱动的 Overworld 子状态间转换。
#[cfg(all(feature = "bevy_kira_audio", not(feature = "firewheel")))]
pub(crate) fn menu_overworld_state_transitions_system(
    audio: Res<bevy_kira_audio::Audio>,
    asset_server: Res<AssetServer>,
    mut next_state: ResMut<NextState<OverworldState>>,
    current_state: Res<State<OverworldState>>,
    mut overworld_ui_query: Query<&mut RonUI>,
    query: Query<&ActionState<Action>, With<character::components::PlayerControlled>>,
    player_data: Res<crate::core::data::PlayerData>,
    transition_config: Res<UILayerTransitionConfig>,
    navigation_config: Res<UILayerNavigationConfig>,
    global_trigger_config: Res<UIGlobalTriggerConfig>,
) {
    if let Ok(action_state) = query.single() {
        // Check global triggers first
        //
        // 首先检查全局触发器
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

        match current_state.get() {
            OverworldState::Normal => {
                // Logic moved to global triggers
            }
            OverworldState::Backpack => {
                let Ok(mut overworld_ui) = overworld_ui_query.single_mut() else {
                    warn!("Backpack menu open but no RonUI entity found");
                    return;
                };

                let current_layer = overworld_ui.layer().clone();

                if let Some(transitions) = transition_config.get(&current_layer) {
                    if action_state.just_pressed(&Action::Confirm) {
                        for rule in &transitions.on_confirm {
                            if let Some(condition) = &rule.condition
                                && !evaluate_transition_condition(
                                    condition,
                                    overworld_ui.index(),
                                    &player_data,
                                )
                            {
                                continue;
                            }

                            // Play confirm sound if configured
                            //
                            // 如果已配置，播放确认声音
                            if let Some(sound_path) = &transitions.sound_on_confirm {
                                audio::play_sound(&audio, &asset_server, sound_path);
                            }

                            execute_transition_action(
                                &rule.action,
                                &mut overworld_ui,
                                &mut next_state,
                                &player_data,
                                &navigation_config,
                            );
                            return;
                        }
                    }

                    if action_state.just_pressed(&Action::Cancel)
                        && let Some(cancel_action) = &transitions.on_cancel
                    {
                        // Play cancel sound if configured
                        //
                        // 如果已配置，播放取消声音
                        if let Some(sound_path) = &transitions.sound_on_cancel {
                            audio::play_sound(&audio, &asset_server, sound_path);
                        }

                        execute_transition_action(
                            cancel_action,
                            &mut overworld_ui,
                            &mut next_state,
                            &player_data,
                            &navigation_config,
                        );
                    }
                }
            }
            OverworldState::Cutscene => {
                info!("Menu key pressed during cutscene, ignoring");
            }
            OverworldState::Chase => {
                // Chase state - menu input is disabled during chase
                // 追逐战状态 - 追逐战期间禁用菜单输入
            }
        }
    }
}

fn evaluate_transition_condition(
    condition: &str,
    index: usize,
    player_data: &crate::core::data::PlayerData,
) -> bool {
    let condition = condition.trim();

    if condition.starts_with("index == ")
        && let Some(num_str) = condition.strip_prefix("index == ")
    {
        let parts: Vec<&str> = num_str.split("&&").map(|s| s.trim()).collect();
        let index_part = parts[0];
        if let Ok(target_index) = index_part.parse::<usize>() {
            if index != target_index {
                return false;
            }
            for part in parts.iter().skip(1) {
                if *part == "!player.inventory.is_empty" && player_data.inventory.is_empty() {
                    return false;
                }
            }
            return true;
        }
    }

    true
}

fn execute_transition_action(
    action: &TransitionAction,
    overworld_ui: &mut RonUI,
    next_state: &mut ResMut<NextState<OverworldState>>,
    player_data: &crate::core::data::PlayerData,
    navigation_config: &UILayerNavigationConfig,
) {
    match action {
        TransitionAction::GotoLayer(target_layer) => {
            let max_index = navigation_config
                .get(target_layer)
                .and_then(|rule| {
                    rule.max_index()
                        .as_ref()
                        .map(|bound| super::ron_view::evaluate_index_bound(bound, player_data))
                })
                .unwrap_or_else(|| calculate_max_index_for_layer(target_layer, player_data));
            info!(
                "Transitioning to layer {} with max_index {}",
                target_layer, max_index
            );
            overworld_ui.set_layer(target_layer.clone(), max_index);
        }
        TransitionAction::PopState => {
            info!("Popping state, returning to Normal");
            next_state.set(OverworldState::Normal);
        }
        TransitionAction::PushState(state_name) => {
            info!("TODO: Push state {}", state_name);
        }
    }
}

fn calculate_max_index_for_layer(
    layer: &UILayer,
    _player_data: &crate::core::data::PlayerData,
) -> usize {
    warn!(
        "Max index for layer {:?} not found in RON configuration! Defaulting to 1.",
        layer
    );
    1
}

/// Update UI focus navigation while the overworld backpack is active.
///
/// 在背包界面激活时更新 UI 焦点导航。
#[cfg(all(feature = "bevy_kira_audio", not(feature = "firewheel")))]
pub(crate) fn update_overworld_ui_navigation_system(
    audio: Res<bevy_kira_audio::Audio>,
    asset_server: Res<AssetServer>,
    overworld_state: Res<State<OverworldState>>,
    navigation: Res<UILayerNavigationConfig>,
    mut ui_query: Query<&mut RonUI>,
    query: Query<&ActionState<Action>, With<character::components::PlayerControlled>>,
    player_data: Res<crate::core::data::PlayerData>,
) {
    if overworld_state.get() != &OverworldState::Backpack {
        return;
    }

    let Ok(action_state) = query.single() else {
        return;
    };

    for mut overworld_ui in ui_query.iter_mut() {
        let Some(rule) = navigation.get(overworld_ui.layer()) else {
            continue;
        };

        let mut delta: isize = 0;
        for action in [Action::Up, Action::Down, Action::Left, Action::Right] {
            if action_state.just_pressed(&action)
                && let Some(change) = rule.delta_for(action)
            {
                delta += change;
            }
        }

        if delta != 0 {
            let min_index = rule
                .min_index()
                .as_ref()
                .map(|bound| super::ron_view::evaluate_index_bound(bound, &player_data))
                .unwrap_or(0);

            let max_index = rule
                .max_index()
                .as_ref()
                .map(|bound| super::ron_view::evaluate_index_bound(bound, &player_data))
                .unwrap_or_else(|| {
                    calculate_max_index_for_layer(overworld_ui.layer(), &player_data)
                });

            let mut next_index = overworld_ui.index() as isize + delta;

            if rule.looping() {
                // With looping enabled, wrap around
                //
                // 启用循环时,进行环绕
                if next_index < min_index as isize {
                    next_index = max_index as isize - 1;
                } else if next_index >= max_index as isize {
                    next_index = min_index as isize;
                }
            } else {
                // Without looping, clamp to bounds
                //
                // 未启用循环时,限制在边界内
                next_index = next_index.clamp(min_index as isize, (max_index - 1) as isize);
            }

            // Only update and play sound if index actually changed
            //
            // 仅当索引实际改变时更新并播放声音
            if overworld_ui.index() != next_index as usize {
                // Play navigation sound if configured
                //
                // 如果已配置,播放导航声音
                if let Some(sound_path) = rule.sound_on_navigate() {
                    audio::play_sound(&audio, &asset_server, sound_path);
                }

                overworld_ui.set_index(next_index as usize);
            }
        }
    }
}

// ============================================================================
// Experimental Seedling Audio Backend Systems
// ============================================================================

#[cfg(feature = "firewheel")]
pub(crate) fn menu_overworld_state_transitions_system(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut next_state: ResMut<NextState<OverworldState>>,
    current_state: Res<State<OverworldState>>,
    mut overworld_ui_query: Query<&mut RonUI>,
    query: Query<&ActionState<Action>, With<character::components::PlayerControlled>>,
    player_data: Res<crate::core::data::PlayerData>,
    transition_config: Res<UILayerTransitionConfig>,
    navigation_config: Res<UILayerNavigationConfig>,
    global_trigger_config: Res<UIGlobalTriggerConfig>,
) {
    if let Ok(action_state) = query.single() {
        // Check global triggers first
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

        match current_state.get() {
            OverworldState::Normal => {
                // Logic moved to global triggers
            }
            OverworldState::Backpack => {
                let Ok(mut overworld_ui) = overworld_ui_query.single_mut() else {
                    warn!("Backpack menu open but no RonUI entity found");
                    return;
                };

                let current_layer = overworld_ui.layer().clone();

                if let Some(transitions) = transition_config.get(&current_layer) {
                    if action_state.just_pressed(&Action::Confirm) {
                        for rule in &transitions.on_confirm {
                            if let Some(condition) = &rule.condition
                                && !evaluate_transition_condition(
                                    condition,
                                    overworld_ui.index(),
                                    &player_data,
                                )
                            {
                                continue;
                            }

                            if let Some(sound_path) = &transitions.sound_on_confirm {
                                audio::play_sound(&mut commands, &asset_server, sound_path);
                            }

                            execute_transition_action(
                                &rule.action,
                                &mut overworld_ui,
                                &mut next_state,
                                &player_data,
                                &navigation_config,
                            );
                            return;
                        }
                    }

                    if action_state.just_pressed(&Action::Cancel)
                        && let Some(cancel_action) = &transitions.on_cancel
                    {
                        if let Some(sound_path) = &transitions.sound_on_cancel {
                            audio::play_sound(&mut commands, &asset_server, sound_path);
                        }

                        execute_transition_action(
                            cancel_action,
                            &mut overworld_ui,
                            &mut next_state,
                            &player_data,
                            &navigation_config,
                        );
                    }
                }
            }
            OverworldState::Cutscene => {
                info!("Menu key pressed during cutscene, ignoring");
            }
            _ => {}
        }
    }
}

#[cfg(feature = "firewheel")]
pub(crate) fn update_overworld_ui_navigation_system(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    overworld_state: Res<State<OverworldState>>,
    navigation: Res<UILayerNavigationConfig>,
    mut ui_query: Query<&mut RonUI>,
    query: Query<&ActionState<Action>, With<character::components::PlayerControlled>>,
    player_data: Res<crate::core::data::PlayerData>,
) {
    if overworld_state.get() != &OverworldState::Backpack {
        return;
    }

    let Ok(action_state) = query.single() else {
        return;
    };

    for mut overworld_ui in ui_query.iter_mut() {
        let Some(rule) = navigation.get(overworld_ui.layer()) else {
            continue;
        };

        let mut delta: isize = 0;
        for action in [Action::Up, Action::Down, Action::Left, Action::Right] {
            if action_state.just_pressed(&action)
                && let Some(change) = rule.delta_for(action)
            {
                delta += change;
            }
        }

        if delta != 0 {
            let min_index = rule
                .min_index()
                .as_ref()
                .map(|bound| super::ron_view::evaluate_index_bound(bound, &player_data))
                .unwrap_or(0);

            let max_index = rule
                .max_index()
                .as_ref()
                .map(|bound| super::ron_view::evaluate_index_bound(bound, &player_data))
                .unwrap_or_else(|| {
                    calculate_max_index_for_layer(overworld_ui.layer(), &player_data)
                });

            let mut next_index = overworld_ui.index() as isize + delta;

            if rule.looping() {
                if next_index < min_index as isize {
                    next_index = max_index as isize - 1;
                } else if next_index >= max_index as isize {
                    next_index = min_index as isize;
                }
            } else {
                next_index = next_index.clamp(min_index as isize, (max_index - 1) as isize);
            }

            if overworld_ui.index() != next_index as usize {
                if let Some(sound_path) = rule.sound_on_navigate() {
                    audio::play_sound(&mut commands, &asset_server, sound_path);
                }

                overworld_ui.set_index(next_index as usize);
            }
        }
    }
}

// ============================================================================
// Unified Interactive Layer Systems (Phase 5)
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

// Firewheel audio backend variants
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

#[cfg(feature = "firewheel")]
pub(crate) fn handle_interactive_layer_confirm_cancel_system(
    _commands: Commands,
    _asset_server: Res<AssetServer>,
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
