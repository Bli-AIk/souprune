//! # sequencer/flow.rs
//!
//! ## Module Overview
//!
//! Battle flow control systems - handles chapter sequencing and lifecycle.
//!
//! 战斗流程控制系统 - 处理章节序列和生命周期。

use super::SequenceAsset;
use super::chapter_schema::Chapter;
use super::context::*;
use bevy::prelude::*;

/// System to load the default chapter resource.
///
/// 加载默认章节资源的系统。
pub fn load_default_chapter_system(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    souprune_config: Res<crate::config::SoupruneConfig>,
) {
    let chapter_path = &souprune_config.game.initial_battle_path;
    let handle = asset_server.load::<SequenceAsset>(chapter_path);
    commands.insert_resource(CurrentSequenceFlow(handle));
    info!("Loading default sequence flow: {}", chapter_path);
}

/// System to sync battle flow when asset is loaded.
/// Also loads sequence-specific FRE rules if specified.
///
/// 当资产加载完成时同步战斗流程的系统。
/// 如果指定了规则文件，也会加载序列特定的 FRE 规则。
pub fn sync_battle_flow_system(
    mut commands: Commands,
    flow_handle: Option<Res<CurrentSequenceFlow>>,
    mut context: ResMut<SequenceContext>,
    assets: Res<Assets<SequenceAsset>>,
    asset_server: Res<AssetServer>,
    mut sequence_rules_handle: ResMut<SequenceRulesHandle>,
) {
    if let Some(handle) = flow_handle
        && let Some(asset) = assets.get(&handle.0)
        && context.chapters.is_empty()
    {
        info!(
            "Sequence flow loaded. Pushing {} chapters to queue.",
            asset.chapters.len()
        );
        context.chapters.extend(asset.chapters.clone());

        // Load sequence-specific rules if specified
        if let Some(rules_path) = &asset.rules_file {
            let rules_handle =
                asset_server.load::<bevy_fact_rule_event::FreAsset>(rules_path.clone());
            sequence_rules_handle.handle = Some(rules_handle);
            sequence_rules_handle.registered = false;
            info!("Sequence FRE: Loading rules from {}", rules_path);
        }

        commands.remove_resource::<CurrentSequenceFlow>();
    }
}

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
    // Check if any root-level chapter is active
    // 检查是否有任何根级章节处于活动状态
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
            // Unpack sequence to the front of the queue
            let mut new_queue = sub_chapters;
            new_queue.append(&mut context.chapters);
            context.chapters = new_queue;
            // Loop again next frame to pick up the first item
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
    // Placeholder to keep the system chain happy if needed, or remove it.
    // Logic moved to cleanup_finished_chapters_system
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
                // Parent finished!
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
