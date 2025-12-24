//! # systems.rs
//!
//! # systems.rs 文件
//!
//! ## Module Overview
//!
//! ## 模块概述
//!
//! This module handles loading Tiled map files and setting up layers.
//!
//! 本模块处理加载 Tiled 地图文件和设置图层。
//!
//! ## Source File Overview
//!
//! ## 源文件概述
//!
//! It spawns collision objects and initializes camera bounds based on map dimensions.
//!
//! 生成碰撞对象并根据地图尺寸初始化摄像机边界。

use crate::app_state::overworld::{OverworldEntity, character};
use crate::core::animation::components::SpriteAnimationClip;
use crate::core::camera::components::Followable;
use crate::core::collision::Rect2DCollider;
use bevy::asset::{AssetServer, Assets};
use bevy::log::info;
use bevy::prelude::{
    Added, Camera, Commands, Component, Entity, Name, Query, Res, ResMut, Sprite, Transform, Vec2,
    Visibility, Window, With, Without,
};
use bevy_ecs_tiled::prelude::{
    TiledLayer, TiledMap, TiledMapAsset, TiledMapLayerZOffset, TiledObject, TilemapAnchor, tiled,
};

/// Marker component for tilemap collision entities.
///
/// 瓦片地图碰撞实体的标记组件。
#[derive(Component)]
pub struct TilemapCollider;

/// Root entity for organizing collision tiles.
///
/// 用于组织碰撞瓦片的根实体。
#[derive(Component)]
pub struct CollisionTileGroup;

/// Root entity for organizing object collisions.
///
/// 用于组织对象碰撞的根实体。
#[derive(Component)]
pub struct ObjectCollisionGroup;

pub fn setup_tilemap_system(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((
        OverworldEntity(),
        TiledMap(asset_server.load("levels/ruins/ruins_3.tmx")),
        TilemapAnchor::Center,
        TiledMapLayerZOffset(10.0),
    ));
    // TODO: Tilemap的资源加载（或许）应当在 AppSetup 阶段完成。
}

/// Initialize Tilemap layers, filter and hide layers with "prototype" in their names,
/// and generate collision for "collision" layers.
///
/// 初始化 Tilemap 图层，过滤并隐藏包含 "prototype" 的图层名称，
/// 并根据图层顺序设置其他图层的 z 轴位置，同时为 "collision" 图层生成碰撞
pub fn initialize_tilemap_system(
    mut commands: Commands,
    layers_query: Query<(Entity, &Name), Added<TiledLayer>>,
) {
    let mut layers: Vec<_> = layers_query.iter().collect();

    layers.sort_by_key(|(entity, _)| entity.index());

    for (index, (layer_entity, layer_name)) in layers.iter().enumerate() {
        let layer_name_str = layer_name.as_str();

        let name_lower = layer_name_str.to_ascii_lowercase();
        if ["prototype", "collision"]
            .iter()
            .any(|s| name_lower.contains(s))
        {
            if name_lower.contains("collision") {
                info!(
                    "Hide collision layer: {} and generate collision tiles",
                    layer_name_str
                );
            } else {
                info!("Hide prototype layer: {}", layer_name_str);
            }
            commands.entity(*layer_entity).insert(Visibility::Hidden);
        } else {
            info!("Show layers: {}", layer_name_str);

            let z_offset = -2.0 - (layers.len() as f32 - 1.0 - index as f32) * 0.5;

            commands
                .entity(*layer_entity)
                .insert(Transform::from_xyz(0.0, 0.0, z_offset));
            info!(
                "Set the Z-axis position of layer {} (order {}): {}",
                layer_name_str, index, z_offset
            );
        }
    }
}

type CollisionLayersQuery<'w, 's> = Query<
    'w,
    's,
    (Entity, &'static Name, &'static TiledLayer),
    (Added<TiledLayer>, With<Visibility>),
