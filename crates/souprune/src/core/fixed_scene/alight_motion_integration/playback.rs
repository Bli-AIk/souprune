//! # playback.rs
//!
//! # playback.rs 文件
//!
//! ## Module Overview
//!
//! ## 模块概述
//!
//! Acts as the playback entry for fixed-scene Alight Motion performances. It reacts to
//! play events, loads projects and optional scene config overrides, inserts the runtime entities
//! into fixed-scene scope, and detects when the imported animation has finished.
//!
//! fixed-scene Alight Motion 演出的播放入口。它会响应播放事件，加载工程与可选的fixed-scene
//! 覆盖配置，把运行时实体放进 fixed-scene scope，并检测导入动画何时真正播放完毕。

use super::config_loading::load_alight_motion_config_from_path;
use super::{AlightMotionEntity, AlightMotionSceneConfig, AlightMotionScenePatterns, markers};
use bevy::prelude::*;
use bevy_alight_motion::prelude::{AmPendingLayers, AmPlayback, load_am_project};

use crate::core::alight_motion_runtime::{
    AlightMotionPerformanceState, PlayAlightMotionPerformanceEvent,
};
use crate::core::fixed_scene::fixed_scene_scoped;
use crate::core::mode::SequenceMode;

/// System to handle PlayAlightMotionPerformanceEvent.
pub(super) fn handle_play_am_performance_event(
    mut commands: Commands,
    mut events: MessageReader<PlayAlightMotionPerformanceEvent>,
    mut am_state: ResMut<AlightMotionPerformanceState>,
    mut am_config: ResMut<AlightMotionSceneConfig>,
    asset_server: Res<AssetServer>,
    sequence_mode: Res<SequenceMode>,
) {
    for event in events.read() {
        let Some(mode_name) = sequence_mode.0.as_deref() else {
            warn!("FixedScene AM: no active mode while starting performance.");
            continue;
        };
        info!("[AM Scene] Starting performance: {}", event.amproj_path);

        if let Some(custom_config_path) = &event.scene_config_path {
            info!("[AM Scene] Using custom config: {}", custom_config_path);
            let (config, bullet_regex, boundary_regex, hidden_regex) =
                load_alight_motion_config_from_path(custom_config_path);
            *am_config = config;
            commands.insert_resource(AlightMotionScenePatterns {
                bullet_regex,
                boundary_regex,
                hidden_regex,
            });
        }

        let entity = load_am_project(&mut commands, &asset_server, &event.amproj_path);

        let base_scale = 0.25;
        let final_scale = base_scale * am_config.scale;
        let offset = Vec3::new(
            am_config.offset.0 * base_scale,
            am_config.offset.1 * base_scale,
            0.0,
        );

        commands.entity(entity).insert((
            fixed_scene_scoped(mode_name),
            Transform {
                translation: offset,
                scale: Vec3::splat(final_scale),
                ..Default::default()
            },
        ));

        commands
            .entity(entity)
            .queue(move |mut entity_world: EntityWorldMut| {
                if let Some(mut pending) = entity_world.get_mut::<AmPendingLayers>() {
                    let old_inv_fit_scale = pending.inv_fit_scale;
                    pending.inv_fit_scale = 1.0 / final_scale;
                    info!(
                        "[AM Scene] Updated inv_fit_scale: {} -> {} (final_scale={})",
                        old_inv_fit_scale, pending.inv_fit_scale, final_scale
                    );
                }
            });

        info!(
            "[AM Scene] Performance started, entity: {:?}, base_scale: {}, config_scale: {}, final_scale: {}, offset: {:?}",
            entity, base_scale, am_config.scale, final_scale, am_config.offset
        );

        commands.add_observer(markers::on_am_entity_spawned);

        am_state.is_playing = true;
        am_state.project_entity = Some(entity);
        am_state.final_scale = final_scale;
    }
}

/// System to check if Alight Motion performance has completed.
pub(super) fn check_am_performance_completion(
    playback: Option<Res<AmPlayback>>,
    mut am_state: ResMut<AlightMotionPerformanceState>,
) {
    if !am_state.is_playing {
        return;
    }

    if let Some(playback) = playback {
        let total_duration = playback.total_time_ms;
        am_state.total_duration_ms = total_duration;

        if playback.current_time_ms >= total_duration {
            info!(
                "[AM Scene] Performance completed ({}ms / {}ms)",
                playback.current_time_ms, total_duration
            );
            am_state.is_playing = false;
        }
    }
}

/// System to cleanup AM entities when exiting fixed-scene mode.
pub(super) fn cleanup_am_entities(
    mut commands: Commands,
    query: Query<Entity, With<AlightMotionEntity>>,
    mut am_state: ResMut<AlightMotionPerformanceState>,
) {
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }

    am_state.is_playing = false;
    am_state.project_entity = None;

    info!("[AM Scene] Cleaned up AM entities");
}
