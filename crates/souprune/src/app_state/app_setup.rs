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
//! 该模块负责加载资源、配置摄像机并切换到主要游戏状态。
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
use bevy::app::{App, Plugin};
use bevy::asset::LoadedFolder;
use bevy::ecs::schedule::ScheduleLabel;
use bevy::prelude::*;
use std::fs;

pub use crate::core::camera::ResolutionScale;

pub struct AppSetupPlugin;

impl Plugin for AppSetupPlugin {
    fn build(&self, app: &mut App) {
        let schedule = crate::game_schedule(app);
        app.add_systems(
            OnEnter(AppState::Loading),
            (
                load_textures_system,
                setup_camera_system,
                #[cfg(not(target_os = "android"))]
                setup_touch_overlay_system,
            ),
        )
        .add_systems(
            schedule,
            (
                check_textures_system.run_if(in_state(AppState::Loading)),
                crate::core::input::touch::update_touch_button_visuals,
                crate::core::input::touch::tick_touch_button_animations,
                crate::core::input::touch::update_controller_directions,
                crate::core::input::touch::update_controller_overlays
                    .after(crate::core::input::touch::update_controller_directions),
            ),
        );

        // On Android, defer touch overlay until window is ready, and maintain 4:3 viewport
        #[cfg(target_os = "android")]
        app.add_systems(
            schedule,
            (
                android_viewport_system,
                deferred_touch_overlay_system.run_if(not(resource_exists::<TouchOverlaySpawned>)),
            ),
        );
    }
}

