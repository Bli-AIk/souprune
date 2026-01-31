//! # sequencer/interaction.rs
//!
//! ## Module Overview
//!
//! AwaitInteraction systems for the battle sequencer.
//!
//! 战斗序列管理器的 AwaitInteraction 系统。

use super::super::chapter_schema::Chapter;
use super::context::*;
use crate::core::view::components::{
    AwaitingInteraction, InteractionResult, InteractiveLayer, SelectionConfirmedEvent,
};
use bevy::ecs::message::MessageReader;
use bevy::prelude::*;

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
/// This system activates the specified interactive layer and marks the chapter
/// as waiting for player input. The chapter won't finish until the player
/// confirms a selection.
///
/// 此系统激活指定的交互层，并将章节标记为等待玩家输入。
/// 章节不会结束，直到玩家确认选择。
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
    mut layer_query: Query<(Entity, &mut InteractiveLayer)>,
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

            // Find and activate the interactive layer
            let mut found = false;
            for (layer_entity, mut layer) in layer_query.iter_mut() {
                if layer.layer_id == *layer_id {
                    // Activate the layer
                    layer.is_active = true;
                    layer.set_selection(*initial_selection);

                    // Attach AwaitingInteraction component to the layer
                    commands.entity(layer_entity).insert(AwaitingInteraction {
                        chapter_entity: Some(chapter_entity),
                    });

                    found = true;
                    info!(
                        "[Battle] Activated InteractiveLayer '{}', waiting for player input",
                        layer_id
                    );
                    break;
                }
            }

            if !found {
                warn!(
                    "[Battle] InteractiveLayer '{}' not found! Chapter will complete immediately.",
                    layer_id
                );
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
    mut layer_query: Query<(Entity, &mut InteractiveLayer), With<AwaitingInteraction>>,
) {
    for event in confirm_events.read() {
        info!(
            "[Battle] Received SelectionConfirmedEvent for layer '{}', index {}",
            event.layer_id, event.selected_index
        );

        // Find the chapter that is waiting for this layer
        for (chapter_entity, awaiting) in awaiting_query.iter() {
            if awaiting.layer_id == event.layer_id {
                // Store the result on the chapter entity for later use
                commands
                    .entity(chapter_entity)
                    .insert(InteractionResult::new(
                        event.selected_index,
                        event.selected_element.clone(),
                        &event.layer_id,
                    ));

                // Mark the chapter as finished
                commands.entity(chapter_entity).insert(ChapterFinished);
                commands
                    .entity(chapter_entity)
                    .remove::<AwaitingInteractionChapter>();

                info!(
                    "[Battle] AwaitInteraction for '{}' completed with selection {}",
                    event.layer_id, event.selected_index
                );
            }
        }

        // Deactivate the layer, reset selection, and remove AwaitingInteraction
        for (layer_entity, mut layer) in layer_query.iter_mut() {
            if layer.layer_id == event.layer_id {
                layer.is_active = false;
                layer.set_selection(0); // Reset to initial selection
                commands
                    .entity(layer_entity)
                    .remove::<AwaitingInteraction>();
                info!(
                    "[Battle] Deactivated InteractiveLayer '{}', reset selection to 0",
                    event.layer_id
                );
            }
        }
    }
}
