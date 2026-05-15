//! # battle.rs
//!
//! # battle.rs 文件
//!
//! ## Module Overview
//!
//! ## 模块概述
//!
//! Battle module manages the game's battle system.
//!
//! The battle system is centered around a "linear sequence" concept,
//! where any STG gameplay can be abstracted as a linear sequence.
//! Minimal unit of the linear sequence is a "Chapter".
//! The framework code part should only include this definition.
//! Other definitions should be considered as abstractions or supersets of Chapter.
//! For UT/DR games, it manifests as players and enemies taking turns to perform actions until the battle ends.
//! For more complex STG games, the linear sequence can manifest as more complex mechanisms.
//!
//! Battle 模块 负责管理游戏的战斗系统。
//!
//! 战斗系统是一个以 “线性序列” 为核心的系统，任何 STG 玩法均可以抽象为线性序列。
//! 线性序列的最小单位为 “Chapter”。框架代码部分应该只包含这个定义。其他定义应该视为 Chapter 的抽象或超集。
//! 对于 UT/DR 游戏，表现为玩家和敌人轮流进行动作，直到战斗结束。
//! 对于更复杂的 STG 游戏，线性序列可以表现为更复杂的机制。

pub mod alight_motion_integration;
pub mod danmaku;
pub mod fre;
pub mod menu_state;
pub mod speech_bubble;

use crate::core::battle_runtime::{
    BattleCamera, BattleInputManager, BattleMovementSet, BattleUpdate,
};
use crate::core::input::{Action, PlayerInputSettings};
use crate::core::mode::{ModeChanged, ModeScoped, is_mode};
use crate::core::sequencer::SequencerPlugin;
use crate::preset::battle::alight_motion_integration::AlightMotionBattlePlugin;
use crate::preset::battle::danmaku::DanmakuPlugin;
use crate::preset::battle::fre::BattleFREPlugin;
use bevy::app::{App, Plugin};
use bevy::ecs::message::MessageReader;
use bevy::prelude::*;
use leafwing_input_manager::action_state::ActionState;

/// 创建 `ModeScoped("battle")` 标记的便捷方法。
pub(crate) fn battle_scoped() -> ModeScoped {
    ModeScoped("battle".to_string())
}

/// Helper: returns true when entering a specific mode (for run_if conditions).
fn on_entering_battle(mut events: MessageReader<ModeChanged>) -> bool {
    events.read().any(|e| e.to.as_deref() == Some("battle"))
}

fn on_exiting_battle(mut events: MessageReader<ModeChanged>) -> bool {
    events.read().any(|e| e.from.as_deref() == Some("battle"))
}

pub struct BattlePlugin;

impl Plugin for BattlePlugin {
    fn build(&self, app: &mut App) {
        // Register this mode in the global ModeRegistry.
        app.world_mut()
            .get_resource_or_init::<crate::core::mode::ModeRegistry>()
            .register("battle");

        let schedule = crate::game_schedule(app);
        app.configure_sets(schedule, BattleUpdate.run_if(is_mode("battle")))
            .configure_sets(schedule, BattleMovementSet.in_set(BattleUpdate))
            .add_plugins((
                SequencerPlugin,
                DanmakuPlugin,
                AlightMotionBattlePlugin,
                BattleFREPlugin,
                speech_bubble::BattleSpeechBubblePlugin,
            ))
            .add_systems(
                schedule,
                (
                    crate::core::sequencer::load_default_chapter_system,
                    setup_battle_camera,
                    setup_battle_input_manager,
                )
                    .run_if(on_entering_battle),
            )
            .add_systems(
                schedule,
                (cleanup_battle_camera, cleanup_battle_input_manager).run_if(on_exiting_battle),
            );
    }
}

