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

pub mod am_integration;
pub mod collision;
pub mod danmaku;
pub mod fre;
pub mod player_config_schema;

use crate::app_state::AppState;
use crate::app_state::battle::am_integration::AmBattlePlugin;
use crate::app_state::battle::collision::BattleCollisionPlugin;
use crate::app_state::battle::danmaku::DanmakuPlugin;
use crate::app_state::battle::fre::BattleFREPlugin;
use crate::app_state::battle::player_config_schema::BattlePlayerConfig;
use crate::core::sequencer::SequencerPlugin;
use crate::core::input::{Action, PlayerInputSettings};
use crate::core::ron_loader::RonAssetLoader;
use bevy::app::{App, Plugin, Update};
use bevy::prelude::*;
use leafwing_input_manager::action_state::ActionState;

/// Marker component for battle entities
///
/// 标记 Battle 实体的组件
#[derive(Component)]
pub(crate) struct BattleEntity;

/// Marker component for the Battle UI root entity.
///
/// Battle UI 根实体的标记组件。
///

#[derive(Component)]
pub struct BattleViewRoot;

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct BattleUpdate;

/// System set for battle movement (mod behaviors).
/// Collision systems should run after this.
///
/// Battle移动系统集（mod行为）。
/// 碰撞系统应在此之后运行。
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct BattleMovementSet;

#[derive(Component)]
pub struct BattleCamera;

/// Marker component for the Battle input manager entity.
/// This entity holds the `ActionState<Action>` for Battle mode input handling.
///
/// Battle 输入管理器实体的标记组件。
/// 该实体持有 Battle 模式下输入处理所需的 `ActionState<Action>`。
#[derive(Component)]
pub struct BattleInputManager;

pub(crate) struct BattlePlugin;

impl Plugin for BattlePlugin {
    fn build(&self, app: &mut App) {
        app.configure_sets(Update, BattleUpdate.run_if(in_state(AppState::Battle)))
            .configure_sets(Update, BattleMovementSet.in_set(BattleUpdate))
            // Note: SequencerUpdate is configured in lib.rs to support both Overworld and Battle
            // Note: ViewUpdate run_if condition is configured in lib.rs to support both Overworld and Battle
            //
            // 注意：ViewUpdate 的运行条件在 lib.rs 中配置，以支持 Overworld 和 Battle 两个状态
            .init_asset::<BattlePlayerConfig>()
            .register_asset_loader(RonAssetLoader::<BattlePlayerConfig>::new(&[
                "battle_player.ron",
            ]))
            .add_plugins((
                SequencerPlugin,
                BattleCollisionPlugin,
                DanmakuPlugin,
                AmBattlePlugin,
                BattleFREPlugin,
            ))
            .add_systems(
                OnEnter(AppState::Battle),
                (
                    crate::core::sequencer::load_default_chapter_system,
                    setup_battle_camera,
                    setup_battle_input_manager,
                ),
            )
            .add_systems(OnExit(AppState::Battle), cleanup_battle_input_manager);
    }
}

fn setup_battle_camera(
    mut commands: Commands,
    q_cameras: Query<Entity, (With<Camera2d>, Without<BattleCamera>)>,
    resolution_scale: Res<crate::app_state::app_setup::ResolutionScale>,
) {
    let scale_value = resolution_scale.get();
    info!(
        "[Battle] Setting up battle camera with resolution_scale={}, camera_scale={}",
        scale_value,
        1.0 / scale_value as f32
    );

    for camera_entity in q_cameras.iter() {
        commands.entity(camera_entity).despawn();
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

    commands.spawn((
        Camera2d,
        projection,
        BattleCamera,
        BattleEntity,
        Name::new("Battle Camera2d"),
    ));
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
        BattleEntity,
        input_map,
        ActionState::<Action>::default(),
        Name::new("Battle Input Manager"),
    ));

    info!("[Battle] Input manager spawned with MOD configuration");
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


