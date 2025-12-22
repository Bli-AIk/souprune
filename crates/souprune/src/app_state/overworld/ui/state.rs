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
    OverworldUI, TransitionAction, UILayer, UILayerNavigationConfig, UILayerTransitionConfig,
};
use super::ron_ui_system::UIGlobalTriggerConfig;
use crate::app_state::overworld::{OverworldState, character};
use crate::core::audio;
use crate::core::input::Action;
use bevy::prelude::*;
use bevy_kira_audio::Audio;
use leafwing_input_manager::action_state::ActionState;

/// Handle transitions between overworld sub-states driven by menu actions.
///
/// 处理菜单输入驱动的 Overworld 子状态间转换。
pub(crate) fn menu_overworld_state_transitions_system(
    audio: Res<Audio>,
    asset_server: Res<AssetServer>,
    mut next_state: ResMut<NextState<OverworldState>>,
    current_state: Res<State<OverworldState>>,
    mut overworld_ui_query: Query<&mut OverworldUI>,
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
                for rule in rules {
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
                    warn!("Backpack menu open but no OverworldUI entity found");
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
    overworld_ui: &mut OverworldUI,
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
                        .map(|bound| super::ron_ui_system::evaluate_index_bound(bound, player_data))
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
pub(crate) fn update_overworld_ui_navigation_system(
    audio: Res<Audio>,
    asset_server: Res<AssetServer>,
    overworld_state: Res<State<OverworldState>>,
    navigation: Res<UILayerNavigationConfig>,
    mut ui_query: Query<&mut OverworldUI>,
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
                .map(|bound| super::ron_ui_system::evaluate_index_bound(bound, &player_data))
                .unwrap_or(0);

            let max_index = rule
                .max_index()
                .as_ref()
                .map(|bound| super::ron_ui_system::evaluate_index_bound(bound, &player_data))
                .unwrap_or_else(|| {
                    calculate_max_index_for_layer(overworld_ui.layer(), &player_data)
                });

            let mut next_index = overworld_ui.index() as isize + delta;

            if rule.looping() {
                // With looping enabled, wrap around
                //
                // 启用循环时，进行环绕
                if next_index < min_index as isize {
                    next_index = max_index as isize - 1;
                } else if next_index >= max_index as isize {
                    next_index = min_index as isize;
                }
            } else {
                // Without looping, clamp to bounds
                //
                // 未启用循环时，限制在边界内
                next_index = next_index.clamp(min_index as isize, (max_index - 1) as isize);
            }

            // Only update and play sound if index actually changed
            //
            // 仅当索引实际改变时更新并播放声音
            if overworld_ui.index() != next_index as usize {
                // Play navigation sound if configured
                //
                // 如果已配置，播放导航声音
                if let Some(sound_path) = rule.sound_on_navigate() {
                    audio::play_sound(&audio, &asset_server, sound_path);
                }

                overworld_ui.set_index(next_index as usize);
            }
        }
    }
}
