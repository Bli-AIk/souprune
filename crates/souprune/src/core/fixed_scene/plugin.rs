//! # fixed_scene/plugin.rs
//!
//! # fixed_scene/plugin.rs 文件
//!
//! ## Module Overview
//!
//! ## 模块概述
//!
//! Fixed-scene primitive wiring for project-declared modes.
//!
//! 固定相机场景 primitive 的接线，由项目声明的 mode 决定何时启用。

use crate::config::ModePrimitiveConfig;
use crate::core::fixed_scene::{
    FixedSceneCamera, FixedSceneInputManager, FixedSceneMovementSet, FixedSceneUpdate,
};
use crate::core::fixed_scene::{
    alight_motion_integration::AlightMotionFixedScenePlugin, danmaku::DanmakuPlugin,
    fre::FixedSceneFREPlugin, speech_bubble,
};
use crate::core::input::{Action, PlayerInputSettings};
use crate::core::mode::{
    ModeChanged, ModeRegistry, ModeScoped, SequenceMode, current_mode_has_primitive,
    on_entering_mode_with_primitive, on_exiting_mode_with_primitive,
};
use crate::core::sequencer::SequencerPlugin;
use bevy::app::{App, Plugin};
use bevy::ecs::message::MessageReader;
use bevy::prelude::*;
use leafwing_input_manager::action_state::ActionState;

/// Create a mode-scoped marker for the active fixed-scene mode.
///
/// 为当前 fixed-scene mode 创建作用域标记。
pub(crate) fn fixed_scene_scoped(mode_name: impl Into<String>) -> ModeScoped {
    ModeScoped(mode_name.into())
}

pub(crate) fn on_entering_fixed_scene(
    events: MessageReader<ModeChanged>,
    registry: Res<ModeRegistry>,
) -> bool {
    on_entering_mode_with_primitive(ModePrimitiveConfig::FixedScene)(events, registry)
}

pub(crate) fn on_exiting_fixed_scene(
    events: MessageReader<ModeChanged>,
    registry: Res<ModeRegistry>,
) -> bool {
    on_exiting_mode_with_primitive(ModePrimitiveConfig::FixedScene)(events, registry)
}

pub struct FixedScenePlugin;

impl Plugin for FixedScenePlugin {
    fn build(&self, app: &mut App) {
        let schedule = crate::game_schedule(app);
        app.configure_sets(
            schedule,
            FixedSceneUpdate.run_if(current_mode_has_primitive(ModePrimitiveConfig::FixedScene)),
        )
        .configure_sets(schedule, FixedSceneMovementSet.in_set(FixedSceneUpdate))
        .add_plugins((
            SequencerPlugin,
            DanmakuPlugin,
            AlightMotionFixedScenePlugin,
            FixedSceneFREPlugin,
            speech_bubble::SceneSpeechBubblePlugin,
        ))
        .add_systems(
            schedule,
            (
                load_fixed_scene_entry_sequence_system,
                setup_fixed_scene_camera,
                setup_fixed_scene_input_manager,
            )
                .run_if(on_entering_fixed_scene),
        )
        .add_systems(
            schedule,
            (
                cleanup_fixed_scene_camera,
                cleanup_fixed_scene_input_manager,
            )
                .run_if(on_exiting_fixed_scene),
        );
    }
}

fn load_fixed_scene_entry_sequence_system(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    sequence_mode: Res<SequenceMode>,
    mode_registry: Res<ModeRegistry>,
) {
    let Some(mode_name) = sequence_mode.0.as_deref() else {
        warn!("FixedScene: no active mode while entering fixed-scene primitive.");
        return;
    };
    let Some(mode_config) = mode_registry.mode(mode_name) else {
        warn!("FixedScene: active mode '{mode_name}' is not registered.");
        return;
    };
    let Some(sequence_path) = mode_config.entry_sequence.as_deref() else {
        warn!("FixedScene: mode '{mode_name}' has no entry_sequence.");
        return;
    };

    let handle =
        asset_server.load::<crate::core::sequencer::SequenceAsset>(sequence_path.to_string());
    commands.insert_resource(crate::core::sequencer::CurrentSequenceFlow(handle));
    info!("FixedScene: Loading entry sequence from '{sequence_path}' for mode '{mode_name}'");
}

