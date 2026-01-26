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
use crate::config;
use crate::core::camera::Followable;
use crate::core::sprite::ModuleSpriteRegistry;
use bevy::app::{App, Plugin, Update};
use bevy::asset::LoadedFolder;
use bevy::prelude::*;
use std::fs;

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

/// Discovers texture modules by scanning the textures directory.
///
/// 通过扫描 textures 目录发现纹理模块。
fn discover_texture_modules() -> Vec<String> {
    let config = config::load_config();
    let roots = config::get_asset_roots(&config.project.mod_name);

    for root in roots {
        let textures_path = root.join("textures");
        if textures_path.exists() && textures_path.is_dir() {
            if let Ok(entries) = fs::read_dir(&textures_path) {
                let modules: Vec<String> = entries
                    .filter_map(|entry| entry.ok())
                    .filter(|entry| entry.path().is_dir())
                    .filter_map(|entry| entry.file_name().into_string().ok())
                    .collect();

                if !modules.is_empty() {
                    info!(
                        "Discovered {} texture modules in {:?}: {:?}",
                        modules.len(),
                        textures_path,
                        modules
                    );
                    return modules;
                }
            }
        }
    }

    // Fallback to default modules if no modules discovered
    //
    // 如果没有发现模块，则回退到默认模块
    warn!("No texture modules discovered, using defaults");
    vec![
        "overworld".to_string(),
        "battle".to_string(),
        "common".to_string(),
    ]
}

fn load_textures_system(mut commands: Commands, asset_server: Res<AssetServer>) {
    let mut registry = ModuleSpriteRegistry::new();

    // Discover available modules dynamically
    //
    // 动态发现可用模块
    let discovered_modules = discover_texture_modules();

    // Register all discovered modules
    //
    // 注册所有发现的模块
    for module_name in &discovered_modules {
        registry.register_module(
            module_name.clone(),
            asset_server.load_folder(format!("textures/{}", module_name)),
        );
        info!("Registered texture module: {}", module_name);
    }

    // Store discovered modules for checking later
    //
    // 存储发现的模块以供后续检查
    commands.insert_resource(DiscoveredModules(discovered_modules));
    commands.insert_resource(registry);
}

/// Resource storing the list of discovered texture modules.
///
/// 存储发现的纹理模块列表的资源。
#[derive(Resource)]
pub struct DiscoveredModules(pub Vec<String>);

fn check_textures_system(
    mut next_state: ResMut<NextState<AppState>>,
    sprite_registry: Res<ModuleSpriteRegistry>,
    asset_server: Res<AssetServer>,
    mut events: MessageReader<AssetEvent<LoadedFolder>>,
    souprune_config: Res<crate::config::SoupruneConfig>,
    discovered_modules: Res<DiscoveredModules>,
) {
    for _ in events.read() {
        // Check that all required modules are loaded
        // Required modules come from config, but must be present in discovered modules
        //
        // 检查所有必需模块是否已加载
        // 必需模块来自配置，但必须存在于发现的模块中
        let required_loaded = souprune_config.game.required_modules.iter().all(|module| {
            if !discovered_modules.0.contains(module) {
                warn!(
                    "Required module '{}' not found in discovered modules",
                    module
                );
                return false;
            }
            if let Some(handle) = sprite_registry.get_module(module) {
                asset_server.is_loaded_with_dependencies(handle)
            } else {
                false
            }
        });

        // Also check that all discovered modules are loaded
        //
        // 同时检查所有发现的模块是否已加载
        let all_discovered_loaded = discovered_modules.0.iter().all(|module| {
            if let Some(handle) = sprite_registry.get_module(module) {
                asset_server.is_loaded_with_dependencies(handle)
            } else {
                false
            }
        });

        if required_loaded && all_discovered_loaded {
            info!("All texture modules loaded: {:?}", discovered_modules.0);
            next_state.set(AppState::Overworld);
            break;
        }
    }
}

fn setup_camera_system(mut commands: Commands, resolution_scale: Res<ResolutionScale>) {
    commands.spawn((
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
