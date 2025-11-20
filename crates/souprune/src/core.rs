//! # core.rs
//! ## Module Overview
//! core is a module that contains the core functional components
//! that run throughout the application, such as input handling,
//! camera control, animation systems, etc.
//!
//! core is the **essential** core architecture for the game to run.
//!
//! It should not contain specific game logic, but rather provide
//! the infrastructure and tools needed to support that logic.
//!
//! > Note: If optional functional components that run throughout the application
//!
//! # Source File Overview
//! Here, the main plugins of the core module, `CorePlugin` and `GlobalPlugin`,
//! are defined, which run as global plugins early and late in the application lifecycle, respectively.
//!
//! ## 模块概述
//! core 是一个模块。它包含了贯穿整个应用程序的核心功能组件，如输入处理、摄像机控制、动画系统等。
//!
//! core 是游戏运行* *必须的** 核心架构。
//!
//! core 不应当包含具体的游戏逻辑，而是提供支撑这些逻辑所需的基础设施和工具。
//!
//! > 注意：将来如果出现可选的贯穿应用的功能组件，应放入 `shared` 模块，而非 `core`。
//!
//! ## 源文件概述
//! 此处定义了 core 模块的主要插件 `CorePlugin` 和 `GlobalPlugin`，
//! 它们分别在应用程序生命周期的早期和后期作为全局插件运行。

pub(crate) mod animation;
pub(crate) mod basic_components;
pub(crate) mod camera;
pub(crate) mod collision;
pub(crate) mod data;
pub(crate) mod input;
pub(crate) mod sprite;

use crate::extra;
use bevy::app::*;

/// CorePlugin is a global plugin that runs early in the app lifecycle.
///
/// It should be used to initialize global resources and register plugins
/// that need to be available before most systems run.
///
/// CorePlugin 是一个在应用程序生命周期早期运行的全局插件。
///
/// 它应用于初始化全局资源和注册需要在大部分系统运行之前可用的插件。
pub(crate) struct CorePlugin;

impl Plugin for CorePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<extra::toml::config::TomlConfigRegistry>()
            .add_plugins((
                animation::AnimationPlugin,
                camera::CameraPlugin,
                collision::CollisionPlugin,
                data::DataPlugin,
                input::InputPlugin,
                sprite::SpritePlugin,
            ));
    }
}

/// GlobalPlugin is a global plugin that runs late in the app lifecycle.
///
/// It should be used to register systems that need to run after most initialization has completed.
///
/// GlobalPlugin 是一个在应用程序生命周期后期运行的全局插件。
///
/// 它应用于注册需要在大部分初始化完成后运行的系统。
pub(crate) struct GlobalPlugin;

impl Plugin for GlobalPlugin {
    fn build(&self, _app: &mut App) {
        // 现在所有系统都由各自的插件管理
        // All systems are now managed by their respective plugins
    }
}
