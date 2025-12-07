//! # object_properties.rs
//!
//! ## Module Overview
//! This module handles custom property detection and processing for Tiled objects.
//! It provides a flexible system for adding new object property handlers in the future.
//!
//! ## 模块概述
//! 该模块处理Tiled对象的自定义属性检测和处理。
//! 它为未来添加新的对象属性处理器提供了灵活的系统。

use crate::app_state::overworld::tilemap::systems::TilemapCollider;
use crate::core::collision::Rect2DCollider;
use bevy::prelude::*;
use bevy_ecs_tiled::prelude::{TiledMap, TiledMapAsset, tiled};

/// Marker component for objects with collision property
/// 具有碰撞属性的对象的标记组件
#[derive(Component)]
pub struct ObjectCollider;

/// System to directly create collision entities for objects with collision properties
/// 直接为具有碰撞属性的对象创建碰撞实体的系统
#[allow(dead_code)]
pub fn process_map_object_properties_system(
    mut commands: Commands,
    tiled_map_assets: Res<Assets<TiledMapAsset>>,
    tiled_maps_query: Query<&TiledMap>,
    existing_colliders: Query<&ObjectCollider>,
) {
    // Add debug logging
    let map_count = tiled_maps_query.iter().count();
    let collider_count = existing_colliders.iter().count();

    info!(
        "Object properties system running: {} maps, {} existing colliders",
        map_count, collider_count
    );

    // Only run if we haven't created object colliders yet
    if !existing_colliders.is_empty() {
        info!("Object colliders already exist, skipping");
        return;
    }

    for tiled_map_handle in tiled_maps_query.iter() {
        info!("Processing tiled map handle");

        if let Some(tiled_map_asset) = tiled_map_assets.get(&tiled_map_handle.0) {
            info!("Found tiled map asset");

            // Calculate map center offset (same as tilemap collision system)
            let tile_size = tiled_map_asset.map.tile_width as f32;
            let tile_height = tiled_map_asset.map.tile_height as f32;
            let map_width = tiled_map_asset.map.width as f32 * tile_size;
            let map_height = tiled_map_asset.map.height as f32 * tile_height;
            let center_offset_x = -map_width / 2.0;
            let center_offset_y = -map_height / 2.0;

            for layer in tiled_map_asset.map.layers() {
                info!("Processing layer: {}", layer.name);

                if let Some(object_layer) = layer.as_object_layer() {
                    info!(
                        "Processing object layer '{}' with {} objects",
                        layer.name,
                        object_layer.objects().count()
                    );

                    for object_data in object_layer.objects() {
                        info!(
                            "Checking object '{}' at ({}, {}) with shape {:?}",
                            object_data.name, object_data.x, object_data.y, object_data.shape
                        );

                        // Check if this object has collision property set to true
                        if let Some(collision_value) = object_data.properties.get("collision") {
                            info!("Found collision property: {:?}", collision_value);

                            if let tiled::PropertyValue::BoolValue(true) = collision_value {
                                info!(
                                    "Found collision object '{}' with collision=true",
                                    object_data.name
                                );

                                if let tiled::ObjectShape::Rect { width, height } =
                                    object_data.shape
                                {
                                    // Calculate world position (same coordinate system as tilemap)
                                    // Tiled uses top-left origin, convert to center-based
                                    let world_x = center_offset_x + object_data.x + width / 2.0;
                                    let world_y = center_offset_y
                                        + (tiled_map_asset.map.height as f32 * tile_height
                                            - object_data.y
                                            - height / 2.0);

                                    let size = Vec2::new(width, height);

                                    info!(
                                        "Creating collision object '{}' at world pos ({}, {}) with size ({}, {})",
                                        object_data.name, world_x, world_y, width, height
                                    );

                                    commands.spawn((
                                        ObjectCollider,
                                        TilemapCollider,
                                        Rect2DCollider::new(size, Vec2::ZERO),
                                        Transform::from_xyz(world_x, world_y, 0.0),
                                        Visibility::Hidden, // Hide the object entity itself
                                        Name::new(format!("ObjectCollision_{}", object_data.name)),
                                    ));
                                } else {
                                    info!("Object '{}' is not a rectangle", object_data.name);
                                }
                            }
                        } else {
                            info!("Object '{}' has no collision property", object_data.name);
                        }
                    }
                } else {
                    info!("Layer '{}' is not an object layer", layer.name);
                }
            }
            break; // Process only the first map for now
        } else {
            info!("Could not get tiled map asset");
        }
    }
}
