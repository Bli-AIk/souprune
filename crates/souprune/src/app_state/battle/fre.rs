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
                )
                    .in_set(BattleFRESet),
            );
    }
}

/// System to initialize FRE state when entering battle.
/// Clears local layer and sets up initial battle facts.
/// Player data is already stored in the global layer by the data.rs module.
///
/// 进入战斗时初始化 FRE 状态的系统。
/// 清空局部层并设置初始战斗事实。
/// 玩家数据已由 data.rs 模块存储在全局层中。
fn setup_battle_fre_system(mut layered_db: ResMut<LayeredFactDatabase>) {
    // Clear local layer from any previous state
    layered_db.clear_local();

    // Set initial battle facts (local layer)
    layered_db.set("battle_is_active", true);
    layered_db.set("battle_turn_count", 0i64);
    layered_db.set("battle_phase", "initializing");

    // Player data is already in global layer (managed by core::data module)
    // No need to sync here

    let hp = layered_db.get_int("player_hp").unwrap_or(20);
    let hp_max = layered_db.get_int("player_hp_max").unwrap_or(20);
    info!("Battle FRE: Initialized with player HP {}/{}", hp, hp_max);
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
