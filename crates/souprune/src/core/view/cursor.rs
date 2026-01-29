//! # cursor.rs
//!
//! # cursor.rs 文件
//!
//! ## Module Overview
//!
//! ## 模块概述
//!
//! This module manages the visual cursor that indicates the currently selected UI element.
//!
//! 本模块管理指示当前选中 UI 元素的可视光标。
//!
//! ## Source File Overview
//!
//! ## 源文件概述
//!
//! It handles spawning cursor sprites and updating cursor positions based on navigation events.
//!
//! 处理光标精灵的生成和基于导航事件的光标位置更新。

use super::components::{
    BoxCursor, BoxCursorOwner, BoxCursorReady, BoxCursorSprite, InteractiveLayer,
    LayerActivatedEvent, LayerDeactivatedEvent, SelectionChangedEvent, UIBox, UIBoxFiller, UILayer,
};
use crate::app_state::overworld::OverworldState;
use bevy::ecs::message::MessageReader;
use bevy::ecs::relationship::Relationship;
use bevy::prelude::*;
use std::collections::VecDeque;

fn find_ui_box_filler_entity(
    root: Entity,
    children_query: &Query<&Children>,
    filler_query: &Query<(), With<UIBoxFiller>>,
) -> Option<Entity> {
    let mut queue: VecDeque<Entity> = VecDeque::new();
    if let Ok(children) = children_query.get(root) {
        for child in children.iter() {
            queue.push_back(child);
        }
    }

    while let Some(child) = queue.pop_front() {
        if filler_query.get(child).is_ok() {
            return Some(child);
        }

        if let Ok(children) = children_query.get(child) {
            for grandchild in children.iter() {
                queue.push_back(grandchild);
            }
        }
    }

    None
}

/// Spawn the sprite used to represent the Undertale box cursor once the UI filler exists.
///
/// 当 UI 填充实体就绪后，生成 Undertale 样式的光标贴图。
#[allow(clippy::type_complexity)]
pub(crate) fn spawn_box_cursor_visual_system(
    mut commands: Commands,
    query: Query<(Entity, &BoxCursor), (With<UIBox>, Without<BoxCursorReady>)>,
    children_query: Query<&Children>,
    filler_query: Query<(), With<UIBoxFiller>>,
) {
    for (entity, cursor) in query.iter() {
        let Some(filler_entity) = find_ui_box_filler_entity(entity, &children_query, &filler_query)
        else {
            continue;
        };

        let mut cursor_transform = cursor.transform();
        cursor_transform.translation = Vec3::ZERO;

        commands.entity(filler_entity).with_children(|parent| {
            parent.spawn((
                Name::new("BoxCursorSprite"),
                BoxCursorSprite,
                BoxCursorOwner(entity),
                cursor.sprite(),
                cursor_transform,
                Visibility::Hidden,
            ));
        });

        commands.entity(entity).insert(BoxCursorReady);
    }
}

/// Update cursor position and visibility based on layer and selection events.
/// This is a reactive system that only updates when events are received.
///
/// 基于层和选择事件更新光标位置和可见性。
/// 这是一个响应式系统，仅在收到事件时更新。
pub(crate) fn update_box_cursor_state_system(
    overworld_state: Res<State<OverworldState>>,
    interactive_layer_query: Query<&InteractiveLayer>,
    mut selection_changed: MessageReader<SelectionChangedEvent>,
    mut layer_activated: MessageReader<LayerActivatedEvent>,
    mut layer_deactivated: MessageReader<LayerDeactivatedEvent>,
    mut box_query: Query<&mut BoxCursor, With<UIBox>>,
    parent_query: Query<&ChildOf>,
    mut sprite_query: Query<
        (&BoxCursorOwner, &mut Transform, &mut Visibility),
        With<BoxCursorSprite>,
    >,
) {
    let _ = &parent_query; // Suppress unused warning

    // Handle selection changed events (cursor position update)
    for event in selection_changed.read() {
        let ui_layer = UILayer::new(event.layer_id.clone());

        for (owner, mut transform, mut visibility) in sprite_query.iter_mut() {
            let Ok(mut cursor) = box_query.get_mut(owner.0) else {
                continue;
            };

            // Check if this cursor should respond to this layer
            if !cursor.visibility().is_visible_for(&ui_layer) {
                continue;
            }

            // Update cursor position
            if let Some(translation) = cursor.translation_for_index(&ui_layer, event.new_index) {
                if transform.translation != translation {
                    transform.translation = translation;
                }
            }

            // Ensure cursor is visible (if not hidden and in correct state)
            let should_show =
                overworld_state.get() == &OverworldState::Backpack && !cursor.is_hidden();
            if should_show && *visibility != Visibility::Inherited {
                *visibility = Visibility::Inherited;
            }
        }
    }

    // Handle layer activated events (show cursor and set initial position)
    for event in layer_activated.read() {
        let ui_layer = UILayer::new(event.layer_id.clone());

        for (owner, mut transform, mut visibility) in sprite_query.iter_mut() {
            let Ok(mut cursor) = box_query.get_mut(owner.0) else {
                continue;
            };

            // Check if this cursor should respond to this layer
            if !cursor.visibility().is_visible_for(&ui_layer) {
                continue;
            }

            // Set initial cursor position
            if let Some(translation) =
                cursor.translation_for_index(&ui_layer, event.current_selection)
            {
                transform.translation = translation;
            }

            // Show cursor if in correct state
            let should_show =
                overworld_state.get() == &OverworldState::Backpack && !cursor.is_hidden();
            if should_show {
                *visibility = Visibility::Inherited;
            }
        }
    }

    // Handle layer deactivated events (hide cursor)
    for event in layer_deactivated.read() {
        let ui_layer = UILayer::new(event.layer_id.clone());

        for (owner, _transform, mut visibility) in sprite_query.iter_mut() {
            let Ok(mut cursor) = box_query.get_mut(owner.0) else {
                continue;
            };

            // Check if this cursor should respond to this layer
            if !cursor.visibility().is_visible_for(&ui_layer) {
                continue;
            }

            // Hide cursor when layer is deactivated
            *visibility = Visibility::Hidden;
        }
    }

    // Fallback: If no events but cursor needs initial state sync
    // This handles the case where a layer is already active when cursor spawns
    // 后备处理：如果没有事件但光标需要初始状态同步
    // 这处理光标生成时层已经处于活跃状态的情况
    let active_layer = interactive_layer_query.iter().find(|layer| layer.is_active);

    for (owner, mut transform, mut visibility) in sprite_query.iter_mut() {
        let Ok(mut cursor) = box_query.get_mut(owner.0) else {
            if *visibility != Visibility::Hidden {
                *visibility = Visibility::Hidden;
            }
            continue;
        };

        // Get the active layer or hide cursor
        let Some(layer) = active_layer else {
            if *visibility != Visibility::Hidden {
                *visibility = Visibility::Hidden;
            }
            continue;
        };

        // Create UILayer from layer_id for visibility check
        let ui_layer = UILayer::new(layer.layer_id.clone());

        let mut should_show = overworld_state.get() == &OverworldState::Backpack;
        should_show &= !cursor.is_hidden();
        should_show &= cursor.visibility().is_visible_for(&ui_layer);

        if should_show {
            if let Some(translation) =
                cursor.translation_for_index(&ui_layer, layer.current_selection)
                && transform.translation != translation
            {
                transform.translation = translation;
            }
            if *visibility != Visibility::Inherited {
                *visibility = Visibility::Inherited;
            }
        } else if *visibility != Visibility::Hidden {
            *visibility = Visibility::Hidden;
        }
    }
}
