//! Schema-backed content asset loading and fact projection.
//!
//! 基于 schema 的内容资源加载与事实投影。
//!
//! Registers the framework-provided content asset wrappers and the generic
//! projection glue that turns loaded data into facts consumed by View, FRE, and
//! project WASM runtimes. Gameplay commands that mutate that data remain in
//! project runtimes.
//!
//! 注册框架提供的内容资源包装器，以及把已加载数据投影为 facts 的通用胶水，
//! 供 View、FRE 与 project WASM runtime 消费。会修改这些数据的玩法命令仍位于
//! project runtime 中。

pub mod enemy;
pub mod item;

mod data_paths;
mod load_enemies_chapter;

use crate::core::view::ron_view::parsing::{
    ConditionResolvers, DataPathResolvers, ExprFunctionResolvers,
};
use bevy::app::*;
use bevy::prelude::IntoScheduleConfigs;

/// Plugin for schema-backed content assets and derived fact projection.
///
/// 基于 schema 的内容资源与派生 fact 投影插件。
pub struct ContentPlugin;

impl Plugin for ContentPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((enemy::EnemyPlugin, item::ItemPlugin));

        let mut data_resolvers = DataPathResolvers::default();
        data_paths::register_data_path_resolvers(&mut data_resolvers);
        app.insert_resource(data_resolvers);

        let mut cond_resolvers = ConditionResolvers::default();
        data_paths::register_condition_resolvers(&mut cond_resolvers);
        app.insert_resource(cond_resolvers);

        let mut expr_func_resolvers = ExprFunctionResolvers::default();
        data_paths::register_expr_function_resolvers(&mut expr_func_resolvers);
        app.insert_resource(expr_func_resolvers);

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
