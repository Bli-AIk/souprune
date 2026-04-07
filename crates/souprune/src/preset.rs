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
pub mod battle_box;
mod battle_box_chapter;
pub mod battle_player;
mod battle_player_spawn;
pub mod battle_runtime;
mod data_paths;
pub mod enemy;
pub mod item;
pub(crate) mod item_actions;
mod load_enemies_chapter;
pub mod overworld;

use crate::core::fre_bridge::ViewActionExtensions;
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

        // Register item action handlers (UseItem, CheckItem, DropItem)
        register_item_action_extensions(app);

        // Register LoadEnemies and BattleBox chapter handlers in the sequencer's system set.
        let schedule = crate::game_schedule(app);
        app.add_systems(
            schedule,
            (
                load_enemies_chapter::process_load_enemies_chapter_system,
                load_enemies_chapter::complete_load_enemies_chapter_system,
                battle_box_chapter::process_battle_box_chapter_system,
            )
                .chain()
                .in_set(crate::core::sequencer::SequencerUpdate)
                .after(crate::core::sequencer::load_default_chapter_system),
        );

        // Register tag-based component injection systems.
        app.add_systems(
            schedule,
            (
                battle_box::apply_battle_box_tag_system,
                battle_player_spawn::process_battle_player_spawn_system
                    .in_set(crate::core::sequencer::SequencerUpdate),
                battle_player_spawn::process_player_spawn_requests
                    .in_set(crate::core::sequencer::SequencerUpdate),
            ),
        );
    }
}

/// Register item-specific Custom action handlers (UseItem, CheckItem, DropItem).
fn register_item_action_extensions(app: &mut App) {
    let mut extensions = app
        .world_mut()
        .get_resource_or_insert_with(ViewActionExtensions::default);

    extensions.register("UseItem", |params, ctx| {
        let index_expr = params.get("index_expr").map(|s| s.as_str()).unwrap_or("");
        let start_dialogue = params.get("start_dialogue").is_some_and(|v| v == "true");
        item_actions::execute_use_item(
            index_expr,
            ctx.local_facts,
            ctx.global_facts,
            ctx.audio,
            ctx.asset_server,
            ctx.enum_registry,
            start_dialogue,
            &ctx.config.game.dialogue_view_default,
            &ctx.config.game.dialogue_voice_default,
        );
    });

    extensions.register("CheckItem", |params, ctx| {
        let index_expr = params.get("index_expr").map(|s| s.as_str()).unwrap_or("");
        item_actions::execute_check_item(
            index_expr,
            ctx.local_facts,
            ctx.global_facts,
            ctx.enum_registry,
            &ctx.config.game.dialogue_view_default,
            &ctx.config.game.dialogue_voice_default,
        );
    });

    extensions.register("DropItem", |params, ctx| {
        let index_expr = params.get("index_expr").map(|s| s.as_str()).unwrap_or("");
        item_actions::execute_drop_item(
            index_expr,
            ctx.local_facts,
            ctx.global_facts,
            ctx.enum_registry,
            &ctx.config.game.dialogue_view_default,
            &ctx.config.game.dialogue_voice_default,
        );
    });
}
