//! Manages chapter entities as they are spawned, waited on, completed, and cleaned up.
//!
//! 管理章节实体从生成、等待、完成到清理的整个生命周期。
//!
//! Contains the generic lifecycle machinery behind the sequencer:
//! spawning active chapters, advancing the root queue, tracking parallel
//! branches, and retiring finished chapter entities. It is the control plane of
//! sequence execution rather than the place where individual chapter semantics live.
//!
//! 放的是 sequencer 背后的通用生命周期机制：生成活动章节、推进根队列、
//! 跟踪并行分支，以及回收已经完成的章节实体。它承担的是序列执行的控制面，
//! 而不是某个具体章节语义本身。

use bevy::prelude::*;
use rand::prelude::*;

use super::super::chapter_schema::Chapter;
use super::super::context::{
    ActiveChapter, ChapterFinished, LoopFrame, ParallelTracker, SequenceContext, WaitTimer,
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

    // Drain queue-manipulation chapters in a loop until we either spawn an
    // entity-backed chapter or run out of work.
    loop {
        if context.chapters.is_empty() {
            // Queue exhausted — check loop stack for refill.
            if !try_refill_loop(&mut context) {
                return;
            }
            continue;
        }

        let next_chapter = context.chapters.remove(0);

        match next_chapter {
            // --- Queue-manipulation chapters (no entity spawned) ---
            Chapter::Sequence(sub_chapters) => {
                let mut new_queue = sub_chapters;
                new_queue.append(&mut context.chapters);
                context.chapters = new_queue;
            }
            Chapter::Loop {
                body,
                max_iterations,
            } => {
                process_loop(&mut context, body, max_iterations);
            }
            Chapter::Break => {
                process_break(&mut context);
            }
            Chapter::RandomPick {
                candidates,
                count,
                allow_repeat,
            } => {
                process_random_pick(&mut context, candidates, count, allow_repeat);
            }
            Chapter::LoopIterationEnd => {
                process_loop_iteration_end(&mut context);
            }

            // --- Entity-backed chapters ---
            _ => {
                info!("Starting Root Chapter: {:?}", next_chapter);
                spawn_chapter(&mut commands, next_chapter, None);
                return;
            }
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

// ---------------------------------------------------------------------------
// Loop / Break / RandomPick helpers
// ---------------------------------------------------------------------------

/// Push a new loop onto the stack and splice its body (+ sentinel) into the queue.
fn process_loop(
    context: &mut ResMut<SequenceContext>,
    body: Vec<Chapter>,
    max_iterations: Option<u32>,
) {
    if body.is_empty() {
        warn!("Loop with empty body — skipping");
        return;
    }

    context.loop_stack.push(LoopFrame {
        body: body.clone(),
        remaining_iterations: max_iterations,
        body_remaining: body.len(),
    });

    // Splice body + sentinel at the front of the queue.
    let mut expanded = body;
    expanded.push(Chapter::LoopIterationEnd);
    expanded.append(&mut context.chapters);
    context.chapters = expanded;
}

/// Handle `LoopIterationEnd` sentinel: decide whether to start the next iteration
/// or finish the loop.
fn process_loop_iteration_end(context: &mut ResMut<SequenceContext>) {
    let Some(frame) = context.loop_stack.last_mut() else {
        warn!("LoopIterationEnd without active loop — ignoring");
        return;
    };

    // Decrement remaining iterations if bounded.
    if let Some(ref mut remaining) = frame.remaining_iterations {
        *remaining = remaining.saturating_sub(1);
        if *remaining == 0 {
            context.loop_stack.pop();
            return;
        }
    }

    // Refill body for next iteration.
    let body = frame.body.clone();
    frame.body_remaining = body.len();
    let mut expanded = body;
    expanded.push(Chapter::LoopIterationEnd);
    expanded.append(&mut context.chapters);
    context.chapters = expanded;
}

/// Handle `Break`: pop the innermost loop and drain remaining body items + sentinel.
fn process_break(context: &mut ResMut<SequenceContext>) {
    if context.loop_stack.pop().is_none() {
        warn!("Break without active loop — ignoring");
        return;
    }

    // Drain until we consume the matching LoopIterationEnd.
    while let Some(ch) = context.chapters.first() {
        if matches!(ch, Chapter::LoopIterationEnd) {
            context.chapters.remove(0);
            break;
        }
        context.chapters.remove(0);
    }
}

/// Try to refill the queue from the top of the loop stack when the queue is empty.
/// Returns `true` if the queue was refilled.
fn try_refill_loop(context: &mut ResMut<SequenceContext>) -> bool {
    let Some(frame) = context.loop_stack.last_mut() else {
        return false;
    };

    // Decrement remaining iterations if bounded.
    if let Some(ref mut remaining) = frame.remaining_iterations {
        *remaining = remaining.saturating_sub(1);
        if *remaining == 0 {
            context.loop_stack.pop();
            return false;
        }
    }

    let body = frame.body.clone();
    frame.body_remaining = body.len();
    let mut expanded = body;
    expanded.push(Chapter::LoopIterationEnd);
    context.chapters = expanded;
    true
}

/// Handle `RandomPick`: select from candidates and splice into queue.
fn process_random_pick(
    context: &mut ResMut<SequenceContext>,
    candidates: Vec<Chapter>,
    count: usize,
    allow_repeat: bool,
) {
    if candidates.is_empty() {
        warn!("RandomPick with empty candidates — skipping");
        return;
    }

    let mut rng = rand::rng();
    let mut picked: Vec<Chapter> = Vec::with_capacity(count);

    if allow_repeat {
        for _ in 0..count {
            let idx = rng.random_range(0..candidates.len());
            picked.push(candidates[idx].clone());
        }
    } else {
        let pick_count = count.min(candidates.len());
        let mut indices: Vec<usize> = (0..candidates.len()).collect();
        indices.partial_shuffle(&mut rng, pick_count);
        for &idx in &indices[..pick_count] {
            picked.push(candidates[idx].clone());
        }
    }

    // Splice picked chapters at the front.
    picked.append(&mut context.chapters);
    context.chapters = picked;
}
