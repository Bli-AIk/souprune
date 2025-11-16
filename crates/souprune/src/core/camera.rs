//! # camera.rs
//!
//! ## Module Overview
//! This module provides core functionalities for camera control, especially for followable cameras.
//!
//! ## Source File Overview
//! This file defines the `CameraPlugin`, which manages camera systems, including updating followable cameras.
//!
//! ## 模块概述
//! 该模块提供了核心的摄像机控制功能，特别是针对可跟随摄像机。
//!
//! ## 源文件概述
//! 该文件定义了 `CameraPlugin`，它管理摄像机系统，包括更新可跟随摄像机。

pub(crate) mod components;
mod systems;

pub(crate) use components::*;

use bevy::app::{App, Plugin, Update};

pub(crate) struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        use systems::*;
        app.add_systems(Update, update_followable_camera_system);
    }
}
