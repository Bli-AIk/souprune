//! # reveal.rs
//!
//! # reveal.rs 文件
//!
//! ## Module Overview
//!
//! ## 模块概述
//!
//! This module implements a tile reveal effect for the tilemap.
//! Tiles are spawned as sprites that animate from small to large, creating a ripple effect
//! from a center point. All tiles are rendered in pure black and white using a shader.
//! After each tile's animation completes, the sprite is hidden and the original tile is shown.
//! The original tilemap also uses a black/white shader.
//!
//! 本模块实现了瓦片地图的揭示效果。
//! 瓦片作为精灵生成，从小到大播放动画，从中心点向外产生涟漪效果。
//! 所有瓦片都通过着色器以纯黑白渲染。
//! 每个瓦片动画完成后，隐藏精灵并显示原始瓦片。
//! 原始瓦片地图也使用黑白着色器。
//!
//! ## Experimental Feature
//!
//! ## 实验功能
//!
//! This module only runs when the `experimental` feature is enabled.
//!
//! 本模块仅在启用 `experimental` feature 时运行。

use bevy::prelude::*;
use bevy::reflect::TypePath;
use bevy::render::render_resource::AsBindGroup;
use bevy::shader::ShaderRef;
use bevy::sprite_render::{AlphaMode2d, Material2d, Material2dPlugin};
use bevy_ecs_tiled::prelude::*;
use bevy_tween::interpolate::Interpolator;
use bevy_tween::prelude::*;
use std::collections::HashMap;
use std::time::Duration;

// ========== BLACK/WHITE THRESHOLD ==========
// This constant is used for BOTH the reveal sprite shader AND the tilemap shader.
// Lower value = more white, higher value = more black.
// Pixels with luminance ABOVE this become WHITE, BELOW become BLACK.
// 此常量用于揭示精灵着色器和瓦片地图着色器。
// 较低的值 = 更多白色，较高的值 = 更多黑色。
// 亮度高于此值的像素变为白色，低于的变为黑色。
pub const BLACK_WHITE_THRESHOLD: f32 = 0.1; // <-- ADJUST THIS VALUE / 调整此值

/// Marker component for revealed tile sprites.
///
/// 已揭示的瓦片精灵的标记组件。
#[derive(Component)]
pub struct RevealedTileSprite {
    /// Manhattan distance from the reveal origin.
    ///
    /// 距离揭示原点的曼哈顿距离。
    pub manhattan_distance: u32,
    /// Tile position (x, y) for matching with original tiles.
    ///
    /// 瓦片位置 (x, y)，用于与原始瓦片匹配。
    pub tile_pos: (u32, u32),
}

/// Marker component for tiles that are currently animating.
///
/// 当前正在动画的瓦片的标记组件。
#[derive(Component)]
pub struct AnimatingTile {
    /// Timer to track when animation is complete.
    ///
    /// 跟踪动画完成的计时器。
    pub animation_timer: Timer,
}

/// Marker component for tiles that have finished animating.
///
/// 已完成动画的瓦片的标记组件。
#[derive(Component)]
pub struct AnimationComplete;

/// Resource to track the reveal animation state.
///
/// 跟踪揭示动画状态的资源。
#[derive(Resource)]
pub struct TileRevealState {
    /// Origin position for the ripple effect (world coordinates).
    ///
    /// 涟漪效果的原点位置（世界坐标）。
    pub origin: Vec2,

    /// Current step of the reveal (Manhattan distance threshold).
    ///
    /// 当前揭示步骤（曼哈顿距离阈值）。
    pub current_step: u32,

    /// Maximum Manhattan distance to reveal.
    ///
    /// 要揭示的最大曼哈顿距离。
    pub max_distance: u32,

    /// Whether the reveal animation triggering is complete.
    ///
    /// 揭示动画触发是否完成。
    pub all_triggered: bool,

    /// Whether initialization is done.
    ///
    /// 初始化是否完成。
    pub initialized: bool,

    /// Whether all original tiles have been made visible initially.
    ///
    /// 是否已将所有原始瓦片初始设置为不可见。
    pub tiles_hidden: bool,
}

