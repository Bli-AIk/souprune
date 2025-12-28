//! # danmaku.rs
//!
//! ## Module Overview
//!
//! ## 模块概述
//!
//! Core danmaku (bullet pattern) system for the battle module.
//! Provides data-driven infrastructure for spawning, updating, and managing bullets.
//! Supports both V1 (Blueprint) and V2 (Performance) architectures.
//!
//! 战斗模块的核心弹幕系统。
//! 提供数据驱动的生成、更新和管理弹幕的基础设施。
//! 同时支持 V1（蓝图）和 V2（演出）架构。

mod components;
mod patterns;
mod systems;

pub use components::*;
pub use patterns::*;

use crate::app_state::battle::BattleUpdate;
use crate::core::ron_loader::RonAssetLoader;
use bevy::prelude::*;

pub struct DanmakuPlugin;

impl Plugin for DanmakuPlugin {
    fn build(&self, app: &mut App) {
        // === V1 Assets (Legacy Blueprint System) ===
        app.init_asset::<DanmakuBlueprint>()
            .register_asset_loader(RonAssetLoader::<DanmakuBlueprint>::new(&["danmaku.ron"]))
            .init_resource::<PendingBlueprintLoads>()
            .add_message::<SpawnPatternEvent>();

        // === V2 Assets (Performance Timeline System) ===
        app.init_asset::<DanmakuPerformance>()
            .register_asset_loader(RonAssetLoader::<DanmakuPerformance>::new(&[
                "performance.ron",
            ]))
            .init_resource::<PendingPerformanceLoads>()
            .add_message::<PlayPerformanceEvent>();

        // === Register Reflect Types ===
        app.register_type::<DanmakuPerformance>()
            .register_type::<BulletPrototype>()
            .register_type::<ColliderShape>()
            .register_type::<BulletBehavior>()
            .register_type::<LinearConfig>()
            .register_type::<HomingConfig>()
            .register_type::<CircularConfig>()
            .register_type::<SineConfig>()
            .register_type::<TweenConfig>()
            .register_type::<TweenTarget>()
            .register_type::<TimelineEvent>()
            .register_type::<SpawnPatternV2>();

        // === Systems ===
        app.add_systems(
            Update,
            (
                // V1 Systems (Legacy)
                systems::process_spawn_pattern_events,
                systems::spawn_bullets_from_blueprints,
                systems::update_bullet_motion,
                // V2 Systems (New)
                systems::process_play_performance_events,
                systems::spawn_performance_players,
                systems::advance_performance_timeline,
                systems::update_bullet_motion_v2,
                // Shared Systems
                systems::update_bullet_lifetime,
                systems::cleanup_dead_bullets,
            )
                .chain()
                .in_set(BattleUpdate),
        );
    }
}
