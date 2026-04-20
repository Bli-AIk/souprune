//! # sequencer/performance.rs
//!
//! ## Module Overview
//!
//! Performance (Danmaku and Alight Motion) systems for the battle sequencer.
//!
//! 战斗序列管理器的演出（弹幕和 Alight Motion）系统。

use super::chapter_schema::Chapter;
use super::context::*;
use crate::core::alight_motion_runtime::{
    AlightMotionPerformanceState, PlayAlightMotionPerformanceEvent,
};
use crate::core::danmaku::DanmakuPerformance;
use bevy::prelude::*;

/// Tracker component for danmaku performance chapter awaiting completion.
/// Inserted when `wait_for_completion: true`. Holds the asset handle so
/// the wait system can read `duration` once the asset finishes loading.
///
/// 弹幕演出章节等待完成的跟踪组件。
/// 当 `wait_for_completion: true` 时插入，持有资产句柄以便等待系统
/// 在资产加载完成后读取 `duration`。
#[derive(Component)]
pub struct DanmakuPerformanceTracker {
    pub handle: Handle<DanmakuPerformance>,
}

/// Marker component for Alight Motion performance chapter tracking.
///
/// Alight Motion 演出章节跟踪的标记组件。
#[derive(Component)]
pub struct AlightMotionPerformanceTracker {
    pub wait_for_completion: bool,
    /// Whether we've seen the performance start (is_playing became true)
    pub started: bool,
}

/// System to process DanmakuPerformance chapters.
///
/// 处理弹幕演出章节的系统。
pub fn process_danmaku_performance_system(
    mut commands: Commands,
    query: Query<
        (Entity, &ActiveChapter),
        (
            Without<WaitTimer>,
            Without<ChapterFinished>,
            Without<DanmakuPerformanceTracker>,
        ),
    >,
    performance_events: Option<
        bevy::ecs::message::MessageWriter<crate::core::danmaku::PlayPerformanceEvent>,
    >,
    asset_server: Res<AssetServer>,
) {
    let Some(mut performance_events) = performance_events else {
        return;
    };
    for (entity, active_chapter) in query.iter() {
        if let Chapter::DanmakuPerformance {
            performance,
            translation,
            wait_for_completion,
        } = &active_chapter.chapter
        {
            info!(
                "[Battle] Starting danmaku performance from: {}",
                performance
            );
            let mut event = crate::core::danmaku::PlayPerformanceEvent::new(performance.clone());
            if let Some((x, y)) = translation {
                event = event.at_position(Vec2::new(*x, *y));
            }
            performance_events.write(event);

            if *wait_for_completion {
                let handle = asset_server.load::<DanmakuPerformance>(performance.clone());
                commands
                    .entity(entity)
                    .insert(DanmakuPerformanceTracker { handle });
            } else {
                commands.entity(entity).insert(ChapterFinished);
            }
        }
    }
}

/// System to resolve the danmaku performance duration and set up the wait timer.
/// Once the asset is loaded, reads `duration` and either inserts a `WaitTimer`
/// or marks `ChapterFinished` immediately if no duration is specified.
///
/// 解析弹幕演出时长并设置等待计时器的系统。
/// 资产加载完成后，读取 `duration` 字段：有值则插入 `WaitTimer`，
/// 无值则直接标记 `ChapterFinished`。
pub fn process_danmaku_wait_chapter_system(
    mut commands: Commands,
    query: Query<(Entity, &DanmakuPerformanceTracker), Without<ChapterFinished>>,
    performances: Res<Assets<DanmakuPerformance>>,
) {
    for (entity, tracker) in query.iter() {
        let Some(perf) = performances.get(&tracker.handle) else {
            continue; // Asset not loaded yet — retry next frame.
        };
        match perf.resolved_duration() {
            Some(d) if d > 0.0 => {
                commands
                    .entity(entity)
                    .insert(WaitTimer(Timer::from_seconds(d, TimerMode::Once)))
                    .remove::<DanmakuPerformanceTracker>();
                info!("[Battle] Danmaku performance blocking for {:.1}s", d);
            }
            _ => {
                commands
                    .entity(entity)
                    .insert(ChapterFinished)
                    .remove::<DanmakuPerformanceTracker>();
                debug!("[Battle] Danmaku performance has no duration — completing immediately");
            }
        }
    }
}

/// System to process AlightMotionPerformance chapters.
///
/// 处理 Alight Motion 演出章节的系统。
pub fn process_am_performance_system(
    mut commands: Commands,
    query: Query<
        (Entity, &ActiveChapter),
        (
            Without<WaitTimer>,
            Without<ChapterFinished>,
            Without<AlightMotionPerformanceTracker>,
        ),
    >,
    performance_events: Option<bevy::ecs::message::MessageWriter<PlayAlightMotionPerformanceEvent>>,
) {
    let Some(mut performance_events) = performance_events else {
        return;
    };
    for (entity, active_chapter) in query.iter() {
        if let Chapter::AlightMotionPerformance {
            amproj_path,
            alight_motion_config,
            wait_for_completion,
        } = &active_chapter.chapter
        {
            info!(
                "[Battle] Starting Alight Motion performance from: {}",
                amproj_path
            );

            // Send event to start the Alight Motion performance
            performance_events.write(PlayAlightMotionPerformanceEvent::with_config(
                amproj_path.clone(),
                alight_motion_config.clone(),
            ));

            if *wait_for_completion {
                // Add tracker component to wait for completion
                commands
                    .entity(entity)
                    .insert(AlightMotionPerformanceTracker {
                        wait_for_completion: true,
                        started: false,
                    });
            } else {
                // Not waiting, mark as finished immediately
                commands.entity(entity).insert(ChapterFinished);
            }
        }
    }
}

/// System to check if Alight Motion performance has completed and finish the chapter.
///
/// 检查 Alight Motion 演出是否完成并结束章节的系统。
pub fn process_alight_motion_wait_chapter_system(
    mut commands: Commands,
    mut query: Query<(Entity, &mut AlightMotionPerformanceTracker), Without<ChapterFinished>>,
    alight_motion_state: Option<Res<AlightMotionPerformanceState>>,
) {
    let Some(alight_motion_state) = alight_motion_state else {
        return;
    };
    for (entity, mut tracker) in query.iter_mut() {
        if !tracker.wait_for_completion {
            continue;
        }

        // Wait for performance to start first
        if alight_motion_state.is_playing {
            tracker.started = true;
        }

        // Only mark finished after performance has started and then stopped
        if tracker.started && !alight_motion_state.is_playing {
            info!("[Battle] Alight Motion performance chapter finished");
            commands.entity(entity).insert(ChapterFinished);
        }
    }
}
