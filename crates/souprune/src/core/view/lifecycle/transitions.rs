//! # transitions.rs
//!
//! # transitions.rs 文件
//!
//! ## Module Overview
//!
//! ## 模块概述
//!
//! Reacts to high-level state transitions that affect the View system. It decides when
//! an interactive state should spawn or despawn a layout, and it also plays the configured enter
//! and exit sounds so state changes stay consistent across View and audio.
//!
//! 处理会影响 View 系统的高层状态切换。它决定交互状态进入或退出时何时生成、
//! 销毁布局，同时负责播放状态配置中的进入/退出音效，让 View 与音频的切换保持一致。

use crate::core::audio;
use crate::core::mode::{SequenceMode, SequenceSubState};
use crate::extra::mortar::LocaleLoaded;
use bevy::prelude::*;

#[derive(Resource, Default)]
pub struct StateTransitionTracker {
    pub previous_state: Option<String>,
}

#[derive(Resource, Default)]
pub struct UIInteractiveStateTracker {
    pub was_view_interactive: bool,
    pub current_view_path: Option<String>,
}

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub(crate) struct StateViewTransitionSet;

fn active_mode_scope(sequence_mode: &SequenceMode) -> Option<String> {
    sequence_mode.0.clone()
}

pub(crate) fn backpack_state_transition_system(
    sequence_mode: Res<SequenceMode>,
    sub_state: Res<State<SequenceSubState>>,
    state_config: Option<Res<crate::core::state_config::LoadedStateConfig>>,
    mut tracker: ResMut<UIInteractiveStateTracker>,
    locale_loaded: Option<Res<LocaleLoaded>>,
    mut spawn_writer: MessageWriter<super::super::SpawnViewRequest>,
    mut despawn_writer: MessageWriter<super::super::DespawnViewRequest>,
) {
    let Some(state_config) = state_config else {
        return;
    };

    let state_name = sub_state.name();
    let is_view_interactive = state_config.is_view_interactive(state_name);

    trace!(
        "[lifecycle] state='{}', is_view_interactive={}, was_view_interactive={}",
        state_name, is_view_interactive, tracker.was_view_interactive,
    );

    if is_view_interactive && !tracker.was_view_interactive {
        if locale_loaded.is_none() {
            return;
        }

        if let Some(view_layout_path) = state_config
            .get_view_layout(state_name)
            .map(|s| s.to_string())
        {
            info!(
                "[lifecycle] Entering View interactive state '{}' - emitting SpawnViewRequest: '{}'",
                state_name, view_layout_path
            );

            spawn_writer.write(super::super::SpawnViewRequest {
                path: view_layout_path.clone(),
                mode_scope: active_mode_scope(&sequence_mode),
                pre_spawn_events: state_config.get_pre_spawn_events(state_name).to_vec(),
                bindings: None,
            });

            tracker.current_view_path = Some(view_layout_path);
        } else {
            warn!(
                "[lifecycle] View interactive state '{}' has no view_layout configured",
                state_name
            );
        }
    }

    if !is_view_interactive && tracker.was_view_interactive {
        info!("[lifecycle] Exiting View interactive state - emitting DespawnViewRequest");

        despawn_writer.write(super::super::DespawnViewRequest {
            path: tracker.current_view_path.take(),
        });
    }

    tracker.was_view_interactive = is_view_interactive;
}

pub(crate) fn state_transition_sound_system(
    audio: Res<bevy_kira_audio::Audio>,
    asset_server: Res<AssetServer>,
    mut audio_cache: ResMut<crate::core::audio::AudioSourceCache>,
    sub_state: Res<State<SequenceSubState>>,
    state_config: Option<Res<crate::core::state_config::LoadedStateConfig>>,
    mut tracker: ResMut<StateTransitionTracker>,
) {
    let Some(state_config) = state_config else {
        return;
    };

    let current_state = sub_state.name();

    if tracker.previous_state.as_deref() != Some(current_state) {
        if let Some(ref prev_state) = tracker.previous_state
            && let Some(state_def) = state_config.get(prev_state)
            && let Some(ref sound_path) = state_def.on_exit_sound
        {
            audio::play_sound_full_path(&audio, &asset_server, &mut audio_cache, sound_path);
            debug!(
                "Playing on_exit_sound for state '{}': {}",
                prev_state, sound_path
            );
        }

        if let Some(state_def) = state_config.get(current_state)
            && let Some(ref sound_path) = state_def.on_enter_sound
        {
            audio::play_sound_full_path(&audio, &asset_server, &mut audio_cache, sound_path);
            debug!(
                "Playing on_enter_sound for state '{}': {}",
                current_state, sound_path
            );
        }

        tracker.previous_state = Some(current_state.to_string());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn state_view_scope_uses_active_project_mode_name() {
        let sequence_mode = SequenceMode(Some("field".to_string()));

        assert_eq!(active_mode_scope(&sequence_mode), Some("field".to_string()));
    }
}
