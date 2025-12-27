//! # danmaku.rs
//!
//! ## Module Overview
//!
//! ## 模块概述
//!
//! Core danmaku (bullet pattern) system for the battle module.
//! Provides data-driven infrastructure for spawning, updating, and managing bullets.
//!
//! 战斗模块的核心弹幕系统。
//! 提供数据驱动的生成、更新和管理弹幕的基础设施。

mod components;
mod patterns;
mod systems;

pub use patterns::*;

use crate::app_state::battle::BattleUpdate;
use crate::core::ron_loader::RonAssetLoader;
use bevy::prelude::*;

pub struct DanmakuPlugin;

impl Plugin for DanmakuPlugin {
    fn build(&self, app: &mut App) {
        app.init_asset::<DanmakuBlueprint>()
            .register_asset_loader(RonAssetLoader::<DanmakuBlueprint>::new(&["danmaku.ron"]))
            .init_resource::<PendingBlueprintLoads>()
            .add_message::<SpawnPatternEvent>()
            .add_systems(
                Update,
                (
                    systems::process_spawn_pattern_events,
                    systems::spawn_bullets_from_blueprints,
                    systems::update_bullet_motion,
                    systems::update_bullet_lifetime,
                    systems::cleanup_dead_bullets,
                )
                    .chain()
                    .in_set(BattleUpdate),
            );
    }
}
