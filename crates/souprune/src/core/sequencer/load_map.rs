//! # sequencer/load_map.rs
//!
//! ## Module Overview
//!
//! LoadMap chapter processing system.
//! Spawns a TiledMap entity from a .tmx file path.
//! Collision generation, object processing, and camera bounds
//! are handled reactively by existing tilemap systems.
//!
//! LoadMap 章节处理系统。
//! 从 .tmx 文件路径生成 TiledMap 实体。
//! 碰撞生成、对象处理和相机边界由现有的 tilemap 系统以响应式方式处理。

use super::chapter_schema::Chapter;
use super::context::*;
use bevy::prelude::*;
use bevy_ecs_tiled::prelude::{TiledMap, TiledMapLayerZOffset, TilemapAnchor};

/// System to process LoadMap chapters.
///
/// 处理 LoadMap 章节的系统。
pub fn process_load_map_system(
    mut commands: Commands,
    query: Query<(Entity, &ActiveChapter), (Without<WaitTimer>, Without<ChapterFinished>)>,
    asset_server: Res<AssetServer>,
    souprune_config: Option<Res<crate::config::SoupruneConfig>>,
) {
    let Some(souprune_config) = souprune_config else {
        return;
    };
    for (entity, active_chapter) in query.iter() {
        if let Chapter::LoadMap {
            path,
            generate_collision: _,
            process_objects: _,
            setup_camera_bounds: _,
        } = &active_chapter.chapter
        {
            info!("[Sequencer] LoadMap: loading map from '{}'", path);

            let map_handle = asset_server.load(path.clone());
            commands.spawn((
                TiledMap(map_handle),
                TilemapAnchor::Center,
                TiledMapLayerZOffset(souprune_config.render.z_layer_tilemap),
            ));

            // Map loading is async. The existing tilemap systems
            // (initialize_tilemap, generate_collision_tiles, setup_camera_bounds, etc.)
            // react to Added<TiledLayer> and Added<TiledObject> queries,
            // so they will process the map automatically.
            commands.entity(entity).insert(ChapterFinished);
        }
    }
}
