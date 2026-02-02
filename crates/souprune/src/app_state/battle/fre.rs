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
use bevy_fact_rule_event::{FactValueDef, LayeredFactDatabase, RuleRegistry, RuleSetAsset};

pub use action_handlers::{apply_pending_damage_system, setup_battle_action_handlers_system};
pub use bridge::{
    ChapterCompletedEvent, SelectionConfirmedEvent, battle_view_navigation_system,
    emit_chapter_completed_events_system, emit_selection_confirmed_events_system,
};

/// System set for Battle FRE processing.
///
/// 战斗 FRE 处理的系统集。
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct BattleFRESet;

/// Resource to track the battle-specific rules handle.
///
/// 跟踪战斗特定规则句柄的资源。
#[derive(Resource, Default)]
pub struct BattleRulesHandle {
    pub handle: Option<Handle<RuleSetAsset>>,
    pub registered: bool,
}

/// Plugin for Battle FRE integration.
///
/// 战斗 FRE 集成插件。
pub struct BattleFREPlugin;

impl Plugin for BattleFREPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<ChapterCompletedEvent>()
            .add_message::<SelectionConfirmedEvent>()
            .init_resource::<BattleRulesHandle>()
            .configure_sets(Update, BattleFRESet.in_set(BattleUpdate))
            .add_systems(
                OnEnter(AppState::Battle),
                (setup_battle_fre_system, setup_battle_action_handlers_system),
            )
            .add_systems(OnExit(AppState::Battle), cleanup_battle_fre_system)
            .add_systems(
                Update,
                (
                    register_battle_rules_system,
                    emit_chapter_completed_events_system,
                    emit_selection_confirmed_events_system,
                    apply_pending_damage_system,
                    // Battle UI navigation - handles input and updates ViewRoot.local_facts
                    // 战斗 UI 导航 - 处理输入并更新 ViewRoot.local_facts
                    battle_view_navigation_system,
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

/// System to register battle-specific rules when loaded.
///
/// 当战斗规则加载完成时注册它们的系统。
fn register_battle_rules_system(
    mut battle_rules_handle: ResMut<BattleRulesHandle>,
    rule_set_assets: Res<Assets<RuleSetAsset>>,
    mut registry: ResMut<RuleRegistry>,
    mut fact_db: ResMut<LayeredFactDatabase>,
) {
    // Skip if already registered or no handle
    if battle_rules_handle.registered {
        return;
    }

    let Some(handle) = &battle_rules_handle.handle else {
        return;
    };

    let Some(rule_set) = rule_set_assets.get(handle) else {
        return;
    };

    // Apply initial facts to Local layer (battle specific)
    for (key, value) in rule_set.get_initial_facts() {
        let fact_value = match value {
            FactValueDef::Int(v) => bevy_fact_rule_event::FactValue::Int(*v),
            FactValueDef::Float(v) => bevy_fact_rule_event::FactValue::Float(*v),
            FactValueDef::Bool(v) => bevy_fact_rule_event::FactValue::Bool(*v),
            FactValueDef::String(v) => bevy_fact_rule_event::FactValue::String(v.clone()),
        };
        fact_db.set_local(key.as_str(), fact_value);
        info!("Battle FRE: Set initial fact '{}' to Local layer", key);
    }

    // Register all rules
    rule_set.register_rules(&mut registry);
    battle_rules_handle.registered = true;
    info!("Battle FRE: Rules registered from battle configuration");
}

/// System to clean up FRE state when exiting battle.
/// Optionally promotes important facts to global layer.
///
/// 退出战斗时清理 FRE 状态的系统。
/// 可选地将重要事实提升到全局层。
fn cleanup_battle_fre_system(
    mut layered_db: ResMut<LayeredFactDatabase>,
    mut battle_rules_handle: ResMut<BattleRulesHandle>,
) {
    // TODO: Optionally promote certain facts to global layer
    // (e.g., battle results, experience gained)

    // Clear local layer
    layered_db.clear_local();

    // Reset rules handle for next battle
    battle_rules_handle.handle = None;
    battle_rules_handle.registered = false;

    info!("Battle FRE: Cleaned up local layer and reset rules handle");
}
