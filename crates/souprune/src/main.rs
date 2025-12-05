//! # main.rs
//!
//! # main.rs 文件
//!
//! ## Module Overview
//!
//! ## 模块概述
//!
//! This standalone file houses the entry point without any submodules.
//!
//! main.rs 是一个没有子模块的独立入口文件。
//!
//! ## Source File Overview
//!
//! ## 源文件概述
//!
//! It initializes the Bevy app, registers resources and states, and manages every plugin.
//!
//! 文件负责初始化 Bevy 应用、注册资源与状态，并管理所有插件。

mod app_state;
mod core;
mod extra;

use std::default::*;

use crate::core::*;
use crate::extra::multi_source::MultiSourceAssetReader;
use app_state::{app_setup, overworld};
use bevy::app::PluginGroupBuilder;
use bevy::asset::io::file::FileAssetReader;
use bevy::asset::io::{AssetSource, AssetSourceId};
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
        (
            extra::markdown::MarkdownPlugin,
            extra::toml::TomlPlugin,
            extra::mortar::MortarExtraPlugin,
        )
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
        // TODO: Read mod config and load the correct project
        .register_asset_source(
            AssetSourceId::Default,
            AssetSource::build().with_reader(|| {
                let project_name = "test_mod";

                let project_path = format!("projects/{}", project_name);

                let mut readers = vec![
                    // Priority 1: Distribution / Standalone (folder next to executable)
                    FileAssetReader::new(&project_path),
                ];

                // Priority 2: Development (absolute path based on source location, strictly for debug builds)
                #[cfg(feature = "debug")]
                {
                    readers.push(FileAssetReader::new(
                        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                            .join("../../")
                            .join(&project_path),
                    ));
                }

                // Priority 3: Core Fallback (embedded assets)
                readers.push(FileAssetReader::new("assets"));

                Box::new(MultiSourceAssetReader::new(readers))
            }),
        )
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