>;
/// Generate collision tiles for collision layers after the layer is initialized
///
/// 在图层初始化后为碰撞图层生成碰撞瓦片
pub fn generate_collision_tiles_system(
    mut commands: Commands,
    tiled_map_assets: Res<Assets<TiledMapAsset>>,
    tiled_maps_query: Query<(Entity, &TiledMap)>,
    collision_layers: CollisionLayersQuery,
    existing_object_colliders: Query<&crate::app_state::overworld::tilemap::ObjectCollider>,
    collision_tile_group: Query<Entity, With<CollisionTileGroup>>,
    object_collision_group: Query<Entity, With<ObjectCollisionGroup>>,
) {
    // Create collision tile group if it doesn't exist
    //
    // 如果不存在，创建碰撞瓦片组
    let collision_tile_group_entity = if let Ok(entity) = collision_tile_group.single() {
        entity
    } else {
        commands
            .spawn((
                OverworldEntity(),
                CollisionTileGroup,
                Name::new("CollisionTiles"),
                Transform::default(),
                Visibility::default(),
            ))
            .id()
    };

    // Create object collision group if it doesn't exist
    //
    // 如果不存在，创建对象碰撞组
    let object_collision_group_entity = if let Ok(entity) = object_collision_group.single() {
        entity
    } else {
        commands
            .spawn((
                OverworldEntity(),
                ObjectCollisionGroup,
                Name::new("ObjectCollisions"),
                Transform::default(),
                Visibility::default(),
            ))
            .id()
    };

    for (layer_entity, layer_name, _tiled_layer) in collision_layers.iter() {
        if !is_collision_layer(layer_name.as_str()) {
            continue;
        }

        if let Some((tiled_map_asset, matching_layer)) =
            find_matching_layer(&tiled_maps_query, &tiled_map_assets, layer_name.as_str())
        {
            generate_tiles_for_layer(
                &mut commands,
                tiled_map_asset,
                &matching_layer,
                collision_tile_group_entity,
            );

            // Also process object collision while we have the map data.
            //
            // 在处理瓦片碰撞的同时处理对象碰撞。
            if existing_object_colliders.is_empty() {
                generate_object_colliders(
                    &mut commands,
                    tiled_map_asset,
                    object_collision_group_entity,
                );
            }
        }

        commands.entity(layer_entity).insert(TilemapCollider);
    }
}

/// Check if a layer name indicates it's a collision layer.
///
/// 检查图层名是否表示它是碰撞图层。
fn is_collision_layer(layer_name: &str) -> bool {
    layer_name.to_ascii_lowercase().contains("collision")
}

/// Find the matching layer in the tiled map assets.
///
/// 在瓦片地图资源中查找匹配的图层。
fn find_matching_layer<'a>(
    tiled_maps_query: &Query<(Entity, &TiledMap)>,
    tiled_map_assets: &'a Res<Assets<TiledMapAsset>>,
    layer_name: &str,
) -> Option<(&'a TiledMapAsset, tiled::Layer<'a>)> {
    for (_map_entity, tiled_map_handle) in tiled_maps_query.iter() {
        let tiled_map_asset = tiled_map_assets.get(&tiled_map_handle.0)?;

        for layer in tiled_map_asset.map.layers() {
            if is_layer_match(&layer.name, layer_name) {
                return Some((tiled_map_asset, layer));
            }
        }
    }
    None
}

/// Check if two layer names match.
///
/// 检查两个图层名是否匹配。
fn is_layer_match(layer_name: &str, target_name: &str) -> bool {
    layer_name == target_name
        || layer_name.to_ascii_lowercase().contains("collision")
        || target_name.contains(layer_name)
}

/// Generate collision tiles for a specific layer.
///
/// 为特定图层生成碰撞瓦片。
fn generate_tiles_for_layer(
    commands: &mut Commands,
    tiled_map_asset: &TiledMapAsset,
    layer: &tiled::Layer,
    parent_entity: Entity,
) {
    let Some(tile_layer) = layer.as_tile_layer() else {
        return;
    };

    let tile_size = tiled_map_asset.map.tile_width as f32;
    let tile_height = tiled_map_asset.map.tile_height as f32;

    // Compute the offset used to center the entire map.
    //
    // 计算地图居中的偏移量。
    let map_width = tiled_map_asset.map.width as f32 * tile_size;
    let map_height = tiled_map_asset.map.height as f32 * tile_height;
    let center_offset_x = -map_width / 2.0;
    let center_offset_y = -map_height / 2.0;

    tiled_map_asset.for_each_tile(
        &tile_layer,
        |layer_tile, _tile_data, tile_pos, _chunk_pos| {
            if layer_tile.get_tile().is_some() {
                // Calculate each tile's world-space position, accounting for the centering offset.
                //
                // 计算瓦片在世界坐标中的位置（考虑居中偏移）。
                let world_x = center_offset_x + (tile_pos.x as f32 * tile_size) + (tile_size / 2.0);
                let world_y =
                    center_offset_y + (tile_pos.y as f32 * tile_height) + (tile_height / 2.0);

                // Spawn the collision tile as a child entity.
                //
                // 作为子实体创建碰撞瓦片。
                commands.entity(parent_entity).with_children(|parent| {
                    parent.spawn((
                        TilemapCollider,
                        Rect2DCollider::new(Vec2::new(tile_size, tile_height), Vec2::ZERO),
                        Transform::from_xyz(world_x, world_y, 0.0),
                        Name::new(format!("CollisionTile({},{})", tile_pos.x, tile_pos.y)),
                    ));
                });
            }
        },
    );
}

