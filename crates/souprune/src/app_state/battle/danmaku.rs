//! # danmaku.rs
//!
//! ## Module Overview
//!
//! ## 模块概述
//!
//! Core danmaku (bullet pattern) system for the battle module.
//! Provides timeline-based infrastructure for spawning, updating, and managing bullets.
//!
//! 战斗模块的核心弹幕系统。
//! 提供基于时间轴的生成、更新和管理弹幕的基础设施。

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
        // === Asset Registration ===
        app.init_asset::<DanmakuPerformance>()
            .register_asset_loader(RonAssetLoader::<DanmakuPerformance>::new(&[
                "performance.ron",
            ]))
            .init_resource::<PendingPerformanceLoads>()
            .add_message::<PlayPerformanceEvent>();

        // === Register Reflect Types ===
        app.register_type::<DanmakuPerformance>()
            .register_type::<BulletPrototype>()
            .register_type::<BulletVisual>()
            .register_type::<ColliderShape>()
            .register_type::<BulletBehavior>()
            .register_type::<LinearConfig>()
            .register_type::<HomingConfig>()
            .register_type::<OrbitalConfig>()
            .register_type::<SineConfig>()
            .register_type::<TweenConfig>()
            .register_type::<TweenTarget>()
            .register_type::<Easing>()
            .register_type::<TimelineEvent>()
            .register_type::<SpawnPattern>()
            .register_type::<EdgeSide>();

        // === Systems ===
        app.add_systems(
            Update,
            (
                systems::process_play_performance_events,
                systems::spawn_performance_players,
                systems::advance_performance_timeline,
                systems::update_bullet_motion,
                systems::update_bullet_lifetime,
                systems::cleanup_dead_bullets,
            )
                .chain()
                .in_set(BattleUpdate),
        );
    }
}
