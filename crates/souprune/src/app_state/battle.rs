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
pub mod collision;
pub mod danmaku;
pub mod fre;
pub mod player_config_schema;

use crate::app_state::battle::alight_motion_integration::AlightMotionBattlePlugin;
use crate::app_state::battle::collision::BattleCollisionPlugin;
use crate::app_state::battle::danmaku::DanmakuPlugin;
use crate::app_state::battle::fre::BattleFREPlugin;
use crate::app_state::battle::player_config_schema::BattlePlayerConfig;
use crate::app_state::{ModeChanged, ModeScoped, is_mode};
use crate::core::battle_runtime::{
    BattleCamera, BattleInputManager, BattleMovementSet, BattleUpdate,
};
use crate::core::input::{Action, PlayerInputSettings};
use crate::core::ron_loader::RonAssetLoader;
use crate::core::sequencer::SequencerPlugin;
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
        let schedule = crate::game_schedule(app);
        app.configure_sets(schedule, BattleUpdate.run_if(is_mode("battle")))
            .configure_sets(schedule, BattleMovementSet.in_set(BattleUpdate))
            .init_asset::<BattlePlayerConfig>()
            .register_asset_loader(RonAssetLoader::<BattlePlayerConfig>::new(&[
                "battle_player.ron",
            ]))
            .add_plugins((
                SequencerPlugin,
                BattleCollisionPlugin,
                DanmakuPlugin,
                AlightMotionBattlePlugin,
                BattleFREPlugin,
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
) {
    let scale_value = resolution_scale.get();
    info!(
        "[Battle] Setting up battle camera with resolution_scale={}, camera_scale={}",
        scale_value,
        1.0 / scale_value as f32
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

    // On Android, use Fixed scaling (viewport system handles aspect ratio)
    #[cfg(target_os = "android")]
    let projection = Projection::Orthographic(OrthographicProjection {
        scaling_mode: bevy::camera::ScalingMode::Fixed {
            width: 320.0,
            height: 240.0,
        },
        ..OrthographicProjection::default_2d()
    });
    #[cfg(not(target_os = "android"))]
    let projection = Projection::Orthographic(OrthographicProjection {
        scale: 1.0 / scale_value as f32,
        ..OrthographicProjection::default_2d()
    });

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
