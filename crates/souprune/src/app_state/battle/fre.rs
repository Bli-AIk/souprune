//! # fre.rs
//!
//! Battle FRE (Fact-Rule-Event) integration module.
//!
//! 战斗 FRE（事实-规则-事件）集成模块。
//!
//! This module provides FRE integration for the battle system, enabling
//! data-driven battle logic through declarative rules.
//!
//! 本模块为战斗系统提供 FRE 集成，通过声明式规则实现数据驱动的战斗逻辑。

mod action_handlers;
mod bridge;

use crate::app_state::AppState;
use crate::app_state::battle::BattleUpdate;
use crate::core::data::PlayerData;
use bevy::prelude::*;
use bevy_fact_rule_event::LayeredFactDatabase;

pub use action_handlers::{apply_pending_damage_system, setup_battle_action_handlers_system};
pub use bridge::{
    ChapterCompletedEvent, SelectionConfirmedEvent, emit_chapter_completed_events_system,
    emit_selection_confirmed_events_system,
};

/// System set for Battle FRE processing.
///
/// 战斗 FRE 处理的系统集。
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct BattleFRESet;

/// Plugin for Battle FRE integration.
///
/// 战斗 FRE 集成插件。
pub struct BattleFREPlugin;

impl Plugin for BattleFREPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<ChapterCompletedEvent>()
            .add_message::<SelectionConfirmedEvent>()
            .configure_sets(Update, BattleFRESet.in_set(BattleUpdate))
            .add_systems(
                OnEnter(AppState::Battle),
                (setup_battle_fre_system, setup_battle_action_handlers_system),
            )
            .add_systems(OnExit(AppState::Battle), cleanup_battle_fre_system)
            .add_systems(
                Update,
                (
                    emit_chapter_completed_events_system,
                    emit_selection_confirmed_events_system,
                    apply_pending_damage_system,
                    sync_player_data_to_fre_system,
                )
                    .in_set(BattleFRESet),
            );
    }
}

/// System to initialize FRE state when entering battle.
/// Clears local layer and sets up initial battle facts from PlayerData.
///
/// 进入战斗时初始化 FRE 状态的系统。
/// 清空局部层并从 PlayerData 设置初始战斗事实。
fn setup_battle_fre_system(
    mut layered_db: ResMut<LayeredFactDatabase>,
    player_data: Res<PlayerData>,
) {
    // Clear local layer from any previous state
    layered_db.clear_local();

    // Set initial battle facts (local layer)
    layered_db.set("battle_is_active", true);
    layered_db.set("battle_turn_count", 0i64);
    layered_db.set("battle_phase", "initializing");

    // Sync player data to facts (global layer - these persist across states)
    layered_db.set_global("player_name", player_data.name.clone());
    layered_db.set_global("player_lv", player_data.lv as i64);
    layered_db.set_global("player_hp", player_data.hp as i64);
    layered_db.set_global("player_hp_max", player_data.hp_max as i64);
    layered_db.set_global("player_atk", player_data.attack as i64);
    layered_db.set_global("player_def", player_data.defense as i64);
    layered_db.set_global("player_gold", player_data.gold as i64);

    info!(
        "Battle FRE: Initialized with player HP {}/{}",
        player_data.hp, player_data.hp_max
    );
}

/// System to sync PlayerData changes to FRE during battle.
/// This keeps the global facts in sync with the ECS PlayerData resource.
///
/// 战斗中同步 PlayerData 变化到 FRE 的系统。
/// 这使全局事实与 ECS PlayerData 资源保持同步。
fn sync_player_data_to_fre_system(
    player_data: Res<PlayerData>,
    mut layered_db: ResMut<LayeredFactDatabase>,
) {
    if !player_data.is_changed() {
        return;
    }

    // Update global facts when PlayerData changes
    layered_db.set_global("player_hp", player_data.hp as i64);
    layered_db.set_global("player_hp_max", player_data.hp_max as i64);
    layered_db.set_global("player_atk", player_data.attack as i64);
    layered_db.set_global("player_def", player_data.defense as i64);
    layered_db.set_global("player_gold", player_data.gold as i64);
    layered_db.set_global("player_lv", player_data.lv as i64);

    trace!("Battle FRE: Synced PlayerData to FRE facts");
}

/// System to clean up FRE state when exiting battle.
/// Optionally promotes important facts to global layer.
///
/// 退出战斗时清理 FRE 状态的系统。
/// 可选地将重要事实提升到全局层。
fn cleanup_battle_fre_system(mut layered_db: ResMut<LayeredFactDatabase>) {
    // TODO: Optionally promote certain facts to global layer
    // (e.g., battle results, experience gained)

    // Clear local layer
    layered_db.clear_local();

    info!("Battle FRE: Cleaned up local layer");
}
