//! # setup.rs
//!
//! # setup.rs 文件
//!
//! ## Module Overview
//!
//! ## 模块概述
//!
//! Prepares the data and proxy entities required by the overworld tile-reveal effect.
//! It swaps the tilemap to a black-and-white material, caches the textures that will be needed
//! later, hides the original tiles, and builds one animated proxy entity per visible tile.
//!
//! 负责准备大地图揭露效果所需的数据和代理实体。它会把 tilemap 切到黑白材质，
//! 缓存后续动画需要的纹理，先隐藏原始格子，再为每个可见格子创建一个可动画化的代理实体。

use super::{
    BlackWhiteTilemapMaterial, FADE_INITIAL, HasBlackWhiteMaterial, RevealTilesRoot,
    RevealedTileSprite, RippleDirection, TileRevealState, TilemapTextureCache,
};
use bevy::prelude::*;
use bevy_ecs_tiled::prelude::*;
use std::collections::HashMap;

/// Apply black/white material to all TiledTilemap entities.
pub(super) fn apply_black_white_tilemap_material_system(
    mut commands: Commands,
    mut materials: ResMut<Assets<BlackWhiteTilemapMaterial>>,
    tilemaps_query: Query<Entity, (With<TiledTilemap>, Without<HasBlackWhiteMaterial>)>,
) {
    for entity in tilemaps_query.iter() {
        let material_handle = materials.add(BlackWhiteTilemapMaterial {});
        commands
            .entity(entity)
            .remove::<MaterialTilemapHandle<StandardTilemapMaterial>>()
            .insert((
                MaterialTilemapHandle(material_handle),
                HasBlackWhiteMaterial,
            ));
    }
}

/// Cache tilemap textures from TiledTilemap entities.
pub(super) fn cache_tilemap_textures_system(
    mut texture_cache: ResMut<TilemapTextureCache>,
    tilemaps_query: Query<(&TilemapTexture, &TilemapTileSize), With<TiledTilemap>>,
) {
    if !texture_cache.textures.is_empty() {
        return;
    }

    for (tilemap_texture, tile_size) in tilemaps_query.iter() {
        texture_cache.tile_size = Vec2::new(tile_size.x, tile_size.y);
        cache_texture(tilemap_texture, &mut texture_cache.textures);
    }
}

/// Initialize TileColor for all tiles to white (fade = 1.0).
pub(super) fn initialize_tile_colors_system(
    mut commands: Commands,
    tiles_query: Query<Entity, (With<TiledTile>, Without<TileColor>)>,
) {
    for entity in tiles_query.iter() {
        commands
            .entity(entity)
            .insert(TileColor(Color::srgba(FADE_INITIAL, 1.0, 1.0, 1.0)));
    }
}

/// Hide all original tiles by setting TileVisible to false.
pub(super) fn hide_all_original_tiles_system(
    mut reveal_state: ResMut<TileRevealState>,
    mut tiles_query: Query<&mut TileVisible, With<TiledTile>>,
) {
    if reveal_state.tiles_hidden {
        return;
    }

    let mut hidden_count = 0;
    for mut tile_visible in tiles_query.iter_mut() {
        tile_visible.0 = false;
        hidden_count += 1;
    }

    if hidden_count > 0 {
        reveal_state.tiles_hidden = true;
        info!("Hidden {} original tiles", hidden_count);
    }
}

