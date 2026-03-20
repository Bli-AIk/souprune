#![allow(dead_code, unexpected_cfgs)]
//! # lib.rs
//!
//! # lib.rs 文件
//!
//! ## Module Overview
//!
//! ## 模块概述
//!
//! This is the main library entry point for the Souprune framework. It orchestrates the application startup, including logging initialization, plugin registration (Bevy defaults, third-party, and game-specific), and configuring the asset system for multi-source loading.
//!
//! 这是 Souprune 框架的主要库入口点。它负责协调应用程序的启动，包括日志初始化、插件注册（Bevy 默认插件、第三方插件和游戏特定插件），以及配置用于多源加载的资产系统。

pub mod app_state;
mod bootstrap;
pub mod config;
pub mod core;
pub mod editor_api;
pub mod extra;

pub use crate::app_state::overworld::player::config::PlayerBehavior;
pub use crate::bootstrap::{
    get_file_importer_plugins, get_game_plugins, get_third_plugins, insert_font_resources,
    insert_input_resources, reset_game_state, run,
};
pub use crate::core::basic_components::Direction;
pub use crate::core::character_asset::{
    AnimationConfigAsset, CharacterAsset, StateAnimationMapping,
};
pub use crate::core::definition::{CombatStats, LocaleInfo};
pub use crate::core::enemy::{ActionOption, EnemyDef, EnemyRegistry};
pub use crate::core::input::actions::Action;
pub use crate::core::item::{Item, ItemEffect, ItemListAsset, ItemRegistry, ItemType};
pub use crate::core::view::layout::{
    FloatOrExpr, SerializableVec3, ViewBoxLogicDef, ViewLayoutAsset, ViewNodeDef,
};

use std::default::Default;

use bevy::ecs::schedule::{InternedScheduleLabel, ScheduleLabel};
use bevy::prelude::*;

/// 游戏逻辑系统的目标调度器。
///
/// 游戏本体使用 `Update`，编辑器使用 `GameSchedule`（由 bevy_workbench 控制执行时机）。
/// 所有游戏 Plugin 在 `build()` 中读取此资源，将系统注册到指定的调度器。
#[derive(Resource, Clone)]
pub struct GameUpdateSchedule(pub InternedScheduleLabel);

impl Default for GameUpdateSchedule {
    fn default() -> Self {
        Self(Update.intern())
    }
}

/// 从 App 中获取游戏逻辑调度器标签。
/// 如果未设置 `GameUpdateSchedule` 资源，返回 `Update`。
pub fn game_schedule(app: &App) -> InternedScheduleLabel {
    app.world()
        .get_resource::<GameUpdateSchedule>()
        .map_or(Update.intern(), |s| s.0)
}

/// 初始化游戏核心状态。
///
/// 注册 AppState、SequenceSubState、SequenceMode、ModeChanged，
/// 以及 ViewUpdate / SequencerUpdate 系统集的 run_if 条件。
/// 游戏本体和编辑器共用此函数，避免重复初始化代码。
pub fn init_game_state(app: &mut App) {
    let schedule = game_schedule(app);
    app.init_state::<app_state::AppState>()
        .init_state::<app_state::SequenceSubState>()
        .init_resource::<app_state::SequenceMode>()
        .add_message::<app_state::ModeChanged>()
        .add_systems(
            PreUpdate,
            (
                app_state::detect_mode_changes,
                app_state::cleanup_mode_scoped_entities,
            )
                .chain(),
        )
        .configure_sets(
            schedule,
            core::view::ViewUpdate.run_if(in_state(app_state::AppState::Running)),
        )
        .configure_sets(
            schedule,
            core::sequencer::SequencerUpdate.run_if(in_state(app_state::AppState::Running)),
        );
}

#[cfg(target_os = "android")]
#[bevy_main]
fn main() {
    run();
}