impl Default for TileRevealState {
    fn default() -> Self {
        Self {
            origin: Vec2::ZERO,
            current_step: 0,
            max_distance: 0,
            all_triggered: false,
            initialized: false,
            tiles_hidden: false,
        }
    }
}

/// Resource to store tilemap texture for black/white rendering.
///
/// 存储瓦片地图纹理以用于黑白渲染的资源。
#[derive(Resource, Default)]
pub struct TilemapTextureCache {
    pub textures: Vec<Handle<Image>>,
    pub tile_size: Vec2,
}

/// Black and white shader material for tiles.
/// Converts tile textures to pure black (#000000) and white (#FFFFFF).
///
/// 瓦片的黑白着色器材质。
/// 将瓦片纹理转换为纯黑色 (#000000) 和纯白色 (#FFFFFF)。
#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct BlackWhiteMaterial {
    // ========== BLACK/WHITE THRESHOLD ==========
    // Threshold for black/white conversion.
    // Lower value = more white, higher value = more black.
    // Pixels with luminance ABOVE this become WHITE, BELOW become BLACK.
    // 黑白转换的阈值。
    // 较低的值 = 更多白色，较高的值 = 更多黑色。
    // 亮度高于此值的像素变为白色，低于的变为黑色。
    #[uniform(0)]
    pub threshold: f32,

    /// Base texture.
    ///
    /// 基础纹理。
    #[texture(1)]
    #[sampler(2)]
    pub texture: Handle<Image>,

    /// UV rect for this tile in the atlas (x, y, width, height).
    ///
    /// 此瓦片在图集中的 UV 矩形 (x, y, width, height)。
    #[uniform(3)]
    pub uv_rect: Vec4,
}

impl Material2d for BlackWhiteMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/black_white_tile.wgsl".into()
    }

    fn alpha_mode(&self) -> AlphaMode2d {
        AlphaMode2d::Blend
    }
}

/// Black and white material for bevy_ecs_tilemap.
/// Uses a custom fragment shader to convert tilemap to pure black and white.
///
/// bevy_ecs_tilemap 的黑白材质。
/// 使用自定义片段着色器将瓦片地图转换为纯黑白。
#[derive(Asset, TypePath, AsBindGroup, Debug, Clone, Default)]
pub struct BlackWhiteTilemapMaterial {}

impl MaterialTilemap for BlackWhiteTilemapMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/black_white_tilemap.wgsl".into()
    }
}

/// Marker component for tilemaps that have been given the black/white material.
///
/// 已应用黑白材质的瓦片地图的标记组件。
#[derive(Component)]
pub struct HasBlackWhiteMaterial;

/// Plugin for the tile reveal effect.
///
/// 瓦片揭示效果的插件。
pub struct TileRevealPlugin;

impl Plugin for TileRevealPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(Material2dPlugin::<BlackWhiteMaterial>::default())
            .add_plugins(MaterialTilemapPlugin::<BlackWhiteTilemapMaterial>::default())
            .init_resource::<TileRevealState>()
            .init_resource::<TilemapTextureCache>()
            .add_tween_systems(bevy_tween::tween::component_tween_system::<ScaleInterpolator>())
            .add_systems(
                Update,
                (
                    apply_black_white_tilemap_material_system,
                    cache_tilemap_textures_system,
                    hide_all_original_tiles_system,
                    create_tile_sprites_system,
                    update_reveal_animation_system,
                    check_animation_complete_system,
                    cleanup_completed_sprites_system,
                )
                    .chain()
                    .in_set(super::super::OverworldUpdate),
            );
    }
}

/// Scale interpolator for tile reveal animation.
///
/// 瓦片揭示动画的缩放插值器。
#[derive(Debug, Clone)]
pub struct ScaleInterpolator {
    pub start: Vec3,
    pub end: Vec3,
}

impl Interpolator for ScaleInterpolator {
    type Item = Transform;

    fn interpolate(&self, item: &mut Self::Item, value: f32, _previous_value: f32) {
        item.scale = self.start.lerp(self.end, value);
    }
}

