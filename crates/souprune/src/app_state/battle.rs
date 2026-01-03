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

pub mod chapter;
pub mod collision;
pub mod config;
pub mod danmaku;
mod sequencer;

use crate::app_state::AppState;
use crate::app_state::battle::chapter::Chapter;
use crate::app_state::battle::collision::BattleCollisionPlugin;
use crate::app_state::battle::config::BattlePlayerConfig;
use crate::app_state::battle::danmaku::DanmakuPlugin;
use crate::app_state::battle::sequencer::SequencerPlugin;
use crate::core::ron_loader::RonAssetLoader;
use bevy::app::{App, Plugin, Update};
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

/// Marker component for overworld entities
///
/// 标记 Battle 实体的组件
#[derive(Component)]
pub(crate) struct BattleEntity;

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

pub(crate) struct BattlePlugin;

impl Plugin for BattlePlugin {
    fn build(&self, app: &mut App) {
        app.configure_sets(Update, BattleUpdate.run_if(in_state(AppState::Battle)))
            .configure_sets(Update, BattleMovementSet.in_set(BattleUpdate))
            // Note: UIUpdate run_if condition is configured in lib.rs to support both Overworld and Battle
            //
            // 注意：UIUpdate 的运行条件在 lib.rs 中配置，以支持 Overworld 和 Battle 两个状态
            .init_asset::<BattleAsset>()
            .register_asset_loader(RonAssetLoader::<BattleAsset>::new(&["battle.ron"]))
            .init_asset::<BattlePlayerConfig>()
            .register_asset_loader(RonAssetLoader::<BattlePlayerConfig>::new(&[
                "battle_player.ron",
            ]))
            .add_plugins((SequencerPlugin, BattleCollisionPlugin, DanmakuPlugin))
            .add_systems(OnEnter(AppState::Battle), setup_battle_camera);
    }
}

fn setup_battle_camera(
    mut commands: Commands,
    q_cameras: Query<Entity, (With<Camera2d>, Without<BattleCamera>)>,
) {
    for camera_entity in q_cameras.iter() {
        commands.entity(camera_entity).despawn();
    }

    commands.spawn((
        Camera2d,
        Projection::Orthographic(OrthographicProjection {
            scale: 1.0,
            ..OrthographicProjection::default_2d()
        }),
        BattleCamera,
        BattleEntity,
        Name::new("Battle Camera2d"),
    ));
}

#[derive(Asset, TypePath, Debug, Clone, Deserialize, Serialize)]
#[serde(transparent)]
pub struct BattleAsset(pub Vec<Chapter>);
