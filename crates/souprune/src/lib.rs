//! This is the public library entry point for SoupRune. It ties together the
//! high-level application state modules, the reusable `core`
//! infrastructure, the editor-facing facade, and the bootstrap helpers that turn
//! those pieces into a runnable Bevy app. Downstream code should treat this file
//! as the stable surface of the main crate rather than reaching into bootstrap
//! internals directly.
//!
//! 这是 SoupRune 的公开库入口。它把高层应用状态模块、可复用的 `core`
//! 基础设施、面向编辑器的门面 API，以及把这些部件组装成可运行 Bevy 应用的
//! bootstrap 辅助函数接到一起。下游代码应把这个文件视为主 crate 的稳定表面，
//! 而不是直接深入 bootstrap 内部路径。
#![allow(dead_code, unexpected_cfgs)]

pub mod app_state;
mod bootstrap;
pub mod config;
pub mod core;
pub mod editor_api;
pub mod extra;
pub mod preset;

pub use crate::bootstrap::{
    get_file_importer_plugins, get_game_plugins, get_third_plugins, insert_font_resources,
    insert_input_resources, reset_game_state, run,
};
pub use crate::core::basic_components::Direction;
pub use crate::core::character_asset::{
    AnimationConfigAsset, CharacterAsset, StateAnimationMapping,
};
pub use crate::core::input::actions::Action;
pub use crate::core::view::layout::{
    FloatOrExpr, SerializableVec3, ViewBoxLogicDef, ViewLayoutAsset, ViewNodeDef,
};
pub use crate::preset::enemy::{ActionOption, EnemyDef, EnemyRegistry};
pub use crate::preset::item::{Item, ItemEffect, ItemListAsset, ItemRegistry, ItemType};
pub use crate::preset::overworld::player::config::PlayerBehavior;
pub use souprune_schema::enemy::{CombatStats, LocaleInfo};

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
// CLA check retry
