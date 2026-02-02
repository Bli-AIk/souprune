//! # sequencer/interaction.rs
//!
//! ## Module Overview
//!
//! AwaitInteraction systems for the battle sequencer.
//! This module has been reimplemented to use ViewRoot.local_facts instead of InteractiveLayer.
//!
//! 战斗序列管理器的 AwaitInteraction 系统。
//! 此模块已重新实现，使用 ViewRoot.local_facts 替代 InteractiveLayer。

use super::super::chapter_schema::Chapter;
use super::super::fre::SelectionConfirmedEvent;
use super::context::*;
use crate::core::view::components::ViewRoot;
use bevy::ecs::message::MessageReader;
use bevy::prelude::*;
use bevy_fact_rule_event::FactValue;

/// Marker component to track that this chapter is waiting for interaction.
///
/// 标记组件，用于追踪此 Chapter 正在等待交互。
#[derive(Component)]
pub struct AwaitingInteractionChapter {
    /// The layer ID we're waiting on.
    pub layer_id: String,
}

/// System to process AwaitInteraction chapters.
///
/// 处理 AwaitInteraction 章节的系统。
///
/// This system activates the menu by setting `active: true` in ViewRoot.local_facts
/// and marks the chapter as waiting for player input. The chapter won't finish
/// until the player confirms a selection.
///
/// 此系统通过在 ViewRoot.local_facts 中设置 `active: true` 来激活菜单，
/// 并将章节标记为等待玩家输入。章节不会结束，直到玩家确认选择。
#[allow(clippy::type_complexity)]
pub fn process_await_selection_system(
    mut commands: Commands,
    query: Query<
        (Entity, &ActiveChapter),
        (
            Without<WaitTimer>,
            Without<ChapterFinished>,
            Without<AwaitingInteractionChapter>,
        ),
    >,
    mut view_root_query: Query<&mut ViewRoot>,
) {
    for (chapter_entity, active_chapter) in query.iter() {
        if let Chapter::AwaitInteraction {
            layer_id,
            initial_selection,
        } = &active_chapter.chapter
        {
            info!(
                "[Battle] Starting AwaitInteraction for layer '{}' at index {}",
                layer_id, initial_selection
            );

            // Activate the menu in ViewRoot.local_facts
            let mut found = false;
            for mut view_root in view_root_query.iter_mut() {
                // Set active to true to enable navigation
                view_root.local_facts.set("active", FactValue::Bool(true));
                view_root
                    .local_facts
                    .set("selection", FactValue::Int(*initial_selection as i64));
                view_root.local_facts.set("depth", FactValue::Int(0));

                found = true;
                info!(
                    "[Battle] Activated view menu for layer '{}', waiting for player input",
                    layer_id
                );
                break;
            }

            if !found {
                warn!("[Battle] No ViewRoot found! Chapter will complete immediately.");
                commands.entity(chapter_entity).insert(ChapterFinished);
            } else {
                // Mark this chapter as waiting
                commands
                    .entity(chapter_entity)
                    .insert(AwaitingInteractionChapter {
                        layer_id: layer_id.clone(),
                    });
            }
        }
    }
}

/// System to check for interaction completion and finish the AwaitInteraction chapter.
///
/// 检查交互完成情况并结束 AwaitInteraction 章节的系统。
///
/// Listens for SelectionConfirmedEvent and marks the corresponding chapter as finished.
///
/// 监听 SelectionConfirmedEvent 并将相应的章节标记为完成。
#[allow(clippy::type_complexity)]
pub fn check_await_selection_completion_system(
    mut commands: Commands,
    mut confirm_events: MessageReader<SelectionConfirmedEvent>,
    awaiting_query: Query<(Entity, &AwaitingInteractionChapter)>,
    mut view_root_query: Query<&mut ViewRoot>,
) {
    for event in confirm_events.read() {
        info!(
            "[Battle] Received SelectionConfirmedEvent for layer '{}', index {}",
            event.layer_id, event.selection_index
        );

        // Find the chapter that is waiting for this layer
        for (chapter_entity, awaiting) in awaiting_query.iter() {
            if awaiting.layer_id == event.layer_id {
                // Mark the chapter as finished
                commands.entity(chapter_entity).insert(ChapterFinished);
                commands
                    .entity(chapter_entity)
                    .remove::<AwaitingInteractionChapter>();

                info!(
                    "[Battle] AwaitInteraction for '{}' completed with selection {}",
                    event.layer_id, event.selection_index
                );
            }
        }

        // Deactivate the menu
        for mut view_root in view_root_query.iter_mut() {
            view_root.local_facts.set("active", FactValue::Bool(false));
            view_root.local_facts.set("selection", FactValue::Int(0));
            info!("[Battle] Deactivated view menu, reset selection to 0");
        }
    }
}
