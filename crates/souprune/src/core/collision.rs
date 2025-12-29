//! # collision.rs
//!
//! # collision.rs 文件
//!
//! ## Module Overview
//!
//! ## 模块概述
//!
//! This module offers collision detection components, SDF functions, and utilities.
//! Concrete collision systems are implemented in app_state modules.
//!
//! 该模块提供碰撞检测组件、SDF函数和工具函数。
//! 具体的碰撞系统在 app_state 模块中实现。

pub mod battle_collision;
pub(crate) mod components;
pub mod systems;

pub use battle_collision::*;
pub use components::*;
pub use systems::{SdfCollisionConfig, collect_nearby_colliders, resolve_sdf_collision};

use bevy::app::*;

/// Collision plugin for managing collision detection components.
/// Note: Collision systems are registered by their respective app_state plugins.
///
/// 碰撞插件，用于管理碰撞检测组件。
/// 注意：碰撞系统由各自的 app_state 插件注册。
pub(crate) struct CollisionPlugin;

impl Plugin for CollisionPlugin {
    fn build(&self, _app: &mut App) {
        // Core collision plugin now only provides components and utilities.
        // Collision systems are registered by overworld and battle plugins.
        //
        // Core碰撞插件现在只提供组件和工具函数。
        // 碰撞系统由 overworld 和 battle 插件注册。
    }
}
