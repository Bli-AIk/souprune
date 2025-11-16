//! # tilemap.rs
//!
//! ## Module Overview
//! This module manages the tilemap and its objects within the overworld game state, including setup, initialization, and rendering order.
//!
//! ## Source File Overview
//! This file defines the `TilemapPlugin`, which integrates systems for handling tiled maps, setting up the tilemap,
//! and updating object rendering order relative to the player.
//!
//! ## 模块概述
//! 该模块管理着 Overworld 游戏状态中的瓦片地图及其对象，包括设置、初始化和渲染顺序。
//!
//! ## 源文件概述
//! 该文件定义了 `TilemapPlugin`，它集成了用于处理瓦片地图、设置瓦片地图，
//! 以及更新相对于玩家的对象渲染顺序的系统。

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