/// Apply black/white material to all TiledTilemap entities.
/// Remove the default StandardTilemapMaterial and add BlackWhiteTilemapMaterial.
///
/// 为所有 TiledTilemap 实体应用黑白材质。
/// 移除默认的 StandardTilemapMaterial 并添加 BlackWhiteTilemapMaterial。
fn apply_black_white_tilemap_material_system(
    mut commands: Commands,
    mut materials: ResMut<Assets<BlackWhiteTilemapMaterial>>,
    tilemaps_query: Query<Entity, (With<TiledTilemap>, Without<HasBlackWhiteMaterial>)>,
) {
    for entity in tilemaps_query.iter() {
        let material_handle = materials.add(BlackWhiteTilemapMaterial {});
        commands
            .entity(entity)
            // Remove the default StandardTilemapMaterial handle
            // 移除默认的 StandardTilemapMaterial 句柄
            .remove::<MaterialTilemapHandle<StandardTilemapMaterial>>()
            // Add our custom BlackWhiteTilemapMaterial handle
            // 添加我们的自定义 BlackWhiteTilemapMaterial 句柄
            .insert((
                MaterialTilemapHandle(material_handle),
                HasBlackWhiteMaterial,
            ));
    }
}

/// Cache tilemap textures from TiledTilemap entities.
///
/// 从 TiledTilemap 实体缓存瓦片地图纹理。
fn cache_tilemap_textures_system(
    mut texture_cache: ResMut<TilemapTextureCache>,
    tilemaps_query: Query<(&TilemapTexture, &TilemapTileSize), With<TiledTilemap>>,
) {
    if !texture_cache.textures.is_empty() {
        return;
    }

    for (tilemap_texture, tile_size) in tilemaps_query.iter() {
        texture_cache.tile_size = Vec2::new(tile_size.x, tile_size.y);

        match tilemap_texture {
            TilemapTexture::Single(handle) => {
                if !texture_cache.textures.contains(handle) {
                    texture_cache.textures.push(handle.clone());
                }
            }
            #[cfg(not(feature = "atlas"))]
            TilemapTexture::Vector(handles) => {
                for handle in handles {
                    if !texture_cache.textures.contains(handle) {
                        texture_cache.textures.push(handle.clone());
                    }
                }
            }
            #[cfg(not(feature = "atlas"))]
            TilemapTexture::TextureContainer(handle) => {
                if !texture_cache.textures.contains(handle) {
                    texture_cache.textures.push(handle.clone());
                }
            }
        }
    }
}