/// Generate collision objects for objects with collision properties.
///
/// 为具有碰撞属性的对象生成碰撞体。
fn generate_object_colliders(
    commands: &mut Commands,
    tiled_map_asset: &TiledMapAsset,
    parent_entity: Entity,
) {
    // Calculate map center offset (same as tile collision system)
    //
    // 计算地图中心偏移（与瓦片碰撞系统相同）
    let tile_size = tiled_map_asset.map.tile_width as f32;
    let tile_height = tiled_map_asset.map.tile_height as f32;
    let map_width = tiled_map_asset.map.width as f32 * tile_size;
    let map_height = tiled_map_asset.map.height as f32 * tile_height;
    let center_offset_x = -map_width / 2.0;
    let center_offset_y = -map_height / 2.0;

    for layer in tiled_map_asset.map.layers() {
        if let Some(object_layer) = layer.as_object_layer() {
            info!(
                "Processing object layer '{}' with {} objects",
                layer.name,
                object_layer.objects().count()
            );

            for object_data in object_layer.objects() {
                // Check if this object has collision property set to true
                //
                // 检查此对象是否将碰撞属性设置为 true
                if let Some(collision_value) = object_data.properties.get("collision")
                    && let tiled::PropertyValue::BoolValue(true) = collision_value
                    && let tiled::ObjectShape::Rect { width, height } = object_data.shape
                {
                    // Calculate world position (same coordinate system as tilemap)
                    // Tiled uses top-left origin, convert to center-based
                    //
                    // 计算世界位置（与瓦片地图坐标系相同）
                    // Tiled 使用左上角原点，转换为基于中心
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

                    // Spawn object colliders as child entities.
                    //
                    // 作为子实体创建对象碰撞体。
                    commands.entity(parent_entity).with_children(|parent| {
                        parent.spawn((
                            crate::app_state::overworld::tilemap::ObjectCollider,
                            TilemapCollider,
                            Rect2DCollider::new(size, Vec2::ZERO),
                            Transform::from_xyz(world_x, world_y, 0.0),
                            Visibility::Hidden,
                            Name::new(format!("ObjectCollision_{}", object_data.name)),
                        ));
                    });
                }
            }
        }
    }
}

type ObjectsQuery<'w, 's> = Query<
    'w,
    's,
    &'static mut Transform,
    (
        With<TiledObject>,
        Without<character::components::PlayerControlled>,
    ),
>;
type PlayerQuery<'w, 's> = Query<
    'w,
    's,
    (
        &'static Transform,
        &'static SpriteAnimationClip,
        &'static Sprite,
    ),
    (
        With<character::components::PlayerControlled>,
        Without<TiledObject>,
    ),
>;
type ObjectLayersQuery<'w, 's> = Query<
    'w,
    's,
    &'static Transform,
    (
        With<TiledLayer>,
        Without<TiledObject>,
        Without<character::components::PlayerControlled>,
    ),
>;

/// Relatively adjust the z-axis position of the entities in all object layers
/// based on the bottom y-coordinate of the Player's image
///
/// 根据 Player 的图像底部 y 坐标相对调整所有对象图层中实体的 z 轴位置
pub fn update_objects_order_with_player_system(
    mut objects: ObjectsQuery,
    player: PlayerQuery,
    object_layers: ObjectLayersQuery,
) {
    let (player_transform, player_anim, current_sprite) = if let Ok(result) = player.single() {
        result
    } else {
        return;
    };

    let player_y = if let Some(rect) = current_sprite.rect {
        player_transform.translation.y - rect.height() / 2.0
    } else {
        let anim_sprite = player_anim.get_current_sprite();
        if let Some(rect) = anim_sprite.rect {
            player_transform.translation.y - rect.height() / 2.0
        } else {
            player_transform.translation.y - 16.0
        }
    };

    let object_layer_z = object_layers
        .iter()
        .map(|transform| transform.translation.z)
        .max_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
        .unwrap_or(-2.0);

    // Update all objects with TiledObject component (from all object layers)
    // This automatically handles objects from any object layer, regardless of layer name
    //
    // 更新具有 TiledObject 组件的所有对象（来自所有对象图层）
    // 这会自动处理来自任何对象图层的对象，无论图层名称如何
    for mut transform in objects.iter_mut() {
        if transform.translation.y > player_y {
            transform.translation.z = -1.0 - object_layer_z;
        } else {
            transform.translation.z = 1.0 - object_layer_z;
        }
    }
}

