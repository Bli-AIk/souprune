//! # danmaku.rs
//!
//! ## Module Overview
//!
//! ## 模块概述
//!
//! Core danmaku (bullet pattern) system for the battle module.
//! Provides infrastructure for spawning, updating, and managing bullets.
//!
//! 战斗模块的核心弹幕系统。
//! 提供生成、更新和管理弹幕的基础设施。

mod components;
mod patterns;
mod systems;

pub use patterns::*;

use crate::app_state::battle::BattleUpdate;
use bevy::prelude::*;

pub struct DanmakuPlugin;

impl Plugin for DanmakuPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PatternRegistry>()
            .add_message::<SpawnPatternEvent>()
            .add_systems(
                Update,
                (
                    systems::process_spawn_pattern_events,
                    systems::update_bullet_motion,
                    systems::update_bullet_lifetime,
                    systems::cleanup_dead_bullets,
                )
                    .chain()
                    .in_set(BattleUpdate),
            );
    }
}
