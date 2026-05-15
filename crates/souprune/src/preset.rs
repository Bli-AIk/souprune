//! # preset — Game-Specific Plugins
//!
//! Contains Undertale/Deltarune-specific data types, loaders, and systems
//! that are NOT part of the generic framework core.
//!
//! 包含 Undertale/Deltarune 特有的数据类型、加载器和系统，
//! 这些不属于通用框架核心。
//!
//! In a multi-preset architecture, different game presets would provide
//! their own implementations of these modules.

pub mod battle;
mod data_paths;
pub mod enemy;
pub mod item;
mod load_enemies_chapter;
pub mod overworld;

use crate::core::view::ron_view::parsing::{
    ConditionResolvers, DataPathResolvers, ExprFunctionResolvers,
};
use bevy::app::*;
use bevy::prelude::IntoScheduleConfigs;

/// Plugin that registers game-specific data loaders and sequencer chapter handlers.
///
/// 注册游戏特有的数据加载器和序列器章节处理器的插件。
pub struct PresetPlugin;

impl Plugin for PresetPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            enemy::EnemyPlugin,
            item::ItemPlugin,
            battle::BattlePlugin,
            overworld::OverworldPlugin,
        ));

        // Register computed data path resolvers
        let mut data_resolvers = DataPathResolvers::default();
        data_paths::register_data_path_resolvers(&mut data_resolvers);
        app.insert_resource(data_resolvers);

        // Register condition resolvers
        let mut cond_resolvers = ConditionResolvers::default();
        data_paths::register_condition_resolvers(&mut cond_resolvers);
        app.insert_resource(cond_resolvers);

        // Register expression function resolvers
        let mut expr_func_resolvers = ExprFunctionResolvers::default();
        data_paths::register_expr_function_resolvers(&mut expr_func_resolvers);
        app.insert_resource(expr_func_resolvers);

        // Register LoadEnemies in the sequencer's system set.
        let schedule = crate::game_schedule(app);
        app.add_systems(
            schedule,
            (
                load_enemies_chapter::process_load_enemies_chapter_system,
                load_enemies_chapter::complete_load_enemies_chapter_system,
            )
                .chain()
                .in_set(crate::core::sequencer::SequencerUpdate)
                .after(crate::core::sequencer::load_default_chapter_system),
        );
    }
}
