use crate::debug_info;
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
    for (layer_entity, layer_name) in layers_query.iter() {
        let layer_name_str = layer_name.as_str();

        if layer_name_str.to_lowercase().contains("prototype") {
            debug_info!("Hide prototype layer: {}", layer_name_str);
            commands.entity(layer_entity).insert(Visibility::Hidden);
        } else {
            debug_info!("Show layers: {}", layer_name_str);

            let z_offset = match layer_name_str {
                name if name.contains("floor") => -2.5,
                name if name.contains("wall") && !name.contains("on_wall") => -2.0,
                name if name.contains("on_wall") => -1.5,
                name if name.contains("objects") => -1.0,
                _ => -3.0,
            };

            commands
                .entity(layer_entity)
                .insert(Transform::from_xyz(0.0, 0.0, z_offset));
            debug_info!(
                "Set the Z-axis position of layer {}: {}",
                layer_name_str,
                z_offset
            );
        }
    }
}
