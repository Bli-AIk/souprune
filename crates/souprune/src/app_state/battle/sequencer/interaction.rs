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

/// DEPRECATED: This system has been replaced by data-driven state_sprite system.
/// Kept for backwards compatibility reference only.
///
/// 已弃用：此系统已被数据驱动的 state_sprite 系统取代。
/// 仅保留供向后兼容参考。
///
/// System to update sprites based on InteractiveLayer selection.
///
/// 根据 InteractiveLayer 选择更新精灵的系统。
///
/// This system changes the texture of selectable elements to show which one is
/// currently selected (highlighted). When the layer is deactivated, all elements
/// revert to their unselected state.
///
/// 此系统更改可选元素的纹理以显示当前选中（高亮）的项目。
/// 当层被停用时，所有元素恢复到未选中状态。
///
/// # Note / 注意
///
/// This functionality is now handled by:
/// 1. `state_sprite` field in ViewNodeDef (RON configuration)
/// 2. `StateSpriteState` component and associated systems in CoreViewPlugin
///
/// 此功能现在由以下部分处理：
/// 1. ViewNodeDef 中的 `state_sprite` 字段（RON 配置）
/// 2. CoreViewPlugin 中的 `StateSpriteState` 组件和相关系统
#[deprecated(
    since = "0.6.0",
    note = "Use state_sprite configuration in RON instead"
)]
#[allow(dead_code)]
pub fn update_interactive_layer_sprites_system(
    asset_server: Res<AssetServer>,
    layer_query: Query<&InteractiveLayer, Changed<InteractiveLayer>>,
    mut sprite_query: Query<(&Name, &mut Sprite)>,
) {
    for layer in layer_query.iter() {
        // Update each selectable element's sprite
        // If layer is inactive, all elements show unselected state
        for (idx, element_name) in layer.selectable_elements.iter().enumerate() {
            // Only selected if layer is active AND this is the selected index
            let is_selected = layer.is_active && idx == layer.current_selection;

            // Find the entity with this name
            for (name, mut sprite) in sprite_query.iter_mut() {
                if name.as_str() == element_name {
                    // Determine the button type from the element name (e.g., "BtnFight" -> "fight")
                    let button_type = element_name
                        .strip_prefix("Btn")
                        .map(|s| s.to_lowercase())
                        .unwrap_or_else(|| element_name.to_lowercase());

                    // Build the texture path based on selection state
                    let state = if is_selected { "true" } else { "false" };
                    let texture_path = format!("textures/battle/ui/{}/{}.png", button_type, state);

                    // Load and set the new texture
                    let new_texture: Handle<Image> = asset_server.load(&texture_path);
                    sprite.image = new_texture;
                }
            }
        }
    }
}