/// Create sprite proxies for each tile to enable individual animations.
pub(super) fn create_tile_sprites_system(
    mut commands: Commands,
    mut reveal_state: ResMut<TileRevealState>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut color_materials: ResMut<Assets<ColorMaterial>>,
    texture_cache: Res<TilemapTextureCache>,
    tiled_maps_query: Query<&TiledMap>,
    tiled_map_assets: Res<Assets<TiledMapAsset>>,
    player_behavior: Option<Res<crate::core::overworld::player::config::PlayerBehavior>>,
    existing_sprites: Query<Entity, With<RevealedTileSprite>>,
) {
    if reveal_state.initialized || !existing_sprites.is_empty() {
        return;
    }

    if texture_cache.textures.is_empty() || !reveal_state.tiles_hidden {
        return;
    }

    let Ok(tiled_map_handle) = tiled_maps_query.single() else {
        return;
    };
    let Some(tiled_map_asset) = tiled_map_assets.get(&tiled_map_handle.0) else {
        return;
    };

    let origin = player_behavior
        .map(|pb| pb.spawn_position)
        .unwrap_or(Vec2::ZERO);
    reveal_state.origin = origin;

    let tile_width = tiled_map_asset.map.tile_width as f32;
    let tile_height = tiled_map_asset.map.tile_height as f32;
    let map_width = tiled_map_asset.map.width;
    let map_height = tiled_map_asset.map.height;

    let center_offset_x = -(map_width as f32 * tile_width) / 2.0;
    let center_offset_y = -(map_height as f32 * tile_height) / 2.0;

    let origin_tile_x = ((origin.x - center_offset_x) / tile_width).floor() as i32;
    let origin_tile_y = ((origin.y - center_offset_y) / tile_height).floor() as i32;

    let mut max_distance: u32 = 0;
    let mut tile_positions: HashMap<(u32, u32), (Vec2, u32, RippleDirection)> = HashMap::new();

    for layer in tiled_map_asset.map.layers() {
        let Some(tile_layer) = layer.as_tile_layer() else {
            continue;
        };

        let layer_name_lower = layer.name.to_ascii_lowercase();
        if layer_name_lower.contains("prototype") || layer_name_lower.contains("collision") {
            continue;
        }

        tiled_map_asset.for_each_tile(
            &tile_layer,
            |layer_tile, _tile_data, tile_pos, _chunk_pos| {
                if layer_tile.get_tile().is_none() {
                    return;
                }

                let pos_key = (tile_pos.x, tile_pos.y);
                let world_x =
                    center_offset_x + (tile_pos.x as f32 * tile_width) + (tile_width / 2.0);
                let world_y =
                    center_offset_y + (tile_pos.y as f32 * tile_height) + (tile_height / 2.0);

                let dx = tile_pos.x as i32 - origin_tile_x;
                let dy = tile_pos.y as i32 - origin_tile_y;
                let distance = (dx.abs() + dy.abs()) as u32;
                max_distance = max_distance.max(distance);

                let direction = RippleDirection::from_delta(dx, dy);
                tile_positions.insert(pos_key, (Vec2::new(world_x, world_y), distance, direction));
            },
        );
    }

    let tile_mesh = meshes.add(Rectangle::new(tile_width + 0.5, tile_height + 0.5));
    let white_material = color_materials.add(ColorMaterial::from_color(Color::WHITE));

    let reveal_tiles_root = commands
        .spawn((
            Name::new("RevealTiles"),
            RevealTilesRoot,
            Transform::default(),
            Visibility::Inherited,
            crate::core::mode::ModeScoped("overworld".to_string()),
        ))
        .id();

    let mut tile_count = 0;

    for ((tile_x, tile_y), (world_pos, distance, direction)) in tile_positions.iter() {
        commands.spawn((
            Name::new(format!("RevealTile({},{})", tile_x, tile_y)),
            RevealedTileSprite {
                manhattan_distance: *distance,
                tile_pos: (*tile_x, *tile_y),
                direction: *direction,
            },
            Mesh2d(tile_mesh.clone()),
            MeshMaterial2d(white_material.clone()),
            Transform::from_xyz(world_pos.x, world_pos.y, -1.0).with_scale(Vec3::ZERO),
            Visibility::Inherited,
            ChildOf(reveal_tiles_root),
        ));
        tile_count += 1;
    }

    reveal_state.max_distance = max_distance;
    reveal_state.initialized = true;

    info!(
        "Tile reveal initialized: {} tiles, max distance: {}, origin: {:?}",
        tile_count, max_distance, origin
    );
}

/// Insert texture handles from a `TilemapTexture` into the cache, skipping duplicates.
fn cache_texture(tilemap_texture: &TilemapTexture, textures: &mut Vec<Handle<Image>>) {
    match tilemap_texture {
        TilemapTexture::Single(handle) => {
            if !textures.contains(handle) {
                textures.push(handle.clone());
            }
        }
        #[cfg(not(feature = "atlas"))]
        TilemapTexture::Vector(handles) => {
            for handle in handles {
                if !textures.contains(handle) {
                    textures.push(handle.clone());
                }
            }
        }
        #[cfg(not(feature = "atlas"))]
        TilemapTexture::TextureContainer(handle) => {
            if !textures.contains(handle) {
                textures.push(handle.clone());
            }
        }
    }
}
