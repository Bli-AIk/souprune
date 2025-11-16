//! # collision.rs
//!
//! ## Module Overview
//! This module provides collision detection components and systems.
//!
//! ## 模块概述
//! 该模块提供碰撞检测组件和系统。

pub(crate) mod components;
pub(crate) mod systems;

pub use components::*;

use bevy::app::*;
use bevy::prelude::*;

/// Collision plugin for managing collision detection components and systems.
///
/// 碰撞插件，用于管理碰撞检测组件和系统。
pub(crate) struct CollisionPlugin;

impl Plugin for CollisionPlugin {
    fn build(&self, app: &mut App) {
        use crate::app_state::overworld::character::MovementSet;
        use systems::*;
        app.add_systems(Update, player_tilemap_collision_system.after(MovementSet));
    }
}
