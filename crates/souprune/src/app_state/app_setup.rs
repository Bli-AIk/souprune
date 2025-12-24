//! # app_setup.rs
//!
//! # app_setup.rs 文件
//!
//! ## Module Overview
//!
//! ## 模块概述
//!
//! This module loads assets, configures the camera, and transitions into the main game states.
//!
//! 该模块负责加载资产、配置摄像机并切换到主要游戏状态。
//!
//! ## Source File Overview
//!
//! ## 源文件概述
//!
//! It defines `AppSetupPlugin`, which orchestrates texture loading, camera setup, and startup transitions.
//!
//! 文件实现了 `AppSetupPlugin`，用于协调纹理加载、摄像机设置与启动阶段的状态转换。

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

    // Register sprite modules here.
    //
    // 在此注册需要的精灵模块。
    register_module(&mut register, "overworld");
    register_module(&mut register, "battle");
    register_module(&mut register, "common");

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
    asset_server: Res<AssetServer>,
    mut events: MessageReader<AssetEvent<LoadedFolder>>,
) {
    // TODO 配置于toml文件
    // 目前会检查所有需要的Sprite是否加载完成，然后才切换状态
    // 但是这样做不够灵活
    // 我们应该在toml文件中配置某个AppState加载前，需要哪些模块的Sprite
    for _ in events.read() {
        let all_loaded = ["overworld", "common"].into_iter().all(|module| {
            if let Some(handle) = sprite_registry.get_module(module) {
                asset_server.is_loaded_with_dependencies(handle)
            } else {
                false
            }
        });

        if all_loaded {
            next_state.set(AppState::Overworld);
            break;
        }
    }
}

fn setup_camera_system(mut commands: Commands, resolution_scale: Res<ResolutionScale>) {
    commands.spawn((
        // TODO: 区分 OW Camera 和 Battle Camera
        Name::new("Overworld Camera2d"),
        Camera2d,
        Projection::Orthographic(OrthographicProjection {
            scale: 1.0 / resolution_scale.get() as f32,
            ..OrthographicProjection::default_2d()
        }),
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
        // Equivalent to (320, 240) * 2 resolution.
        //
        // 等效于 (320, 240) * 2 的分辨率。
        Self(5)
    }
}
