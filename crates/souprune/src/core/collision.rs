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

/// Collision plugin for managing collision detection components and systems.
///
/// 碰撞插件，用于管理碰撞检测组件和系统。
pub(crate) struct CollisionPlugin;

impl Plugin for CollisionPlugin {
    fn build(&self, app: &mut App) {}
}
