//! # sequencer/interaction.rs
//!
//! ## Module Overview
//!
//! AwaitFact systems for the battle sequencer.
//! This module implements reactive fact-based blocking for chapter sequences.
//!
//! 战斗序列管理器的 AwaitFact 系统。
//! 此模块实现基于 Fact 条件的响应式阻塞机制。

use super::chapter_schema::Chapter;
use super::context::*;
use crate::core::fre_bridge::evaluate_single_condition;
use crate::core::view::components::ViewRoot;
use bevy::prelude::*;
use bevy_fact_rule_event::LayeredFactDatabase;

/// Marker component to track that this chapter is waiting for a fact condition.
///
/// 标记组件，用于追踪此 Chapter 正在等待 Fact 条件。
#[derive(Component)]
pub struct AwaitingFactChapter {
    /// The condition expression to evaluate.
    pub condition: String,
    /// Whether to use View's local_facts or global FactDatabase.
    pub local: bool,
}

/// System to process AwaitFact chapters - mark them as awaiting.
///
/// 处理 AwaitFact 章节的系统 - 标记为等待状态。
pub fn process_await_fact_system(
    mut commands: Commands,
    query: Query<
        (Entity, &ActiveChapter),
        (
            Without<WaitTimer>,
            Without<ChapterFinished>,
            Without<AwaitingFactChapter>,
        ),
    >,
) {
    for (chapter_entity, active_chapter) in query.iter() {
        if let Chapter::AwaitFact { condition, local } = &active_chapter.chapter {
            info!(
                "[Battle] Starting AwaitFact with condition: '{}' (local: {})",
                condition, local
            );

            commands.entity(chapter_entity).insert(AwaitingFactChapter {
                condition: condition.clone(),
                local: *local,
            });
        }
    }
}

/// System to check if fact condition is met and finish the AwaitFact chapter.
///
/// 检查 Fact 条件是否满足并结束 AwaitFact 章节的系统。
pub fn check_await_fact_completion_system(
    mut commands: Commands,
    awaiting_query: Query<(Entity, &AwaitingFactChapter)>,
    view_root_query: Query<&ViewRoot>,
    global_facts: Res<LayeredFactDatabase>,
) {
    for (chapter_entity, awaiting) in awaiting_query.iter() {
        let condition_met = if awaiting.local {
            // Use View's local_facts
            if let Some(view_root) = view_root_query.iter().next() {
                evaluate_single_condition(
                    &awaiting.condition,
                    &view_root.local_facts,
                    &global_facts,
                )
            } else {
                warn!("[Battle] No ViewRoot found for local fact evaluation!");
                false
            }
        } else {
            // Use global FactDatabase
            let empty_local = bevy_fact_rule_event::FactDatabase::new();
            evaluate_single_condition(&awaiting.condition, &empty_local, &global_facts)
        };

        if condition_met {
            info!(
                "[Battle] AwaitFact condition '{}' met, completing chapter",
                awaiting.condition
            );
            commands.entity(chapter_entity).insert(ChapterFinished);
            commands
                .entity(chapter_entity)
                .remove::<AwaitingFactChapter>();
        }
    }
}
