use bevy::prelude::{Added, Commands, Entity, Name, Query, Res, Sprite, Transform, Visibility, With, Without};
use bevy_ecs_tiled::prelude::{TiledLayer, TiledMap, TiledMapLayerZOffset, TiledObject, TilemapAnchor};
use bevy::log::info;
use bevy::asset::AssetServer;
use crate::app_state::overworld::character;
use crate::core::animation::components::SpriteAnimationClip;

// TODO: 添加碰撞系统
pub fn setup_tilemap_system(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((
        TiledMap(asset_server.load("levels/ruins/ruins_3.tmx")),
        TilemapAnchor::Center,
        TiledMapLayerZOffset(10.0),
    ));
    // TODO: Tilemap的资源加载（或许）应当在 AppSetup 阶段完成。
}

/// Initialize Tilemap layers, filter and hide layers with "prototype" in their names,
///
/// 初始化 Tilemap 图层，过滤并隐藏包含 "prototype" 的图层名称，并根据图层顺序设置其他图层的 z 轴位置
pub fn initialize_tilemap_system(
    mut commands: Commands,
    layers_query: Query<(Entity, &Name), Added<TiledLayer>>,
) {
    let mut layers: Vec<_> = layers_query.iter().collect();

    layers.sort_by_key(|(entity, _)| entity.index());

    for (index, (layer_entity, layer_name)) in layers.iter().enumerate() {
        let layer_name_str = layer_name.as_str();

        if layer_name_str.to_lowercase().contains("prototype") {
            info!("Hide prototype layer: {}", layer_name_str);
            commands.entity(*layer_entity).insert(Visibility::Hidden);
        } else {
            info!("Show layers: {}", layer_name_str);

            let z_offset = -1.0 - (layers.len() as f32 - 1.0 - index as f32) * 0.5;

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

/// Relatively adjust the z-axis position of the entities in the objects layer
/// based on the bottom y-coordinate of the Player's image
///
/// 根据 Player 的 图像底部 y 坐标相对调整 objects 图层中实体的 z 轴位置
pub fn update_objects_order_with_player_system(mut objects: ObjectsQuery, player: PlayerQuery) {
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

    for mut transform in objects.iter_mut() {
        if transform.translation.y > player_y {
            transform.translation.z = -1.0;
        } else {
            transform.translation.z = 1.0;
        }
    }
}