//! Manages chapter entities as they are spawned, waited on, completed, and cleaned up.
//!
//! 管理章节实体从生成、等待、完成到清理的整个生命周期。
//!
//! This file contains the generic lifecycle machinery behind the sequencer:
//! spawning active chapters, advancing the root queue, tracking parallel
//! branches, and retiring finished chapter entities. It is the control plane of
//! sequence execution rather than the place where individual chapter semantics live.
//!
//! 这个文件放的是 sequencer 背后的通用生命周期机制：生成活动章节、推进根队列、
//! 跟踪并行分支，以及回收已经完成的章节实体。它承担的是序列执行的控制面，
//! 而不是某个具体章节语义本身。

use bevy::prelude::*;

use super::super::chapter_schema::Chapter;
use super::super::context::{
    ActiveChapter, ChapterFinished, ParallelTracker, SequenceContext, WaitTimer,
};

/// Helper to spawn chapters.
///
/// 生成章节的辅助函数。
pub fn spawn_chapter(commands: &mut Commands, chapter: Chapter, parent: Option<Entity>) {
    let entity = commands
        .spawn(ActiveChapter {
            chapter: chapter.clone(),
            parent,
        })
        .id();

    match chapter {
        Chapter::Wait(secs) => {
            commands
                .entity(entity)
                .insert(WaitTimer(Timer::from_seconds(secs, TimerMode::Once)));
        }
        Chapter::Parallel(children) => {
            commands.entity(entity).insert(ParallelTracker {
                pending_count: children.len(),
            });
            for child in children {
                spawn_chapter(commands, child, Some(entity));
            }
        }
        Chapter::Sequence(children) => {
            if parent.is_some() {
                warn!("Nested Sequence not fully implemented yet, treating as Parallel for now");
                commands.entity(entity).insert(ParallelTracker {
                    pending_count: children.len(),
                });
                for child in children {
                    spawn_chapter(commands, child, Some(entity));
                }
            }
        }
        _ => {}
    }
}

/// System to advance the battle flow.
///
/// 推进战斗流程系统。
pub fn advance_battle_flow_system(
    mut commands: Commands,
    mut context: ResMut<SequenceContext>,
    active_chapters: Query<&ActiveChapter, Without<ChapterFinished>>,
) {
    for chapter in active_chapters.iter() {
        if chapter.parent.is_none() {
            return;
        }
    }

    if context.chapters.is_empty() {
        return;
    }

    let next_chapter = context.chapters.remove(0);

    match next_chapter {
        Chapter::Sequence(sub_chapters) => {
            let mut new_queue = sub_chapters;
            new_queue.append(&mut context.chapters);
            context.chapters = new_queue;
        }
        _ => {
            info!("Starting Root Chapter: {:?}", next_chapter);
            spawn_chapter(&mut commands, next_chapter, None);
        }
    }
}

/// Placeholder system for parallel chapter processing.
///
/// 并行章节处理的占位系统。
pub fn process_parallel_chapter_system(
    _commands: Commands,
    _parents: Query<(Entity, &mut ParallelTracker)>,
) {
    // Logic lives in cleanup_finished_chapters_system.
}

/// System to cleanup finished chapters.
///
/// 清理已完成章节的系统。
pub fn cleanup_finished_chapters_system(
    mut commands: Commands,
    finished_query: Query<(Entity, &ActiveChapter), With<ChapterFinished>>,
    mut parallel_parents: Query<&mut ParallelTracker>,
) {
    for (entity, chapter) in finished_query.iter() {
        if let Some(parent_entity) = chapter.parent
            && let Ok(mut tracker) = parallel_parents.get_mut(parent_entity)
        {
            tracker.pending_count = tracker.pending_count.saturating_sub(1);
            if tracker.pending_count == 0 {
                commands.entity(parent_entity).insert(ChapterFinished);
            }
        }

        commands.entity(entity).despawn();
    }
}

/// System to process wait chapters.
///
/// 处理等待章节的系统。
pub fn process_wait_chapter_system(
    mut commands: Commands,
    mut query: Query<(Entity, &mut WaitTimer), Without<ChapterFinished>>,
    time: Res<Time>,
) {
    for (entity, mut timer) in query.iter_mut() {
        timer.0.tick(time.delta());
        if timer.0.is_finished() {
            commands.entity(entity).insert(ChapterFinished);
            info!("Wait Chapter finished.");
        }
    }
}
