use crate::info;
use bevy::prelude::Commands;
use bevy::prelude::*;
use bevy_ecs_tiled::prelude::*;

pub(crate) fn setup_tilemap(commands: &mut Commands, asset_server: &Res<AssetServer>) {
    commands.spawn((
        TiledMap(asset_server.load("levels/ruins/ruins_3.tmx")),
        TilemapAnchor::Center,
        TiledMapLayerZOffset(10.0),
    ));

    // TODO: Tilemap的资源加载（或许）应当在AppSetup阶段完成。
}

pub(crate) fn filter_prototype_layers_and_set_z_order_system(
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