fn setup_fixed_scene_camera(
    mut commands: Commands,
    mut q_game_cameras: Query<
        (Entity, &mut Camera),
        (
            With<Camera2d>,
            With<crate::core::camera::MainGameCamera>,
            Without<FixedSceneCamera>,
        ),
    >,
    q_render_targets: Query<&bevy::camera::RenderTarget>,
    resolution_scale: Res<crate::core::camera::ResolutionScale>,
    config: Res<crate::config::SoupruneConfig>,
    sequence_mode: Res<SequenceMode>,
    mode_registry: Res<ModeRegistry>,
) {
    let Some(mode_name) = sequence_mode.0.as_deref() else {
        warn!("FixedScene: no active mode while setting up camera.");
        return;
    };
    let zoom = mode_registry
        .mode(mode_name)
        .map(|mode_config| mode_config.fixed_camera_zoom())
        .unwrap_or(2.0);
    let scale_value = resolution_scale.get();
    info!(
        "[FixedScene] Setting up camera for mode '{}' with resolution_scale={}, zoom={}, camera_scale={}",
        mode_name,
        scale_value,
        zoom,
        zoom / scale_value as f32
    );

    // Capture render target and camera order before deactivating the main game camera,
    // so the fixed-scene camera inherits them (e.g. editor texture render target).
    let mut inherited_order: isize = 0;
    let mut inherited_target: Option<bevy::camera::RenderTarget> = None;

    for (entity, mut camera) in q_game_cameras.iter_mut() {
        inherited_order = camera.order;
        inherited_target = q_render_targets.get(entity).ok().cloned();
        camera.is_active = false;
    }

    #[cfg(target_os = "android")]
    let projection =
        fixed_scene_projection(&config.render, zoom, FixedSceneProjectionPlatform::Android);
    #[cfg(not(target_os = "android"))]
    let projection = fixed_scene_projection(
        &config.render,
        zoom,
        FixedSceneProjectionPlatform::Desktop {
            resolution_scale: scale_value,
        },
    );

    info!(
        "[Sequencer] Inherited camera settings: order={}, has_render_target={}",
        inherited_order,
        inherited_target.is_some()
    );

    let mut battle_cam = commands.spawn((
        Camera2d,
        Camera {
            order: inherited_order,
            ..default()
        },
        // Explicitly position at world origin — all fixed-scene View nodes and game entities
        // use world-space coordinates relative to (0, 0, 0).
        Transform::from_translation(Vec3::ZERO),
        projection,
        FixedSceneCamera,
        crate::core::camera::MainGameCamera,
        IsDefaultUiCamera,
        fixed_scene_scoped(mode_name),
        Name::new("Fixed Scene Camera2d"),
    ));
    if let Some(target) = inherited_target {
        battle_cam.insert(target);
    }
}

#[derive(Clone, Copy)]
enum FixedSceneProjectionPlatform {
    Android,
    Desktop { resolution_scale: u32 },
}

fn fixed_scene_projection(
    render_config: &crate::config::RenderConfig,
    fixed_scene_camera_zoom: f32,
    platform: FixedSceneProjectionPlatform,
) -> Projection {
    match platform {
        FixedSceneProjectionPlatform::Android => Projection::Orthographic(OrthographicProjection {
            scaling_mode: bevy::camera::ScalingMode::Fixed {
                width: render_config.base_resolution_width as f32,
                height: render_config.base_resolution_height as f32,
            },
            scale: fixed_scene_camera_zoom,
            ..OrthographicProjection::default_2d()
        }),
        FixedSceneProjectionPlatform::Desktop { resolution_scale } => {
            Projection::Orthographic(OrthographicProjection {
                scale: fixed_scene_camera_zoom / resolution_scale as f32,
                ..OrthographicProjection::default_2d()
            })
        }
    }
}

/// Sets up the fixed-scene input manager entity with ActionState for handling UI navigation.
/// Uses input configuration from PlayerInputSettings resource.
///
/// 设置 Battle 输入管理器实体，用于处理 UI 导航的 ActionState。
/// 使用来自 PlayerInputSettings 资源的输入配置。
fn setup_fixed_scene_input_manager(
    mut commands: Commands,
    player_input_settings: Res<PlayerInputSettings>,
    sequence_mode: Res<SequenceMode>,
) {
    let Some(mode_name) = sequence_mode.0.as_deref() else {
        warn!("FixedScene: no active mode while setting up input manager.");
        return;
    };
    let input_map = player_input_settings.get_merged_map();

    commands.spawn((
        FixedSceneInputManager,
        fixed_scene_scoped(mode_name),
        input_map,
        ActionState::<Action>::default(),
        Name::new("Fixed Scene Input Manager"),
    ));

    info!("[FixedScene] Input manager spawned with project input configuration");
}

/// Reactivates the main game camera when exiting fixed-scene mode.
///
/// 退出 fixed-scene mode时恢复主游戏相机。
fn cleanup_fixed_scene_camera(
    mut q_game_cameras: Query<
        &mut Camera,
        (
            With<Camera2d>,
            With<crate::core::camera::MainGameCamera>,
            Without<FixedSceneCamera>,
        ),
    >,
) {
    for mut camera in q_game_cameras.iter_mut() {
        camera.is_active = true;
    }
}

/// Cleans up the fixed-scene input manager when exiting fixed-scene mode.
///
/// 退出 fixed-scene mode时清理 Battle 输入管理器。
fn cleanup_fixed_scene_input_manager(
    mut commands: Commands,
    query: Query<Entity, With<FixedSceneInputManager>>,
) {
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::camera::CameraProjection;

    fn visible_size(mut projection: Projection, viewport_width: f32, viewport_height: f32) -> Vec2 {
        let Projection::Orthographic(ref mut orthographic) = projection else {
            panic!("expected orthographic projection");
        };
        orthographic.update(viewport_width, viewport_height);
        Vec2::new(
            orthographic.area.width().abs(),
            orthographic.area.height().abs(),
        )
    }

    #[test]
    fn android_fixed_scene_projection_applies_battle_zoom_to_fixed_base_resolution() {
        let size = visible_size(
            fixed_scene_projection(
                &crate::config::RenderConfig::default(),
                2.0,
                FixedSceneProjectionPlatform::Android,
            ),
            1920.0,
            1080.0,
        );

        assert_eq!(size, Vec2::new(640.0, 480.0));
    }

    #[test]
    fn desktop_fixed_scene_projection_keeps_existing_resolution_scale_mapping() {
        let size = visible_size(
            fixed_scene_projection(
                &crate::config::RenderConfig::default(),
                2.0,
                FixedSceneProjectionPlatform::Desktop {
                    resolution_scale: 2,
                },
            ),
            640.0,
            480.0,
        );

        assert_eq!(size, Vec2::new(640.0, 480.0));
    }
}
