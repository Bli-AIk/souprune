//! # reactive.rs
//!
//! # reactive.rs 文件
//!
//! ## Module Overview
//!
//! ## 模块概述
//!
//! This module manages reactive indicator elements that respond to UI events.
//! These elements update their position and visibility based on selection changes
//! and layer activations.
//!
//! 本模块管理响应 UI 事件的响应式指示器元素。
//! 这些元素根据选择变更和层激活来更新位置和可见性。
//!
//! ## Source File Overview
//!
//! ## 源文件概述
//!
//! It handles spawning reactive indicator sprites and updating their state based on events.
//!
//! 处理响应式指示器精灵的生成和基于事件的状态更新。

use super::components::{
    InteractiveLayer, LayerActivatedEvent, LayerDeactivatedEvent, ReactiveIndicator,
    ReactiveIndicatorOwner, ReactiveIndicatorReady, ReactiveIndicatorSprite, SelectionChangedEvent,
    ViewBox, ViewBoxFiller, ViewLayer,
};
use crate::app_state::overworld::OverworldState;
use bevy::ecs::message::MessageReader;
use bevy::prelude::*;
use std::collections::VecDeque;

fn find_ui_box_filler_entity(
    root: Entity,
    children_query: &Query<&Children>,
    filler_query: &Query<(), With<ViewBoxFiller>>,
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

/// Spawn the sprite for a reactive indicator once the UI filler exists.
///
/// 当 UI 填充实体就绪后，生成响应式指示器的精灵。
#[allow(clippy::type_complexity)]
pub(crate) fn spawn_reactive_indicator_system(
    mut commands: Commands,
    query: Query<(Entity, &ReactiveIndicator), (With<ViewBox>, Without<ReactiveIndicatorReady>)>,
    children_query: Query<&Children>,
    filler_query: Query<(), With<ViewBoxFiller>>,
) {
    for (entity, indicator) in query.iter() {
        let Some(filler_entity) = find_ui_box_filler_entity(entity, &children_query, &filler_query)
        else {
            continue;
        };

        let mut indicator_transform = indicator.transform();
        indicator_transform.translation = Vec3::ZERO;

        commands.entity(filler_entity).with_children(|parent| {
            parent.spawn((
                Name::new("ReactiveIndicatorSprite"),
                ReactiveIndicatorSprite,
                ReactiveIndicatorOwner(entity),
                indicator.sprite(),
                indicator_transform,
                Visibility::Hidden,
            ));
        });

        commands.entity(entity).insert(ReactiveIndicatorReady);
    }
}

/// Update reactive indicator position and visibility based on layer and selection events.
/// This system responds to events rather than polling state every frame.
///
/// 基于层和选择事件更新响应式指示器的位置和可见性。
/// 该系统响应事件而非每帧轮询状态。
pub(crate) fn update_reactive_indicator_system(
    overworld_state: Res<State<OverworldState>>,
    interactive_layer_query: Query<&InteractiveLayer>,
    mut selection_changed: MessageReader<SelectionChangedEvent>,
    mut layer_activated: MessageReader<LayerActivatedEvent>,
    mut layer_deactivated: MessageReader<LayerDeactivatedEvent>,
    mut indicator_query: Query<&mut ReactiveIndicator, With<ViewBox>>,
    parent_query: Query<&ChildOf>,
    mut sprite_query: Query<
        (&ReactiveIndicatorOwner, &mut Transform, &mut Visibility),
        With<ReactiveIndicatorSprite>,
    >,
) {
    let _ = &parent_query; // Suppress unused warning

    // Handle selection changed events (indicator position update)
    for event in selection_changed.read() {
        let ui_layer = ViewLayer::new(event.layer_id.clone());

        for (owner, mut transform, mut visibility) in sprite_query.iter_mut() {
            let Ok(mut indicator) = indicator_query.get_mut(owner.0) else {
                continue;
            };

            // Check if this indicator should respond to this layer
            if !indicator.visibility().is_visible_for(&ui_layer) {
                continue;
            }

            // Update indicator position
            if let Some(translation) = indicator.translation_for_index(&ui_layer, event.new_index)
                && transform.translation != translation
            {
                transform.translation = translation;
            }

            // Ensure indicator is visible (if not hidden and in correct state)
            let should_show =
                overworld_state.get() == &OverworldState::Backpack && !indicator.is_hidden();
            if should_show && *visibility != Visibility::Inherited {
                *visibility = Visibility::Inherited;
            }
        }
    }

    // Handle layer activated events (show indicator and set initial position)
    for event in layer_activated.read() {
        let ui_layer = ViewLayer::new(event.layer_id.clone());

        for (owner, mut transform, mut visibility) in sprite_query.iter_mut() {
            let Ok(mut indicator) = indicator_query.get_mut(owner.0) else {
                continue;
            };

            // Check if this indicator should respond to this layer
            if !indicator.visibility().is_visible_for(&ui_layer) {
                continue;
            }

            // Set initial indicator position
            if let Some(translation) =
                indicator.translation_for_index(&ui_layer, event.current_selection)
            {
                transform.translation = translation;
            }

            // Show indicator if in correct state
            let should_show =
                overworld_state.get() == &OverworldState::Backpack && !indicator.is_hidden();
            if should_show {
                *visibility = Visibility::Inherited;
            }
        }
    }

    // Handle layer deactivated events (hide indicator)
    for event in layer_deactivated.read() {
        let ui_layer = ViewLayer::new(event.layer_id.clone());

        for (owner, _transform, mut visibility) in sprite_query.iter_mut() {
            let Ok(indicator) = indicator_query.get_mut(owner.0) else {
                continue;
            };

            // Check if this indicator should respond to this layer
            if !indicator.visibility().is_visible_for(&ui_layer) {
                continue;
            }

            // Hide indicator when layer is deactivated
            *visibility = Visibility::Hidden;
        }
    }

    // Fallback: If no events but indicator needs initial state sync
    // This handles the case where a layer is already active when indicator spawns
    let active_layer = interactive_layer_query.iter().find(|layer| layer.is_active);

    for (owner, mut transform, mut visibility) in sprite_query.iter_mut() {
        let Ok(mut indicator) = indicator_query.get_mut(owner.0) else {
            if *visibility != Visibility::Hidden {
                *visibility = Visibility::Hidden;
            }
            continue;
        };

        // Get the active layer or hide indicator
        let Some(layer) = active_layer else {
            if *visibility != Visibility::Hidden {
                *visibility = Visibility::Hidden;
            }
            continue;
        };

        // Create ViewLayer from layer_id for visibility check
        let ui_layer = ViewLayer::new(layer.layer_id.clone());

        let mut should_show = overworld_state.get() == &OverworldState::Backpack;
        should_show &= !indicator.is_hidden();
        should_show &= indicator.visibility().is_visible_for(&ui_layer);

        if should_show {
            if let Some(translation) =
                indicator.translation_for_index(&ui_layer, layer.current_selection)
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
