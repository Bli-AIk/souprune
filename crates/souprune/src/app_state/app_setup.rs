//! # app_setup.rs
//!
//! ## Module Overview
//! This module handles the initial setup of the application, including loading assets, configuring the camera,
//! and managing the transition to the main game states.
//!
//! ## Source File Overview
//! This file defines the `AppSetupPlugin`, which orchestrates the loading of textures, camera initialization,
//! and state transitions during the application's startup phase.
//!
//! ## 模块概述
//! 该模块处理应用程序的初始设置，包括加载资产、配置摄像机，以及管理到主要游戏状态的过渡。
//!
//! ## 源文件概述
//! 该文件定义了 `AppSetupPlugin`，它在应用程序启动阶段协调纹理加载、摄像机初始化和状态转换。

use crate::app_state::AppState;
use crate::core::camera::Followable;
use crate::core::sprite::ModuleSpriteRegistry;
use bevy::app::{App, Plugin, Update};
use bevy::asset::LoadedFolder;
use bevy::prelude::*;

pub(crate) struct AppSetupPlugin;

impl Plugin for AppSetupPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(AppState::AppSetup),
            (load_textures_system, setup_camera_system),
        )
        .add_systems(
            Update,
            check_textures_system.run_if(in_state(AppState::AppSetup)),
        );
    }
}
fn load_textures_system(mut commands: Commands, asset_server: Res<AssetServer>) {
    let mut registry = ModuleSpriteRegistry::new();
    let mut register = (&mut registry, &asset_server);

    // Register for modules here!
    register_module(&mut register, "overworld");
    register_module(&mut register, "battle");

    commands.insert_resource(registry);
}

fn register_module(
    (registry, asset_server): &mut (&mut ModuleSpriteRegistry, &Res<AssetServer>),
    module_name: &str,
) {
    registry.register_module(
        module_name.to_string(),
        asset_server.load_folder(format!("textures/{}", module_name)),
    );
}

fn check_textures_system(
    mut next_state: ResMut<NextState<AppState>>,
    sprite_registry: Res<ModuleSpriteRegistry>,
    mut events: MessageReader<AssetEvent<LoadedFolder>>,
) {
    // TODO 配置于toml文件
    // 目前会检查所有需要的Sprite是否加载完成，然后才切换状态
    // 但是这样做不够灵活
    // 我们应该在toml文件中配置某个AppState加载前，需要哪些模块的Sprite
    for event in events.read() {
        if let Some(handle) = sprite_registry.get_module("overworld")
            && event.is_loaded_with_dependencies(handle)
        {
            next_state.set(AppState::Overworld);
        }
    }
}

fn setup_camera_system(mut commands: Commands, resolution_scale: Res<ResolutionScale>) {
    commands.spawn((
        Camera2d,
        Transform::from_scale(Vec3::splat(1.0 / resolution_scale.get() as f32)),
        Followable::default(),
    ));
}

#[derive(Resource)]
pub(crate) struct ResolutionScale(pub(crate) u32);

impl ResolutionScale {
    pub(crate) fn get(&self) -> u32 {
        self.0
    }
}

impl Default for ResolutionScale {
    fn default() -> Self {
        // (320, 240) * 2
        Self(5)
    }
}