fn setup_battle_camera(
    mut commands: Commands,
    mut q_game_cameras: Query<
        (Entity, &mut Camera),
        (
            With<Camera2d>,
            With<crate::core::camera::MainGameCamera>,
            Without<BattleCamera>,
        ),
    >,
    q_render_targets: Query<&bevy::camera::RenderTarget>,
    resolution_scale: Res<crate::core::camera::ResolutionScale>,
    config: Res<crate::config::SoupruneConfig>,
) {
    let scale_value = resolution_scale.get();
    let zoom = config.game.battle_camera_zoom;
    info!(
        "[Battle] Setting up battle camera with resolution_scale={}, zoom={}, camera_scale={}",
        scale_value,
        zoom,
        zoom / scale_value as f32
    );

    // Capture render target and camera order before deactivating the main game camera,
    // so the battle camera inherits them (e.g. editor texture render target).
    let mut inherited_order: isize = 0;
    let mut inherited_target: Option<bevy::camera::RenderTarget> = None;

    for (entity, mut camera) in q_game_cameras.iter_mut() {
        inherited_order = camera.order;
        inherited_target = q_render_targets.get(entity).ok().cloned();
        camera.is_active = false;
    }

    #[cfg(target_os = "android")]
    let projection = battle_projection(&config.render, zoom, BattleProjectionPlatform::Android);
    #[cfg(not(target_os = "android"))]
    let projection = battle_projection(
        &config.render,
        zoom,
        BattleProjectionPlatform::Desktop {
            resolution_scale: scale_value,
        },
    );

    info!(
        "[Battle] Inherited camera settings: order={}, has_render_target={}",
        inherited_order,
        inherited_target.is_some()
    );

    let mut battle_cam = commands.spawn((
        Camera2d,
        Camera {
            order: inherited_order,
            ..default()
        },
        // Explicitly position at world origin — all battle View nodes and game entities
        // use world-space coordinates relative to (0, 0, 0).
        Transform::from_translation(Vec3::ZERO),
        projection,
        BattleCamera,
        crate::core::camera::MainGameCamera,
        IsDefaultUiCamera,
        battle_scoped(),
        Name::new("Battle Camera2d"),
    ));
    if let Some(target) = inherited_target {
        battle_cam.insert(target);
    }
}

#[derive(Clone, Copy)]
enum BattleProjectionPlatform {
    Android,
    Desktop { resolution_scale: u32 },
}

fn battle_projection(
    render_config: &crate::config::RenderConfig,
    battle_camera_zoom: f32,
    platform: BattleProjectionPlatform,
) -> Projection {
    match platform {
        BattleProjectionPlatform::Android => Projection::Orthographic(OrthographicProjection {
            scaling_mode: bevy::camera::ScalingMode::Fixed {
                width: render_config.base_resolution_width as f32,
                height: render_config.base_resolution_height as f32,
            },
            scale: battle_camera_zoom,
            ..OrthographicProjection::default_2d()
        }),
        BattleProjectionPlatform::Desktop { resolution_scale } => {
            Projection::Orthographic(OrthographicProjection {
                scale: battle_camera_zoom / resolution_scale as f32,
                ..OrthographicProjection::default_2d()
            })
        }
    }
}

/// Sets up the Battle input manager entity with ActionState for handling UI navigation.
/// Uses input configuration from PlayerInputSettings resource.
///
/// 设置 Battle 输入管理器实体，用于处理 UI 导航的 ActionState。
/// 使用来自 PlayerInputSettings 资源的输入配置。
fn setup_battle_input_manager(
    mut commands: Commands,
    player_input_settings: Res<PlayerInputSettings>,
) {
    let input_map = player_input_settings.get_merged_map();

    commands.spawn((
        BattleInputManager,
        battle_scoped(),
        input_map,
        ActionState::<Action>::default(),
        Name::new("Battle Input Manager"),
    ));

    info!("[Battle] Input manager spawned with MOD configuration");
}

/// Reactivates the main game camera when exiting Battle state.
///
/// 退出 Battle 状态时恢复主游戏相机。
fn cleanup_battle_camera(
    mut q_game_cameras: Query<
        &mut Camera,
        (
            With<Camera2d>,
            With<crate::core::camera::MainGameCamera>,
            Without<BattleCamera>,
        ),
    >,
) {
    for mut camera in q_game_cameras.iter_mut() {
        camera.is_active = true;
    }
}

/// Cleans up the Battle input manager when exiting Battle state.
///
/// 退出 Battle 状态时清理 Battle 输入管理器。
fn cleanup_battle_input_manager(
    mut commands: Commands,
    query: Query<Entity, With<BattleInputManager>>,
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
    fn android_battle_projection_applies_battle_zoom_to_fixed_base_resolution() {
        let size = visible_size(
            battle_projection(
                &crate::config::RenderConfig::default(),
                2.0,
                BattleProjectionPlatform::Android,
            ),
            1920.0,
            1080.0,
        );

        assert_eq!(size, Vec2::new(640.0, 480.0));
    }

    #[test]
    fn desktop_battle_projection_keeps_existing_resolution_scale_mapping() {
        let size = visible_size(
            battle_projection(
                &crate::config::RenderConfig::default(),
                2.0,
                BattleProjectionPlatform::Desktop {
                    resolution_scale: 2,
                },
            ),
            640.0,
            480.0,
        );

        assert_eq!(size, Vec2::new(640.0, 480.0));
    }
}