/// Discovers texture modules by scanning the textures directory.
///
/// 通过扫描 textures 目录发现纹理模块。
fn discover_texture_modules() -> Vec<String> {
    let roots = config::get_all_asset_roots();
    let mut all_modules = Vec::new();

    for root in roots {
        let textures_path = root.join("textures");
        let Ok(entries) = fs::read_dir(&textures_path) else {
            continue;
        };
        if !textures_path.is_dir() {
            continue;
        }

        let modules: Vec<String> = entries
            .filter_map(|entry| entry.ok())
            .filter(|entry| entry.path().is_dir())
            .filter_map(|entry| entry.file_name().into_string().ok())
            .collect();

        if modules.is_empty() {
            continue;
        }
        info!(
            "Discovered {} texture modules in {:?}: {:?}",
            modules.len(),
            textures_path,
            modules
        );
        for m in modules {
            if !all_modules.contains(&m) {
                all_modules.push(m);
            }
        }
    }

    if !all_modules.is_empty() {
        return all_modules;
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
            asset_server.load_folder(format!("assets/textures/{}", module_name)),
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
    mut sequence_mode: ResMut<crate::app_state::SequenceMode>,
    sprite_registry: Res<ModuleSpriteRegistry>,
    asset_server: Res<AssetServer>,
    mut events: MessageReader<AssetEvent<LoadedFolder>>,
    souprune_config: Res<crate::config::SoupruneConfig>,
    discovered_modules: Res<DiscoveredModules>,
) {
    for _ in events.read() {
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

        let all_discovered_loaded = discovered_modules.0.iter().all(|module| {
            if let Some(handle) = sprite_registry.get_module(module) {
                asset_server.is_loaded_with_dependencies(handle)
            } else {
                false
            }
        });

        if required_loaded && all_discovered_loaded {
            info!("All texture modules loaded: {:?}", discovered_modules.0);

            next_state.set(AppState::Running);

            // Mode is data-driven: prefer initial_sequence_path inference,
            // fall back to config's initial_mode.
            let mode = if souprune_config.game.initial_sequence_path.is_none()
                && !souprune_config.game.initial_battle_path.is_empty()
            {
                info!(
                    "No initial_sequence_path; initial_battle_path found. Using initial_mode from battle path."
                );
                "battle".to_string()
            } else {
                souprune_config.game.initial_mode.clone()
            };
            info!("Setting initial SequenceMode: {:?}", mode);
            sequence_mode.0 = Some(mode);
            break;
        }
    }
}

fn setup_camera_system(
    mut commands: Commands,
    #[cfg(not(target_os = "android"))] resolution_scale: Res<ResolutionScale>,
    game_schedule: Option<Res<crate::GameUpdateSchedule>>,
    existing: Query<(), With<crate::core::camera::MainGameCamera>>,
    #[cfg(target_os = "android")] config: Option<Res<crate::config::SoupruneConfig>>,
) {
    // Idempotent: skip if camera already exists (prevents duplicates on re-enter Loading)
    if !existing.is_empty() {
        return;
    }

    let is_editor = game_schedule
        .as_ref()
        .is_some_and(|gs| gs.0 != Update.intern());

    // Android 使用 Fixed 缩放 + viewport 系统保持宽高比（见 android_viewport_system）。
    // 编辑器和独立游戏使用 WindowSize + scale：render target 分辨率 = base * scale，
    // 相机可视区域 = render_target_size / scale = base_resolution，像素完美。
    #[cfg(target_os = "android")]
    let projection = {
        let base_w = config
            .as_ref()
            .map_or(320.0, |c| c.render.base_resolution_width as f32);
        let base_h = config
            .as_ref()
            .map_or(240.0, |c| c.render.base_resolution_height as f32);
        Projection::Orthographic(OrthographicProjection {
            scaling_mode: bevy::camera::ScalingMode::Fixed {
                width: base_w,
                height: base_h,
            },
            ..OrthographicProjection::default_2d()
        })
    };
    #[cfg(not(target_os = "android"))]
    let projection = Projection::Orthographic(OrthographicProjection {
        scale: 1.0 / resolution_scale.get() as f32,
        ..OrthographicProjection::default_2d()
    });

    let mut entity = commands.spawn((
        Name::new("Overworld Camera2d"),
        Camera2d,
        projection,
        Followable::default(),
        crate::core::camera::MainGameCamera,
    ));

    // 编辑器模式下禁用游戏相机（由 GameViewPlugin 在 Play 时劫持激活）
    if is_editor {
        entity.insert(Camera {
            is_active: false,
            ..default()
        });
    }
}

fn setup_touch_overlay_system(
    mut commands: Commands,
    registry: Res<crate::core::input::ActionRegistry>,
    enabled: Res<crate::core::input::touch::TouchOverlayEnabled>,
    asset_server: Res<AssetServer>,
    layout: Option<Res<crate::core::input::config::TouchLayoutDef>>,
    windows: Query<&Window>,
    resolution_scale: Res<ResolutionScale>,
    souprune_config: Res<crate::config::SoupruneConfig>,
) {
    if enabled.0 {
        let window_width = windows.iter().next().map(|w| w.width());
        crate::core::input::touch::spawn_touch_overlay(
            &mut commands,
            &registry,
            &asset_server,
            layout.as_deref(),
            window_width,
            resolution_scale.get(),
            souprune_config.render.base_resolution_width,
        );
    }
}

/// On Android, set camera viewport to maintain 4:3 aspect ratio with letterboxing.
#[cfg(target_os = "android")]
fn android_viewport_system(
    windows: Query<&Window>,
    mut cameras: Query<&mut Camera, With<Camera2d>>,
    souprune_config: Res<config::SoupruneConfig>,
) {
    let Ok(window) = windows.single() else {
        return;
    };
    let screen_w = window.physical_width() as f32;
    let screen_h = window.physical_height() as f32;
    if screen_w <= 0.0 || screen_h <= 0.0 {
        return;
    }

    let target_ratio = souprune_config.render.base_resolution_width as f32
        / souprune_config.render.base_resolution_height as f32;
    let screen_ratio = screen_w / screen_h;

    let (vp_w, vp_h, offset_x, offset_y) = if screen_ratio > target_ratio {
        // Wider than 4:3 → pillarbox (black bars on sides)
        let vp_h = screen_h as u32;
        let vp_w = (screen_h * target_ratio) as u32;
        let offset_x = (screen_w as u32 - vp_w) / 2;
        (vp_w, vp_h, offset_x, 0)
    } else {
        // Taller than 4:3 → letterbox (black bars top/bottom)
        let vp_w = screen_w as u32;
        let vp_h = (screen_w / target_ratio) as u32;
        let offset_y = (screen_h as u32 - vp_h) / 2;
        (vp_w, vp_h, 0, offset_y)
    };

    for mut camera in cameras.iter_mut() {
        camera.viewport = Some(bevy::camera::Viewport {
            physical_position: UVec2::new(offset_x, offset_y),
            physical_size: UVec2::new(vp_w, vp_h),
            ..default()
        });
    }
}

/// Marker resource indicating that the touch overlay has been spawned on Android.
#[cfg(target_os = "android")]
#[derive(Resource)]
struct TouchOverlaySpawned;

/// On Android, spawn the touch overlay once the window has its actual dimensions.
/// The window reports default size (1280x720) before it's fully initialized.
#[cfg(target_os = "android")]
fn deferred_touch_overlay_system(
    mut commands: Commands,
    registry: Option<Res<crate::core::input::ActionRegistry>>,
    enabled: Res<crate::core::input::touch::TouchOverlayEnabled>,
    asset_server: Res<AssetServer>,
    layout: Option<Res<crate::core::input::config::TouchLayoutDef>>,
    windows: Query<&Window>,
    resolution_scale: Res<ResolutionScale>,
    souprune_config: Res<crate::config::SoupruneConfig>,
) {
    let Some(registry) = registry else { return };
    if !enabled.0 {
        return;
    }
    let Ok(window) = windows.single() else {
        return;
    };
    // Wait until the physical dimensions differ from the default (1280x720).
    let phys_w = window.physical_width();
    let phys_h = window.physical_height();
    if phys_w == 1280 && phys_h == 720 || phys_w == 0 || phys_h == 0 {
        return;
    }

    let logical_width = window.width();
    info!(
        "Android window ready: physical={}x{}, logical_width={}, scale_factor={}",
        phys_w,
        phys_h,
        logical_width,
        window.scale_factor()
    );
    crate::core::input::touch::spawn_touch_overlay(
        &mut commands,
        &registry,
        &asset_server,
        layout.as_deref(),
        Some(logical_width),
        resolution_scale.get(),
        souprune_config.render.base_resolution_width,
    );
    commands.insert_resource(TouchOverlaySpawned);
}
