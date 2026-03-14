//! # sequencer/performance.rs
//!
//! ## Module Overview
//!
//! Performance (Danmaku and Alight Motion) systems for the battle sequencer.
//!
//! 战斗序列管理器的演出（弹幕和 Alight Motion）系统。

use super::chapter_schema::Chapter;
use super::context::*;
use crate::app_state::battle::alight_motion_integration::{
    AlightMotionPerformanceState, PlayAlightMotionPerformanceEvent,
};
use bevy::prelude::*;

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
    query: Query<(Entity, &ActiveChapter), (Without<WaitTimer>, Without<ChapterFinished>)>,
    performance_events: Option<
        bevy::ecs::message::MessageWriter<crate::core::danmaku::PlayPerformanceEvent>,
    >,
) {
    let Some(mut performance_events) = performance_events else {
        return;
    };
    for (entity, active_chapter) in query.iter() {
        if let Chapter::DanmakuPerformance {
            performance,
            translation,
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
            commands.entity(entity).insert(ChapterFinished);
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
    performance_events: Option<
        bevy::ecs::message::MessageWriter<
            crate::app_state::battle::alight_motion_integration::PlayAlightMotionPerformanceEvent,
        >,
    >,
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
