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

pub use components::CameraControlOverride;
pub(crate) use components::Followable;
pub use components::MainGameCamera;
pub use systems::CameraUpdateSet;

use bevy::app::{App, Plugin};
use bevy::prelude::IntoScheduleConfigs;

pub(crate) struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        use systems::*;
        // Configure CameraUpdateSet to run in Update schedule.
        // Other systems can use .before(CameraUpdateSet) to ensure they run first.
        //
        // 配置 CameraUpdateSet 在 Update 调度中运行。
        // 其他系统可以使用 .before(CameraUpdateSet) 确保它们先运行。
        let schedule = crate::game_schedule(app);
        app.configure_sets(schedule, CameraUpdateSet).add_systems(
            schedule,
            update_followable_camera_system.in_set(CameraUpdateSet),
        );
    }
}
