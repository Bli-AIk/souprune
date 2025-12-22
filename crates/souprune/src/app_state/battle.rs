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

mod chapter;
mod sequencer;

use crate::app_state::battle::chapter::Chapter;
use crate::app_state::battle::sequencer::SequencerPlugin;
use crate::app_state::{AppState, cleanup_entities_system};
use crate::core::ron_loader::RonAssetLoader;
use bevy::app::{App, Plugin, Update};
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

/// Marker component for overworld entities
///
/// 标记 Battle 实体的组件
#[derive(Component)]
pub(crate) struct BattleEntity();

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct BattleUpdate;

pub(crate) struct BattlePlugin;

impl Plugin for BattlePlugin {
    fn build(&self, app: &mut App) {
        app.configure_sets(Update, BattleUpdate.run_if(in_state(AppState::Battle)))
            .init_asset::<BattleFlowAsset>()
            .register_asset_loader(RonAssetLoader::<BattleFlowAsset>::new(&["chapter.ron"]))
            .add_plugins(SequencerPlugin)
            .add_systems(
                OnExit(AppState::Battle),
                cleanup_entities_system::<BattleEntity>,
            );
    }
}

#[derive(Asset, TypePath, Debug, Clone, Deserialize, Serialize)]
#[serde(transparent)]
pub struct BattleFlowAsset(pub Vec<Chapter>);
