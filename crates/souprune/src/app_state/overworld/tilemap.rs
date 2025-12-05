//! # tilemap.rs
//!
//! # tilemap.rs 文件
//!
//! ## Module Overview
//!
//! ## 模块概述
//!
//! This module manages the overworld tilemap setup, initialization, and render ordering.
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

use crate::app_state::AppState::Overworld;
use bevy::prelude::*;

pub mod object_properties;
pub mod systems;

pub use object_properties::ObjectCollider;

pub(crate) struct TilemapPlugin;

impl Plugin for TilemapPlugin {
    fn build(&self, app: &mut App) {
        use systems::*;
        app.add_systems(OnEnter(Overworld), setup_tilemap_system)
            .add_systems(
                Update,
                (
                    initialize_tilemap_system,
                    generate_collision_tiles_system,
                    setup_camera_bounds_system,
                    update_objects_order_with_player_system,
                ),
            );
    }
}
