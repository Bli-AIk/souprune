//! # camera.rs
//!
//! # camera.rs 文件
//!
//! ## Module Overview
//!
//! ## 模块概述
//!
//! This module provides camera control systems, especially for followable cameras.
//!
//! 该模块提供面向可跟随摄像机的控制系统。
//!
//! ## Source File Overview
//!
//! ## 源文件概述
//!
//! It defines `CameraPlugin`, which updates and manages the camera systems.
//!
//! 本文件定义了负责更新与管理摄像机系统的 `CameraPlugin`。

pub(crate) mod components;
mod systems;

pub(crate) use components::*;

use crate::core::collision::systems::player_tilemap_collision_system;
use bevy::app::{App, Plugin, Update};
use bevy::prelude::IntoScheduleConfigs;

pub(crate) struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        use systems::*;
        app.add_systems(
            Update,
            update_followable_camera_system.after(player_tilemap_collision_system),
        );
    }
}
