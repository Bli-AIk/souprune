//! # object_properties.rs
//!
//! ## Module Overview
//! This module handles custom property detection and processing for Tiled objects.
//! It provides a flexible system for adding new object property handlers in the future.
//!
//! ## 模块概述
//! 该模块处理Tiled对象的自定义属性检测和处理。
//! 它为未来添加新的对象属性处理器提供了灵活的系统。

use crate::app_state::overworld::OverworldEntity;
use crate::app_state::overworld::tilemap::systems::TilemapCollider;
use crate::app_state::overworld::trigger::{Interactable, TriggerZone};
use crate::core::collision::Rect2DCollider;
use crate::core::map_property_schema::{get_object_bool_property, object_keys};
use bevy::prelude::*;
use bevy_ecs_tiled::prelude::{TiledMap, TiledMapAsset, tiled};

/// Marker component for objects with collision property
/// 具有碰撞属性的对象的标记组件
#[derive(Component)]
pub struct ObjectCollider;

/// Marker component for trigger zones spawned from Tiled
/// 从 Tiled 生成的触发区域的标记组件
#[derive(Component)]
pub struct TiledTriggerZone;

/// Marker component for interactable objects spawned from Tiled
/// 从 Tiled 生成的可交互物体的标记组件
#[derive(Component)]
pub struct TiledInteractable;

/// System to process Tiled object layer properties and spawn corresponding entities.
/// Handles both collision objects and trigger zones.
///
/// 处理 Tiled 对象层属性并生成相应实体的系统。
/// 处理碰撞对象和触发区域。
#[allow(dead_code)]
pub fn process_map_object_properties_system(
    mut commands: Commands,
    tiled_map_assets: Res<Assets<TiledMapAsset>>,
    tiled_maps_query: Query<&TiledMap>,
    existing_triggers: Query<&TiledTriggerZone>,
    existing_interactables: Query<&TiledInteractable>,
    mut processed: Local<bool>,
) {
    // Only process once
    if *processed {
        return;
    }

    let map_count = tiled_maps_query.iter().count();
    let trigger_count = existing_triggers.iter().count();
    let interactable_count = existing_interactables.iter().count();

    // Wait for map to be available
    if map_count == 0 {
        return;
    }

    // Try to get map asset - if not loaded yet, wait
    let mut any_map_loaded = false;
    for tiled_map_handle in tiled_maps_query.iter() {
        if tiled_map_assets.get(&tiled_map_handle.0).is_some() {
            any_map_loaded = true;
            break;
        }
    }

    if !any_map_loaded {
        return;
    }

    info!(
        "Object properties system processing: {} maps, {} triggers, {} interactables",
        map_count, trigger_count, interactable_count
    );

    for tiled_map_handle in tiled_maps_query.iter() {
        let Some(tiled_map_asset) = tiled_map_assets.get(&tiled_map_handle.0) else {
            continue;
        };

        info!("Processing tiled map for triggers and interactables");

        // Calculate map center offset (Tiled uses top-left origin, Bevy uses center)
        let tile_width = tiled_map_asset.map.tile_width as f32;
        let tile_height = tiled_map_asset.map.tile_height as f32;
        let map_width = tiled_map_asset.map.width as f32 * tile_width;
        let map_height = tiled_map_asset.map.height as f32 * tile_height;
        let center_offset_x = -map_width / 2.0;
        let center_offset_y = -map_height / 2.0;

        for layer in tiled_map_asset.map.layers() {
            let Some(object_layer) = layer.as_object_layer() else {
                continue;
            };

            trace!(
                "Processing object layer '{}' with {} objects",
                layer.name,
                object_layer.objects().count()
            );

            for object_data in object_layer.objects() {
                trace!(
                    "Checking object '{}' at ({}, {}) with shape {:?}",
                    object_data.name, object_data.x, object_data.y, object_data.shape
                );

                // Note: collision objects are handled by generate_collision_tiles_system
                // Skip collision objects here to avoid duplicates
                if get_object_bool_property(&object_data.properties, object_keys::COLLISION)
                    == Some(true)
                {
                    continue;
                }

                // Handle trigger zones
                if get_object_bool_property(&object_data.properties, object_keys::TRIGGER)
                    == Some(true)
                {
                    spawn_trigger_zone(
                        &mut commands,
                        &object_data,
                        center_offset_x,
                        center_offset_y,
                        map_height,
                    );
                    continue;
                }

                // Handle interactable objects
                let is_interactable =
                    get_object_bool_property(&object_data.properties, object_keys::INTERACTABLE);
                info!(
                    "Object '{}' interactable property: {:?}",
                    object_data.name, is_interactable
                );
                if is_interactable == Some(true) {
                    spawn_interactable(
                        &mut commands,
                        &object_data,
                        center_offset_x,
                        center_offset_y,
                        map_height,
                    );
                }
            }
        }

        // Process only the first map for now
        break;
    }

    // Mark as processed
    *processed = true;
    info!("Object properties system completed processing");
}

