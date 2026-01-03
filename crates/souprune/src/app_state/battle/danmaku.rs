//! # danmaku.rs
//!
//! ## Module Overview
//!
//! ## 模块概述
//!
//! Battle-specific danmaku configuration.
//! This module configures the core danmaku system for battle state.
//!
//! 战斗特定的弹幕配置。
//! 此模块为战斗状态配置核心弹幕系统。

// Re-export core danmaku types for backwards compatibility
pub use crate::core::danmaku::*;

use crate::app_state::AppState;
use crate::core::danmaku::{DanmakuSpawnContext, DanmakuUpdate};
use bevy::prelude::*;

/// Battle-specific danmaku plugin.
/// Configures CoreDanmakuPlugin for battle state.
///
/// 战斗特定的弹幕插件。
/// 为战斗状态配置 CoreDanmakuPlugin。
pub struct DanmakuPlugin;

impl Plugin for DanmakuPlugin {
    fn build(&self, app: &mut App) {
        // Configure DanmakuUpdate run condition:
        // When experimental feature is OFF, only run in Battle state
        // When experimental feature is ON, the run_if is configured in overworld.rs to allow both states
        #[cfg(not(feature = "experimental"))]
        app.configure_sets(Update, DanmakuUpdate.run_if(in_state(AppState::Battle)));

        // Set spawn context to Battle when entering battle state
        app.add_systems(OnEnter(AppState::Battle), set_battle_context);

        // Register reflect types for inspector
        app.register_type::<DanmakuPerformance>()
            .register_type::<BulletPrototype>()
            .register_type::<BulletVisual>()
            .register_type::<ColliderShape>()
            .register_type::<BulletBehavior>()
            .register_type::<LinearConfig>()
            .register_type::<OrbitalConfig>()
            .register_type::<SineConfig>()
            .register_type::<TweenConfig>()
            .register_type::<TweenTarget>()
            .register_type::<Easing>()
            .register_type::<TimelineEvent>()
            .register_type::<SpawnPattern>()
            .register_type::<EdgeSide>();
    }
}

fn set_battle_context(mut spawn_context: ResMut<DanmakuSpawnContext>) {
    *spawn_context = DanmakuSpawnContext::Battle;
    info!("Danmaku: Set spawn context to Battle");
}
