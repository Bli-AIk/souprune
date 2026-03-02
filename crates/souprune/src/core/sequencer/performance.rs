//! # sequencer/performance.rs
//!
//! ## Module Overview
//!
//! Performance (Danmaku and AM) systems for the battle sequencer.
//!
//! 战斗序列管理器的演出（弹幕和 AM）系统。

use super::chapter_schema::Chapter;
use super::context::*;
use crate::app_state::battle::am_integration::{AmPerformanceState, PlayAmPerformanceEvent};
use crate::app_state::battle::danmaku::PlayPerformanceEvent;
use bevy::ecs::message::MessageWriter;
use bevy::prelude::*;

/// Marker component for AM performance chapter tracking.
///
/// AM 演出章节跟踪的标记组件。
#[derive(Component)]
pub struct AmPerformanceTracker {
    pub wait_for_completion: bool,
    /// Whether we've seen the performance start (is_playing became true)
    pub started: bool,
}

/// System to process DanmakuPerformance chapters.
///
/// 处理弹幕演出章节的系统。
#[allow(clippy::type_complexity)]
pub fn process_danmaku_performance_system(
    mut commands: Commands,
    query: Query<(Entity, &ActiveChapter), (Without<WaitTimer>, Without<ChapterFinished>)>,
    mut performance_events: MessageWriter<PlayPerformanceEvent>,
) {
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
            let mut event = PlayPerformanceEvent::new(performance.clone());
            if let Some((x, y)) = translation {
                event = event.at_position(Vec2::new(*x, *y));
            }
            performance_events.write(event);
            commands.entity(entity).insert(ChapterFinished);
        }
    }
}

/// System to process AmPerformance chapters.
///
/// 处理 AM 演出章节的系统。
#[allow(clippy::type_complexity)]
pub fn process_am_performance_system(
    mut commands: Commands,
    query: Query<
        (Entity, &ActiveChapter),
        (
            Without<WaitTimer>,
            Without<ChapterFinished>,
            Without<AmPerformanceTracker>,
        ),
    >,
    mut performance_events: MessageWriter<PlayAmPerformanceEvent>,
) {
    for (entity, active_chapter) in query.iter() {
        if let Chapter::AmPerformance {
            amproj_path,
            am_config,
            wait_for_completion,
        } = &active_chapter.chapter
        {
            info!("[Battle] Starting AM performance from: {}", amproj_path);

            // Send event to start the AM performance
            performance_events.write(PlayAmPerformanceEvent::with_config(
                amproj_path.clone(),
                am_config.clone(),
            ));

            if *wait_for_completion {
                // Add tracker component to wait for completion
                commands.entity(entity).insert(AmPerformanceTracker {
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

/// System to check if AM performance has completed and finish the chapter.
///
/// 检查 AM 演出是否完成并结束章节的系统。
pub fn process_am_wait_chapter_system(
    mut commands: Commands,
    mut query: Query<(Entity, &mut AmPerformanceTracker), Without<ChapterFinished>>,
    am_state: Res<AmPerformanceState>,
) {
    for (entity, mut tracker) in query.iter_mut() {
        if !tracker.wait_for_completion {
            continue;
        }

        // Wait for performance to start first
        if am_state.is_playing {
            tracker.started = true;
        }

        // Only mark finished after performance has started and then stopped
        if tracker.started && !am_state.is_playing {
            info!("[Battle] AM performance chapter finished");
            commands.entity(entity).insert(ChapterFinished);
        }
    }
}
