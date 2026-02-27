//! # state.rs
//!
//! # state.rs 文件
//!
//! ## Module Overview
//!
//! ## 模块概述
//!
//! This module handles global triggers for UI state transitions.
//!
//! 本模块处理 UI 状态转换的全局触发器。

use super::ron_view::ViewGlobalTriggerConfig;
use crate::app_state::overworld::{OverworldSubState, character};
use crate::core::audio;
use crate::core::input::Action;
use bevy::prelude::*;
use leafwing_input_manager::action_state::ActionState;

// ============================================================================
// Global Trigger System
// 全局触发器系统
// ============================================================================

/// Handle global triggers that can activate from any overworld state.
///
/// 处理可以从任何 Overworld 状态激活的全局触发器。
pub(crate) fn global_trigger_system(
    audio: Res<bevy_kira_audio::Audio>,
    asset_server: Res<AssetServer>,
    mut next_state: ResMut<NextState<OverworldSubState>>,
    current_state: Res<State<OverworldSubState>>,
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
                if rule.allowed_states.iter().any(|s| s == current_state.get()) {
                    info!(
                        "Global trigger activated: {:?} -> {:?} via {:?}",
                        current_state.get(),
                        rule.target_state,
                        action
                    );

                    if let Some(sound_path) = &rule.sound {
                        audio::play_sound(&audio, &asset_server, sound_path);
                    }

                    next_state.set(rule.target_state.clone());
                    return;
                }
            }
        }
    }
}
