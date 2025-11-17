//! # main.rs
//!
//! ## Module Overview
//! This file does not contain any submodules; it is a standalone file. Keep it simple.
//!
//! ## Source File Overview
//! Welcome! This is the entry point of the program.
//!
//! Here, the entire Bevy application is initialized, along with related resources and states,
//! and all plugins are managed.
//!
//! ## 模块概述
//! main.rs 不包含任何子模块，它只是一个单独的文件，一切从简。
//!
//! ## 源文件概述
//! 欢迎！这是程序的入口点。
//!
//! 这里初始化了整个 Bevy 应用程序，以及相关资源和状态，并管理所有插件。

mod app_state;
mod core;
mod extra;

use std::default::*;

use crate::core::*;
use app_state::{app_setup, overworld};
use bevy::app::PluginGroupBuilder;
use bevy::prelude::*;
use bevy::window::*;

/// Get the default Bevy plugins with custom window size and image plugin settings.
///
/// 获取具有自定义窗口大小和图像插件设置的默认 Bevy 插件。
fn get_bevy_default_plugins() -> PluginGroupBuilder {
    let resolution_scale = app_setup::ResolutionScale::default();
    DefaultPlugins
        .set(ImagePlugin::default_nearest())
        .set(WindowPlugin {
            primary_window: Some(Window {
                resolution: WindowResolution::new(
                    320 * resolution_scale.get(),
                    240 * resolution_scale.get(),
                ),
                resizable: false,
                ..default()
            }),
            ..default()
        })
}

/// Get the file importer plugins used in the application.
///
/// 获取应用程序中使用的文件导入器插件。
macro_rules! get_file_importer_plugins {
    () => {
        (extra::markdown::MarkdownPlugin, extra::toml::TomlPlugin)
    };
}

/// Get the third-party plugins used in the application.
///
/// 获取应用程序中使用的第三方插件。
macro_rules! get_third_plugins {
    () => {
        (
            leafwing_input_manager::prelude::InputManagerPlugin::<crate::core::input::Action>::default(),
            seldom_state::prelude::StateMachinePlugin::default(),
            bevy_ecs_tiled::prelude::TiledPlugin::default(),
            bevy_smud::SmudPlugin,
            bevy_rich_text3d::Text3dPlugin{
                default_atlas_dimension: (1024, 1024),
                load_system_fonts: false,
                ..Default::default()
            }

        )
    };
}

/// Get the game-specific plugins.
///
/// 获取特定于游戏的插件。
macro_rules! get_game_plugins {
    () => {
        (
            CorePlugin,
            app_setup::AppSetupPlugin,
            overworld::OverworldPlugin,
            GlobalPlugin,
        )
    };
}
fn main() {
    App::new()
        .init_resource::<app_setup::ResolutionScale>()
        .add_plugins((
            get_bevy_default_plugins(),
            get_file_importer_plugins!(),
            get_third_plugins!(),
            #[cfg(feature = "debug")]
            extra::debug::DebugPlugin,
        ))
        .insert_resource(bevy_rich_text3d::LoadFonts {
            font_directories: vec!["crates/souprune/assets/fonts".to_owned()],
            ..Default::default()
        })
        .init_resource::<input::PlayerInputSettings>()
        .init_state::<app_state::AppState>()
        .add_plugins(get_game_plugins!())
        .run();
}
