//! # core.rs
//! ## Overview
//! core is a module that contains the core functional components
//! that run throughout the application, such as input handling,
//! camera control, animation systems, etc.
//!
//! It should not contain specific game logic, but rather provide
//! the infrastructure and tools needed to support that logic.
//!
//!
//! ## 概述
//! core 是一个模块。它包含了贯穿整个应用程序的核心功能组件，如输入处理、摄像机控制、动画系统等。
//!
//! core 中 不应当包含具体的游戏逻辑，而是提供支撑这些逻辑所需的基础设施和工具。

// TODO: 重构 core，并将其所有具体的游戏逻辑移出 core 模块

mod basic_components;
pub(crate) mod input;
pub(crate) mod overworld;
pub(crate) mod setup;
pub(crate) mod sprite;

mod animation;
mod camera;

use crate::extra;
use bevy::app::*;

/// CorePlugin 是一个在应用程序生命周期早期运行的全局插件。
///
/// 它应用于初始化全局资源和注册需要在大部分系统运行之前可用的插件。
///
/// CorePlugin is a global plugin that runs early in the app lifecycle.
///
/// It should be used to initialize global resources and register plugins
/// that need to be available before most systems run.
pub(crate) struct CorePlugin;

impl Plugin for CorePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<extra::toml::config::TomlConfigRegistry>()
            .add_plugins(animation::AnimationPlugin);
    }
}
/// GlobalPlugin 是一个在应用程序生命周期后期运行的全局插件。
///
/// 它应用于注册需要在大部分初始化完成后运行的系统。
///
/// GlobalPlugin is a global plugin that runs late in the app lifecycle.
///
/// It should be used to register systems that need to run after most initialization has completed.
pub(crate) struct GlobalPlugin;

impl Plugin for GlobalPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, camera::update_followable_camera_system);
    }
}
