//! # SoupRune Editor Standalone Entry Point
//!
//! # SoupRune 编辑器独立入口
//!
//! ## Module Overview
//!
//! ## 模块概述
//!
//! Standalone entry point for the SoupRune editor.
//!
//! SoupRune 编辑器的独立入口点。

use bevy::asset::io::file::FileAssetReader;
use bevy::asset::io::{AssetSourceBuilder, AssetSourceId};
use bevy::prelude::*;
use bevy_workbench::console::console_log_layer;
use souprune::config;
use souprune::editor_api::multi_source::MultiSourceAssetReader;
use souprune_editor::SoupRuneEditorPlugin;

fn main() {
    let cfg = config::load_config();
    let project_name = cfg.project.mod_name.clone();

    App::new()
        // 资源源必须在 DefaultPlugins 之前注册
        .register_asset_source(
            AssetSourceId::Default,
            AssetSourceBuilder::new(move || {
                let roots = config::get_asset_roots(&project_name);
                let readers = roots.into_iter().map(FileAssetReader::new).collect();
                Box::new(MultiSourceAssetReader::new(readers))
            }),
        )
        .insert_resource(cfg)
        .add_plugins(
            DefaultPlugins
                .set(ImagePlugin::default_nearest())
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "SoupRune Editor".to_string(),
                        resolution: (1280u32, 720u32).into(),
                        ..default()
                    }),
                    ..default()
                })
                .set(bevy::asset::AssetPlugin {
                    unapproved_path_mode: bevy::asset::UnapprovedPathMode::Allow,
                    ..default()
                })
                .set(bevy::log::LogPlugin {
                    custom_layer: console_log_layer,
                    ..default()
                }),
        )
        .insert_resource(ClearColor(Color::BLACK))
        .add_plugins(SoupRuneEditorPlugin)
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);
}