/// Hide all original tiles by setting TileVisible to false.
///
/// 通过将 TileVisible 设置为 false 来隐藏所有原始瓦片。
fn hide_all_original_tiles_system(
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

/// Stores information about a tile position and its texture.
///
/// 存储瓦片位置及其纹理的信息。
struct TileInfo {
    world_pos: Vec2,
    distance: u32,
    texture_handle: Handle<Image>,
    /// UV rect in normalized coordinates (0-1).
    uv_rect: Vec4,
}

/// Create sprite proxies for each tile to enable individual animations.
/// Uses actual tile textures with black/white material.
///
/// 为每个瓦片创建精灵代理以启用独立动画。
/// 使用实际的瓦片纹理和黑白材质。
fn create_tile_sprites_system(
    mut commands: Commands,
    mut reveal_state: ResMut<TileRevealState>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<BlackWhiteMaterial>>,
    texture_cache: Res<TilemapTextureCache>,
    tiled_maps_query: Query<&TiledMap>,
    tiled_map_assets: Res<Assets<TiledMapAsset>>,
    images: Res<Assets<Image>>,
    player_behavior: Option<Res<crate::app_state::overworld::player::config::PlayerBehavior>>,
    existing_sprites: Query<Entity, With<RevealedTileSprite>>,
) {
    // Skip if already initialized or sprites already exist
    if reveal_state.initialized || !existing_sprites.is_empty() {
        return;
    }

    // Wait for textures to be cached and tiles to be hidden
    if texture_cache.textures.is_empty() || !reveal_state.tiles_hidden {
        return;
    }

    let Ok(tiled_map_handle) = tiled_maps_query.single() else {
        return;
    };

    let Some(tiled_map_asset) = tiled_map_assets.get(&tiled_map_handle.0) else {
        return;
    };

    // Set origin from player spawn position
    // 从玩家生成位置设置原点
    let origin = player_behavior
        .map(|pb| pb.spawn_position)
        .unwrap_or(Vec2::ZERO);
    reveal_state.origin = origin;

    let tile_width = tiled_map_asset.map.tile_width as f32;
    let tile_height = tiled_map_asset.map.tile_height as f32;
    let map_width = tiled_map_asset.map.width;
    let map_height = tiled_map_asset.map.height;

    // Calculate center offset for map positioning
    // 计算地图定位的中心偏移
    let center_offset_x = -(map_width as f32 * tile_width) / 2.0;
    let center_offset_y = -(map_height as f32 * tile_height) / 2.0;

    // Origin tile position (converted from world to tile coordinates)
    // 原点瓦片位置（从世界坐标转换为瓦片坐标）
    let origin_tile_x = ((origin.x - center_offset_x) / tile_width).floor() as i32;
    let origin_tile_y = ((origin.y - center_offset_y) / tile_height).floor() as i32;

    let mut max_distance: u32 = 0;

    // Collect all visible tiles from all layers (ignoring layer, only using xy)
    // 从所有图层收集所有可见瓦片（忽略图层，只使用 xy）
    let mut tile_infos: HashMap<(u32, u32), TileInfo> = HashMap::new();

    // Get the first texture for use with all tiles (simplified approach)
    let Some(first_texture) = texture_cache.textures.first() else {
        return;
    };

    // Get image dimensions for UV calculation
    let image_size = images
        .get(first_texture)
        .map(|img| Vec2::new(img.width() as f32, img.height() as f32))
        .unwrap_or(Vec2::new(256.0, 256.0));

    let tiles_per_row = (image_size.x / tile_width).floor() as u32;

    for layer in tiled_map_asset.map.layers() {
        let Some(tile_layer) = layer.as_tile_layer() else {
            continue;
        };

        // Skip prototype and collision layers
        // 跳过原型和碰撞图层
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

                // Calculate Manhattan distance from origin
                // 计算距离原点的曼哈顿距离
                let distance = ((tile_pos.x as i32 - origin_tile_x).abs()
                    + (tile_pos.y as i32 - origin_tile_y).abs())
                    as u32;
                max_distance = max_distance.max(distance);

                // Calculate UV rect based on tile id
                // 根据瓦片 ID 计算 UV 矩形
                let tile_id = layer_tile.id();
                let tex_x = (tile_id % tiles_per_row) as f32 * tile_width;
                let tex_y = (tile_id / tiles_per_row) as f32 * tile_height;

                let uv_rect = Vec4::new(
                    tex_x / image_size.x,
                    tex_y / image_size.y,
                    tile_width / image_size.x,
                    tile_height / image_size.y,
                );

                tile_infos.insert(
                    pos_key,
                    TileInfo {
                        world_pos: Vec2::new(world_x, world_y),
                        distance,
                        texture_handle: first_texture.clone(),
                        uv_rect,
                    },
                );
            },
        );
    }

    // Create a shared mesh for all tiles
    // Add a small overlap (0.5 pixels) to prevent gaps between tiles
    // 为所有瓦片创建共享网格
    // 添加少量重叠（0.5 像素）以防止瓦片之间出现缝隙
    let tile_mesh = meshes.add(Rectangle::new(tile_width + 0.5, tile_height + 0.5));

    let mut tile_count = 0;

    // Now create sprites for each unique tile position
    // 现在为每个唯一的瓦片位置创建精灵
    for ((tile_x, tile_y), tile_info) in tile_infos.iter() {
        // Create black/white material for this tile
        // Uses BLACK_WHITE_THRESHOLD constant defined at the top of this file
        // 为此瓦片创建黑白材质
        // 使用此文件顶部定义的 BLACK_WHITE_THRESHOLD 常量
        let material = materials.add(BlackWhiteMaterial {
            threshold: BLACK_WHITE_THRESHOLD,
            texture: tile_info.texture_handle.clone(),
            uv_rect: tile_info.uv_rect,
        });

        // Spawn sprite with initial scale of 0 (invisible)
        // 生成初始缩放为 0 的精灵（不可见）
        commands.spawn((
            Name::new(format!("RevealTile({},{})", tile_x, tile_y)),
            RevealedTileSprite {
                manhattan_distance: tile_info.distance,
                tile_pos: (*tile_x, *tile_y),
            },
            Mesh2d(tile_mesh.clone()),
            MeshMaterial2d(material),
            Transform::from_xyz(tile_info.world_pos.x, tile_info.world_pos.y, -1.0)
                .with_scale(Vec3::ZERO),
            Visibility::Inherited,
            super::super::OverworldEntity(),
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

/// Update the reveal animation, triggering scale tweens for tiles at the current distance.
/// Now synchronized with eighth notes from the beat system.
///
/// 更新揭示动画，为当前距离的瓦片触发缩放补间。
/// 现在与节拍系统的八分音符同步。
fn update_reveal_animation_system(
    mut commands: Commands,
    mut reveal_state: ResMut<TileRevealState>,
    mut beat_events: MessageReader<super::beat::BeatEvent>,
    tiles_query: Query<
        (Entity, &RevealedTileSprite),
        (Without<AnimatingTile>, Without<AnimationComplete>),
    >,
) {
    if !reveal_state.initialized || reveal_state.all_triggered {
        // Clear events even if not processing to prevent buildup
        beat_events.clear();
        return;
    }

    // Check for eighth note events to trigger tile reveal
    // 检查八分音符事件以触发瓦片揭示
    let mut should_step = false;
    for event in beat_events.read() {
        if matches!(event, super::beat::BeatEvent::EighthNote) {
            should_step = true;
            break;
        }
    }

    if should_step {
        // Animate all tiles at the current step distance
        // 为当前步骤距离的所有瓦片添加动画
        for (entity, tile) in tiles_query.iter() {
            if tile.manhattan_distance == reveal_state.current_step {
                // ========== ANIMATION DURATION ==========
                // Duration of each tile's scale animation (in milliseconds).
                // 每个瓦片缩放动画的持续时间（毫秒）。
                let animation_duration_ms = 500; // <-- ADJUST THIS VALUE TO CHANGE ANIMATION SPEED / 调整此值以更改动画速度
                let animation_duration = Duration::from_millis(animation_duration_ms);

                commands.entity(entity).insert(AnimatingTile {
                    animation_timer: Timer::from_seconds(
                        animation_duration_ms as f32 / 1000.0,
                        TimerMode::Once,
                    ),
                });
                commands.entity(entity).animation().insert_tween_here(
                    animation_duration,
                    // ========== ANIMATION EASING ==========
                    // Easing function for the scale animation.
                    // 缩放动画的缓动函数。
                    EaseKind::BackOut,
                    entity.into_target().with(ScaleInterpolator {
                        start: Vec3::ZERO,
                        end: Vec3::ONE,
                    }),
                );
            }
        }

        reveal_state.current_step += 1;

        if reveal_state.current_step > reveal_state.max_distance {
            reveal_state.all_triggered = true;
            info!("All tile reveal animations triggered");
        }
    }
}

/// Check if animating tiles have completed their animation.
///
/// 检查动画中的瓦片是否已完成动画。
fn check_animation_complete_system(
    mut commands: Commands,
    time: Res<Time>,
    mut animating_tiles: Query<(Entity, &mut AnimatingTile), Without<AnimationComplete>>,
) {
    for (entity, mut animating) in animating_tiles.iter_mut() {
        animating.animation_timer.tick(time.delta());
        if animating.animation_timer.is_finished() {
            commands.entity(entity).insert(AnimationComplete);
        }
    }
}

/// Cleanup completed sprite animations: hide sprite, show original tile.
///
/// 清理已完成的精灵动画：隐藏精灵，显示原始瓦片。
fn cleanup_completed_sprites_system(
    mut commands: Commands,
    completed_sprites: Query<(Entity, &RevealedTileSprite), With<AnimationComplete>>,
    mut tiles_query: Query<(&TilePos, &mut TileVisible), With<TiledTile>>,
) {
    for (sprite_entity, reveal_sprite) in completed_sprites.iter() {
        // Find and show the original tile(s) at this position
        // 找到并显示此位置的原始瓦片
        for (tile_pos, mut tile_visible) in tiles_query.iter_mut() {
            if tile_pos.x == reveal_sprite.tile_pos.0 && tile_pos.y == reveal_sprite.tile_pos.1 {
                tile_visible.0 = true;
            }
        }

        // Despawn the sprite
        // 销毁精灵
        commands.entity(sprite_entity).despawn();
    }
}
