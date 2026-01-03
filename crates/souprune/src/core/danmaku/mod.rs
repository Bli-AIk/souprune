//! # danmaku/mod.rs
//!
//! ## Module Overview
//!
//! Core danmaku (bullet hell) system - generic and state-agnostic.
//! This module provides the foundation for bullet pattern systems that can be
//! used in both Battle and Overworld states.
//!
//! ## 模块概述
//!
//! 核心弹幕系统 - 通用且与状态无关。
//! 此模块提供弹幕模式系统的基础，可用于 Battle 和 Overworld 状态。

mod components;
mod patterns;
mod systems;
mod target;

pub use components::*;
pub use patterns::*;
pub use systems::*;
pub use target::*;

use bevy::asset::AssetApp;
use bevy::prelude::*;

use crate::core::ron_loader::RonAssetLoader;

/// System set for danmaku updates.
/// This set can be included in both BattleUpdate and OverworldUpdate.
///
/// 弹幕更新的系统集。
/// 此集合可以包含在 BattleUpdate 和 OverworldUpdate 中。
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct DanmakuUpdate;

/// The active state for danmaku spawning.
///
/// 弹幕生成的活动状态。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DanmakuActiveState {
    /// Battle state (adds BattleEntity marker)
    #[default]
    Battle,
    /// Overworld state (adds OverworldEntity marker)
    Overworld,
}

/// Resource to configure danmaku spawning context.
/// Determines which entity marker to add to spawned bullets.
///
/// 配置弹幕生成上下文的资源。
/// 决定将哪种实体标记添加到生成的弹幕上。
#[derive(Resource, Debug, Clone, Default)]
pub struct DanmakuSpawnContext {
    /// The active state for spawning (Battle or Overworld)
    pub state: DanmakuActiveState,
}

impl DanmakuSpawnContext {
    /// Create a new context for Battle state.
    pub fn battle() -> Self {
        Self {
            state: DanmakuActiveState::Battle,
        }
    }

    /// Create a new context for Overworld state.
    pub fn overworld() -> Self {
        Self {
            state: DanmakuActiveState::Overworld,
        }
    }
}

/// Core danmaku plugin - provides state-agnostic bullet system.
/// Add this plugin to your app, then configure DanmakuUpdate set for your state.
///
/// 核心弹幕插件 - 提供与状态无关的弹幕系统。
/// 将此插件添加到应用中，然后为你的状态配置 DanmakuUpdate 集合。
pub struct CoreDanmakuPlugin;

impl Plugin for CoreDanmakuPlugin {
    fn build(&self, app: &mut App) {
        app.init_asset::<DanmakuPerformance>()
            .register_asset_loader(RonAssetLoader::<DanmakuPerformance>::new(&[
                "performance.ron",
                "danmaku.ron",
            ]))
            .init_resource::<PendingPerformanceLoads>()
            .init_resource::<DanmakuSpawnContext>()
            .add_message::<PlayPerformanceEvent>()
            .add_systems(
                Update,
                (
                    process_play_performance_events,
                    spawn_performance_players,
                    advance_performance_timeline,
                    update_bullet_motion,
                    update_bullet_lifetime,
                    cleanup_dead_bullets,
                )
                    .chain()
                    .in_set(DanmakuUpdate),
            );
    }
}
