//! # performance_loading.rs
//!
//! # performance_loading.rs 文件
//!
//! ## Module Overview
//!
//! ## 模块概述
//!
//! Handles the loading edge of danmaku performances. It receives play requests, queues
//! the referenced performance assets, and once those assets are ready it spawns the runtime player
//! entity together with the bullet container that will own the emitted bullets.
//!
//! 负责弹幕演出的加载入口。它会接收播放请求并排队加载对应的演出资产，等资产准备好
//! 后，再生成运行时播放器实体以及承载弹幕的容器实体。

use super::*;

/// System to process play performance events and queue asset loads.
///
/// 处理播放演出事件并排队加载资产。
pub fn process_play_performance_events(
    mut events: MessageReader<PlayPerformanceEvent>,
    mut pending: ResMut<PendingPerformanceLoads>,
    asset_server: Res<AssetServer>,
) {
    for event in events.read() {
        let handle = asset_server.load::<DanmakuPerformance>(&event.performance_path);
        pending.pending.push((handle, event.clone()));
        info!("Queued performance load: {}", event.performance_path);
    }
}

/// System to spawn performance players when assets are loaded.
///
/// 当资产加载完成时生成演出播放器。
pub fn spawn_performance_players(
    mut commands: Commands,
    mut pending: ResMut<PendingPerformanceLoads>,
    performances: Res<Assets<DanmakuPerformance>>,
    spawn_context: Res<DanmakuSpawnContext>,
) {
    let mut still_pending = Vec::new();

    for (handle, event) in pending.pending.drain(..) {
        if performances.get(&handle).is_some() {
            let mut container_commands = commands.spawn((
                BulletContainer {
                    center: event.position,
                },
                Transform::from_translation(event.position.extend(0.0)),
                GlobalTransform::default(),
                Visibility::default(),
                InheritedVisibility::default(),
                ViewVisibility::default(),
                Name::new("BulletContainer"),
            ));

            if let Some(ref mode) = spawn_context.mode {
                container_commands.insert(ModeScoped(mode.clone()));
            }

            let container_entity = container_commands.id();

            let mut player = PerformancePlayer::new(event.position);
            player.container_entity = Some(container_entity);

            let mut player_commands = commands.spawn((
                player,
                PerformanceHandle(handle.clone()),
                PerformancePlayerMarker,
                Name::new("PerformancePlayer"),
            ));

            if let Some(ref mode) = spawn_context.mode {
                player_commands.insert(ModeScoped(mode.clone()));
            }

            info!(
                "Started performance: {} with container {:?}",
                event.performance_path, container_entity
            );
        } else {
            still_pending.push((handle, event));
        }
    }

    pending.pending = still_pending;
}