/// Spawn a collision entity from a Tiled object.
fn spawn_collision_object(
    commands: &mut Commands,
    object_data: &tiled::ObjectData,
    center_offset_x: f32,
    center_offset_y: f32,
    map_height: f32,
) {
    let tiled::ObjectShape::Rect { width, height } = object_data.shape else {
        trace!("Collision object '{}' is not a rectangle", object_data.name);
        return;
    };

    // Convert Tiled coordinates (top-left origin) to Bevy (center-based)
    let world_x = center_offset_x + object_data.x + width / 2.0;
    let world_y = center_offset_y + (map_height - object_data.y - height / 2.0);
    let size = Vec2::new(width, height);

    trace!(
        "Creating collision object '{}' at ({}, {}) size ({}, {})",
        object_data.name, world_x, world_y, width, height
    );

    commands.spawn((
        ObjectCollider,
        TilemapCollider,
        Rect2DCollider::new(size, Vec2::ZERO),
        Transform::from_xyz(world_x, world_y, 0.0),
        Visibility::Hidden,
        Name::new(format!("ObjectCollision_{}", object_data.name)),
    ));
}

/// Spawn a trigger zone entity from a Tiled object.
fn spawn_trigger_zone(
    commands: &mut Commands,
    object_data: &tiled::ObjectData,
    center_offset_x: f32,
    center_offset_y: f32,
    map_height: f32,
) {
    let tiled::ObjectShape::Rect { width, height } = object_data.shape else {
        trace!("Trigger object '{}' is not a rectangle", object_data.name);
        return;
    };

    // Use object name if provided, otherwise use Tiled object ID
    let trigger_id = if !object_data.name.is_empty() {
        object_data.name.clone()
    } else {
        format!("trigger_{}", object_data.id())
    };

    // Convert Tiled coordinates (top-left origin) to Bevy (center-based)
    let world_x = center_offset_x + object_data.x + width / 2.0;
    let world_y = center_offset_y + (map_height - object_data.y - height / 2.0);
    let size = Vec2::new(width, height);

    info!(
        "FRE: Creating trigger zone '{}' at ({:.1}, {:.1}) size ({}, {})",
        trigger_id, world_x, world_y, width, height
    );

    commands.spawn((
        OverworldEntity(),
        TiledTriggerZone,
        TriggerZone::new(&trigger_id),
        Rect2DCollider::new(size, Vec2::ZERO),
        Transform::from_xyz(world_x, world_y, 0.0),
        Visibility::Hidden,
        Name::new(format!("TriggerZone_{}", trigger_id)),
    ));
}

/// Spawn an interactable entity from a Tiled object.
fn spawn_interactable(
    commands: &mut Commands,
    object_data: &tiled::ObjectData,
    center_offset_x: f32,
    center_offset_y: f32,
    map_height: f32,
) {
    let tiled::ObjectShape::Rect { width, height } = object_data.shape else {
        trace!(
            "Interactable object '{}' is not a rectangle",
            object_data.name
        );
        return;
    };

    // Use object name if provided, otherwise use Tiled object ID
    let interactable_id = if !object_data.name.is_empty() {
        object_data.name.clone()
    } else {
        format!("interactable_{}", object_data.id())
    };

    // Convert Tiled coordinates (top-left origin) to Bevy (center-based)
    let world_x = center_offset_x + object_data.x + width / 2.0;
    let world_y = center_offset_y + (map_height - object_data.y - height / 2.0);
    let size = Vec2::new(width, height);

    info!(
        "FRE: Creating interactable '{}' at ({:.1}, {:.1}) size ({}, {})",
        interactable_id, world_x, world_y, width, height
    );

    commands.spawn((
        OverworldEntity(),
        TiledInteractable,
        Interactable::new(&interactable_id),
        Rect2DCollider::new(size, Vec2::ZERO),
        Transform::from_xyz(world_x, world_y, 0.0),
        Visibility::Hidden,
        Name::new(format!("Interactable_{}", interactable_id)),
    ));
}
