//! # preset — Game-Specific Plugins
//!
//! Contains Undertale/Deltarune-specific data types, loaders, and systems
//! that are NOT part of the generic engine core.
//!
//! 包含 Undertale/Deltarune 特有的数据类型、加载器和系统，
//! 这些不属于通用引擎核心。
//!
//! In a multi-preset architecture, different game presets would provide
//! their own implementations of these modules.

pub mod battle_box;
pub mod battle_player;
pub mod battle_runtime;
pub mod enemy;
pub mod item;
mod load_enemies_chapter;

use bevy::app::*;
use bevy::prelude::IntoScheduleConfigs;

/// Plugin that registers game-specific data loaders and sequencer chapter handlers.
///
/// 注册游戏特有的数据加载器和序列器章节处理器的插件。
pub struct PresetPlugin;

impl Plugin for PresetPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((enemy::EnemyPlugin, item::ItemPlugin));

        // Register LoadEnemies chapter handler in the sequencer's system set.
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
