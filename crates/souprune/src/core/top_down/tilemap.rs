//! # tilemap.rs
//!
//! # tilemap.rs 文件
//!
//! ## Module Overview
//!
//! ## 模块概述
//!
//! This module manages the top_down tilemap setup, initialization, and render ordering.
//!
//! 该模块管理 Overworld 瓦片地图的设置、初始化以及渲染顺序。
//!
//! ## Source File Overview
//!
//! ## 源文件概述
//!
//! The file implements `TilemapPlugin`, wiring systems for tiled map handling and object ordering relative to the player.
//!
//! 本文件实现了 `TilemapPlugin`，连接用于处理瓦片地图与相对玩家更新对象排序的系统。

use bevy::prelude::*;

#[cfg(feature = "experimental")]
pub mod beat;
pub mod object_properties;
#[cfg(feature = "experimental")]
pub mod reveal;
pub mod systems;

use crate::core::top_down::TopDownUpdate;
pub use object_properties::ObjectCollider;

#[derive(Resource, Default)]
pub struct CurrentMapBgm(pub Option<String>);

// ============================================================================
// BGM Handle Resource
// ============================================================================
#[derive(Resource, Default)]
pub struct CurrentBgmHandle(pub Option<Handle<bevy_kira_audio::AudioInstance>>);

pub(crate) struct TilemapPlugin;

impl Plugin for TilemapPlugin {
    fn build(&self, app: &mut App) {
        let schedule = crate::game_schedule(app);
        use object_properties::process_map_object_properties_system;
        use systems::*;
        // Tilemap is loaded by the sequencer's LoadMap chapter, not by hardcoded OnEnter systems.
        // 瓦片地图由序列器的 LoadMap 章节加载，而非硬编码的 OnEnter 系统。
        app.init_resource::<CurrentMapBgm>()
            .init_resource::<CurrentBgmHandle>()
            .add_systems(
                schedule,
                (
                    initialize_tilemap_system,
                    generate_collision_tiles_system,
                    process_map_object_properties_system,
                    update_camera_bounds_system,
                    update_objects_order_with_player_system,
                    update_map_bgm_system,
                )
                    .in_set(TopDownUpdate),
            );

        // Add tile reveal effect plugin when experimental feature is enabled
        // 当启用 experimental feature 时添加瓦片揭示效果插件
        #[cfg(feature = "experimental")]
        {
            app.add_plugins(reveal::TileRevealPlugin);
            app.add_plugins(beat::BeatPlugin);
        }
    }
}
