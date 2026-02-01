//! # core.rs
//!
//! # core.rs 文件
//!
//! ## Module Overview
//!
//! ## 模块概述
//!
//! The `core` module contains application-wide systems such as input handling, camera logic, and animation.
//!
//! core 模块包含输入处理、摄像机逻辑与动画等贯穿应用程序的系统。
//!
//! It is the essential architecture that the rest of the game relies on.
//!
//! 它是游戏运行所必需的核心架构。
//!
//! The module should avoid concrete game logic and instead provide infrastructure to support it.
//!
//! 该模块不应包含具体游戏逻辑，而是提供支撑这些逻辑所需的基础设施。
//!
//! > Note: Optional application-wide components belong in `shared` instead of `core`.
//!
//! > 注意：可选的全局功能组件应放入 `shared` 模块，而不是 `core`。
//!
//! # Source File Overview
//!
//! # 源文件概述
//!
//! This file defines the `CorePlugin` and `GlobalPlugin` that run early and late in the app lifecycle.
//!
//! 此处定义了 `CorePlugin` 与 `GlobalPlugin`，它们分别在应用生命周期的早期和后期运行。

pub(crate) mod animation;
pub(crate) mod audio;
pub(crate) mod basic_components;
pub(crate) mod camera;
pub(crate) mod character_asset;
pub(crate) mod collision;
pub mod danmaku;
pub(crate) mod data;
pub(crate) mod input;
pub mod item;
pub mod mod_system;
pub mod player_components;
pub mod render_layers;
pub mod ron_loader;
pub mod sprite;
pub(crate) mod state_config;
pub(crate) mod view;

use crate::extra;
use bevy::app::*;
use bevy::asset::AssetApp;

/// CorePlugin is a global plugin that runs early in the app lifecycle.
///
/// CorePlugin 是一个在应用程序生命周期早期运行的全局插件。
///
/// It should initialize global resources and register plugins that must exist before most systems run.
///
/// 它负责初始化全局资源，并注册在大多数系统运行前就必须存在的插件。
pub(crate) struct CorePlugin;

impl Plugin for CorePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<extra::toml::config::TomlConfigRegistry>()
            .init_asset::<character_asset::CharacterAsset>()
            .init_asset::<character_asset::AnimationConfigAsset>()
            .register_asset_loader(
                ron_loader::RonAssetLoader::<character_asset::CharacterAsset>::new(&[
                    "character.ron",
                ]),
            )
            .register_asset_loader(ron_loader::RonAssetLoader::<
                character_asset::AnimationConfigAsset,
            >::new(&["animation.ron"]))
            .add_plugins((
                animation::AnimationPlugin,
                audio::AudioPlugin,
                camera::CameraPlugin,
                collision::CollisionPlugin,
                danmaku::CoreDanmakuPlugin,
                data::DataPlugin,
                input::InputPlugin,
                item::ItemPlugin,
                sprite::SpritePlugin,
                state_config::StateConfigPlugin,
            ));
    }
}

/// GlobalPlugin is a global plugin that runs late in the app lifecycle.
///
/// GlobalPlugin 是一个在应用程序生命周期后期运行的全局插件。
///
/// It registers systems that should only run after most initialization work finishes.
///
/// 它注册需要在大部分初始化完成后才运行的系统。
pub(crate) struct GlobalPlugin;

impl Plugin for GlobalPlugin {
    fn build(&self, _app: &mut App) {
        // All systems are now managed by their respective plugins.
        //
        // 现在所有系统都由各自的插件管理。
    }
}
