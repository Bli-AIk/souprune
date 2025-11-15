use crate::app_state::overworld::character;
use crate::core::animation::components::SpriteAnimationClip;
use crate::core::collision::Rect2DCollider;
use bevy::asset::{AssetServer, Assets};
use bevy::log::info;
use bevy::prelude::{
    Added, Commands, Component, Entity, Name, Query, Res, Sprite, Transform, Vec2, Visibility,
    With, Without,
};
use bevy_ecs_tiled::prelude::{
    TiledLayer, TiledMap, TiledMapAsset, TiledMapLayerZOffset, TiledObject, TilemapAnchor, tiled,
};

/// Marker component for tilemap collision entities
/// 瓦片地图碰撞实体的标记组件
#[derive(Component)]
pub struct TilemapCollider;

pub fn setup_tilemap_system(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((
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
) {
    for (layer_entity, layer_name, _tiled_layer) in collision_layers.iter() {
        if !is_collision_layer(layer_name.as_str()) {
            continue;
        }

        if let Some((tiled_map_asset, matching_layer)) =
            find_matching_layer(&tiled_maps_query, &tiled_map_assets, layer_name.as_str())
        {
            generate_tiles_for_layer(&mut commands, tiled_map_asset, &matching_layer);

            // Also process object collision while we have the map data
            // 在处理瓦片碰撞的同时处理对象碰撞
            if existing_object_colliders.is_empty() {
                generate_object_colliders(&mut commands, tiled_map_asset);
            }
        }

        commands.entity(layer_entity).insert(TilemapCollider);
    }
}

/// Check if a layer name indicates it's a collision layer
/// 检查图层名是否表示它是碰撞图层
fn is_collision_layer(layer_name: &str) -> bool {
    layer_name.to_ascii_lowercase().contains("collision")
}

/// Find the matching layer in the tiled map assets
/// 在瓦片地图资源中查找匹配的图层
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

/// Check if two layer names match
/// 检查两个图层名是否匹配
fn is_layer_match(layer_name: &str, target_name: &str) -> bool {
    layer_name == target_name
        || layer_name.to_ascii_lowercase().contains("collision")
        || target_name.contains(layer_name)
}

/// Generate collision tiles for a specific layer
/// 为特定图层生成碰撞瓦片
fn generate_tiles_for_layer(
    commands: &mut Commands,
    tiled_map_asset: &TiledMapAsset,
    layer: &tiled::Layer,
) {
    let Some(tile_layer) = layer.as_tile_layer() else {
        return;
    };

    let tile_size = tiled_map_asset.map.tile_width as f32;
    let tile_height = tiled_map_asset.map.tile_height as f32;

    // 计算地图居中的偏移量
    let map_width = tiled_map_asset.map.width as f32 * tile_size;
    let map_height = tiled_map_asset.map.height as f32 * tile_height;
    let center_offset_x = -map_width / 2.0;
    let center_offset_y = -map_height / 2.0;

    tiled_map_asset.for_each_tile(
        &tile_layer,
        |layer_tile, _tile_data, tile_pos, _chunk_pos| {
            if layer_tile.get_tile().is_some() {
                // 计算瓦片在世界坐标中的位置（考虑居中偏移）
                let world_x = center_offset_x + (tile_pos.x as f32 * tile_size) + (tile_size / 2.0);
                let world_y =
                    center_offset_y + (tile_pos.y as f32 * tile_height) + (tile_height / 2.0);

                // 创建碰撞瓦片实体
                commands.spawn((
                    TilemapCollider,
                    Rect2DCollider::new(Vec2::new(tile_size, tile_height), Vec2::ZERO),
                    Transform::from_xyz(world_x, world_y, 0.0),
                    Name::new(format!("CollisionTile({},{})", tile_pos.x, tile_pos.y)),
                ));
            }
        },
    );
}

/// Generate collision objects for objects with collision properties
/// 为具有碰撞属性的对象生成碰撞体
fn generate_object_colliders(commands: &mut Commands, tiled_map_asset: &TiledMapAsset) {
    // Calculate map center offset (same as tile collision system)
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
                if let Some(collision_value) = object_data.properties.get("collision") {
                    if let tiled::PropertyValue::BoolValue(true) = collision_value {
                        if let tiled::ObjectShape::Rect { width, height } = object_data.shape {
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
                                crate::app_state::overworld::tilemap::ObjectCollider,
                                TilemapCollider,
                                Rect2DCollider::new(size, Vec2::ZERO),
                                Transform::from_xyz(world_x, world_y, 0.0),
                                Visibility::Hidden,
                                Name::new(format!("ObjectCollision_{}", object_data.name)),
                            ));
                        }
                    }
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
    for mut transform in objects.iter_mut() {
        if transform.translation.y > player_y {
            transform.translation.z = -1.0 - object_layer_z;
        } else {
            transform.translation.z = 1.0 - object_layer_z;
        }
    }
}