/// Setup camera bounds based on tilemap size after the tilemap loads.
///
/// 在 tilemap 加载后根据地图大小设置摄像机边界。
pub fn setup_camera_bounds_system(
    mut followable_cameras: Query<&mut Followable, With<Camera>>,
    tiled_map_assets: Res<Assets<TiledMapAsset>>,
    tiled_maps_query: Query<&TiledMap>,
    windows: Query<&Window>,
    cameras: Query<&Transform, (With<Camera>, Without<Followable>)>,
) {
    // Only proceed if a tilemap asset is loaded.
    //
    // 只有在已加载 tilemap 时才继续。
    let Ok(tiled_map_handle) = tiled_maps_query.single() else {
        return;
    };

    let Some(tiled_map_asset) = tiled_map_assets.get(&tiled_map_handle.0) else {
        return;
    };

    // Calculate the map bounds.
    //
    // 计算地图边界。
    let tile_width = tiled_map_asset.map.tile_width as f32;
    let tile_height = tiled_map_asset.map.tile_height as f32;
    let map_width = tiled_map_asset.map.width as f32 * tile_width;
    let map_height = tiled_map_asset.map.height as f32 * tile_height;

    // Get the viewport size from the window and camera.
    //
    // 从窗口和摄像机获取视口大小。
    let viewport_width = if let Ok(window) = windows.single() {
        // Get the actual in-game viewport size considering resolution scaling.
        //
        // 获取实际游戏世界视口大小（考虑分辨率缩放）。
        if let Ok(camera_transform) = cameras.single() {
            // Camera scale affects the viewport size.
            //
            // 摄像机缩放影响视口大小。
            let scale = camera_transform.scale.x;
            window.resolution.width() * scale
        } else {
            320.0 // fallback to default
        }
    } else {
        320.0 // fallback to default
    };

    let viewport_height = if let Ok(window) = windows.single() {
        if let Ok(camera_transform) = cameras.single() {
            let scale = camera_transform.scale.y;
            window.resolution.height() * scale
        } else {
            240.0 // fallback to default
        }
    } else {
        240.0 // fallback to default
    };

    // Calculate bounds so the camera center stays inside the map minus half the viewport.
    //
    // 计算边界，确保摄像机中心始终位于地图范围内减去半个视口的位置。
    let half_viewport_width = viewport_width / 2.0;
    let half_viewport_height = viewport_height / 2.0;

    let min_x = (-map_width / 2.0) + half_viewport_width;
    let max_x = (map_width / 2.0) - half_viewport_width;
    let min_y = (-map_height / 2.0) + half_viewport_height;
    let max_y = (map_height / 2.0) - half_viewport_height;

    // Apply bounds to every followable camera that does not already have them enabled.
    //
    // 为所有尚未启用边界的可跟随摄像机应用边界。
    for mut followable in followable_cameras.iter_mut() {
        if !followable.bounds_enabled {
            followable.enable_bounds(min_x, max_x, min_y, max_y);
            info!(
                "Enabled camera bounds: X({:.1}, {:.1}), Y({:.1}, {:.1}) for viewport {}x{} on map {}x{}",
                min_x, max_x, min_y, max_y, viewport_width, viewport_height, map_width, map_height
            );
        }
    }
}

/// Update background music based on map properties.
///
/// 根据地图属性更新背景音乐。
pub fn update_map_bgm_system(
    mut current_bgm: ResMut<super::CurrentMapBgm>,
    mut bgm_handle: ResMut<super::CurrentBgmHandle>,
    tiled_maps: Query<&TiledMap>,
    tiled_map_assets: Res<Assets<TiledMapAsset>>,
    audio: Res<bevy_kira_audio::Audio>,
    mut audio_instances: ResMut<Assets<bevy_kira_audio::AudioInstance>>,
    asset_server: Res<AssetServer>,
) {
    for tiled_map in tiled_maps.iter() {
        if let Some(map_asset) = tiled_map_assets.get(&tiled_map.0)
            && let Some(bgm_prop) = map_asset.map.properties.get("bgm")
            && let tiled::PropertyValue::StringValue(bgm_path) = bgm_prop
            && current_bgm.0.as_deref() != Some(bgm_path)
        {
            if let Some(handle) = &bgm_handle.0
                && let Some(instance) = audio_instances.get_mut(handle)
            {
                instance.stop(bevy_kira_audio::AudioTween::default());
            }

            info!("Switching BGM to: {}", bgm_path);
            let handle = crate::core::audio::play_bgm(&audio, &asset_server, bgm_path);
            current_bgm.0 = Some(bgm_path.clone());
            bgm_handle.0 = Some(handle);
        }
    }
}
