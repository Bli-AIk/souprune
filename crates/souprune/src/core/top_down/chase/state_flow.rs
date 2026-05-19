//! # state_flow.rs
//!
//! # state_flow.rs 文件
//!
//! ## Module Overview
//!
//! ## 模块概述
//!
//! Manages the chase-mode state machine hooks in the top_down. It detects entry and
//! exit from the configured chase sub-state, loads the chase configuration from `flow.ron`, and
//! advances the transition timer that the visual layer consumes.
//!
//! 负责大地图追逐模式的状态流转钩子。它会检测何时进入或退出配置好的 chase 子状态，
//! 从 `flow.ron` 里加载追逐配置，并推进供视觉层消费的过渡计时器。

use super::*;

/// Run condition: check if chase is enabled.
pub(super) fn chase_enabled(enabled: Res<ChaseEnabled>) -> bool {
    enabled.0
}

/// Check if current state is a chase state.
pub fn is_in_chase_state(
    current_state: &SequenceSubState,
    chase_state_name: &ChaseStateName,
) -> bool {
    chase_state_name
        .0
        .as_ref()
        .is_some_and(|name| current_state.0 == *name)
}

/// System to detect when entering chase state.
pub(super) fn detect_chase_state_enter_system(
    mut commands: Commands,
    current_state: Res<State<SequenceSubState>>,
    chase_state_name: Res<ChaseStateName>,
    mut tracker: ResMut<ChaseStateTracker>,
    mut transition: ResMut<ChaseTransition>,
    chase_config: Option<Res<ChaseConfig>>,
    asset_server: Res<AssetServer>,
    sequence_mode: Res<crate::core::mode::SequenceMode>,
) {
    let in_chase = is_in_chase_state(&current_state, &chase_state_name);

    if in_chase && !tracker.was_in_chase {
        tracker.was_in_chase = true;
        let Some(scope) = crate::core::top_down::top_down_scoped(&sequence_mode) else {
            warn!("Chase: no active mode while creating effect root.");
            return;
        };

        commands.spawn((
            scope,
            ChaseEffectRoot,
            Transform::default(),
            Visibility::default(),
        ));
        info!("Chase: Created effect root entity");

        transition.active = true;
        transition.timer = 0.0;
        transition.transitioning_in = true;
        transition.cleanup_done = false;
        info!("Chase: Starting transition IN");

        if let Some(config) = chase_config {
            let layout_path = &config.damage_ui.layout_path;
            if !layout_path.is_empty() {
                let _handle: Handle<crate::core::view::layout::ViewLayoutAsset> =
                    asset_server.load(layout_path.clone());
                info!("Chase: Setup HUD from {}", layout_path);
            } else {
                warn!("Chase: No damage UI layout path configured");
            }
        }
    }
}

/// System to detect when exiting chase state.
pub(super) fn detect_chase_state_exit_system(
    current_state: Res<State<SequenceSubState>>,
    chase_state_name: Res<ChaseStateName>,
    mut tracker: ResMut<ChaseStateTracker>,
    mut transition: ResMut<ChaseTransition>,
) {
    let in_chase = is_in_chase_state(&current_state, &chase_state_name);

    if !in_chase && tracker.was_in_chase {
        tracker.was_in_chase = false;
        transition.transitioning_in = false;
        transition.timer = 0.0;
        info!("Chase: Starting transition OUT");
    }
}

/// Load chase configuration from the active state config.
pub(super) fn load_chase_config_system(
    mut commands: Commands,
    state_config: Res<LoadedStateConfig>,
    state_config_loaded: Res<crate::core::state_config::StateConfigLoaded>,
    mut chase_enabled: ResMut<ChaseEnabled>,
    mut chase_state_name: ResMut<ChaseStateName>,
    mut chase_loaded: ResMut<ChaseConfigLoaded>,
) {
    if !state_config_loaded.0 {
        return;
    }

    chase_loaded.0 = true;

    for (state_name, state_def) in state_config.iter() {
        if state_def.chase_config.is_some() {
            chase_state_name.0 = Some(state_name.clone());
            info!("Chase: Found chase state '{}' in flow.ron", state_name);

            if let Some(path) = &state_def.chase_config
                && let Some(config) = ChaseConfig::load_from_path(Some(path.as_str()))
            {
                info!("Chase: Enabled with config from {}", path);
                commands.insert_resource(config);
                chase_enabled.0 = true;
                return;
            }
        }
    }

    info!("Chase: Disabled - no chase state config found in flow.ron");
    commands.insert_resource(ChaseConfig::default());
    chase_enabled.0 = false;
}

/// Update chase transition timer.
pub(super) fn update_chase_transition_system(
    time: Res<Time>,
    mut transition: ResMut<ChaseTransition>,
    chase_config: Res<ChaseConfig>,
) {
    if !transition.active && !transition.transitioning_in && transition.timer <= 0.0 {
        return;
    }

    let duration = chase_config.transition_duration();
    if transition.transitioning_in {
        transition.timer = (transition.timer + time.delta_secs()).min(duration);
    } else {
        transition.timer = (transition.timer - time.delta_secs()).max(0.0);
        if transition.timer <= 0.0 {
            transition.active = false;
        }
    }
}
