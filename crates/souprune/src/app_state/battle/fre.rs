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
//!
//! NOTE: Battle UI navigation has been moved to FRE rules in battle_menu.rules.ron.
//! The old hardcoded navigation system has been removed.
//!
//! 注意：战斗 UI 导航已移至 battle_menu.rules.ron 中的 FRE 规则。
//! 旧的硬编码导航系统已被移除。

mod action_handlers;
mod bridge;

use crate::app_state::AppState;
use crate::app_state::battle::BattleUpdate;
use crate::app_state::overworld::trigger::RuleActionDefs;
use bevy::prelude::*;
use bevy_fact_rule_event::{FactValueDef, LayeredFactDatabase, RuleRegistry, RuleSetAsset};

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

/// Resource to track the battle-specific rules handle.
///
/// 跟踪战斗特定规则句柄的资源。
#[derive(Resource, Default)]
pub struct BattleRulesHandle {
    /// Handle for custom battle rules (from battle asset rules_file)
    pub handle: Option<Handle<RuleSetAsset>>,
    /// Handle for battle menu rules (always loaded)
    pub menu_handle: Option<Handle<RuleSetAsset>>,
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
                    // Note: Battle UI navigation is now handled by FRE rules in battle_menu.rules.ron
                    // The core::fre_bridge::FREBridgePlugin provides ActionEvent-to-FRE conversion
                    // 注意：战斗 UI 导航现在由 battle_menu.rules.ron 中的 FRE 规则处理
                    // core::fre_bridge::FREBridgePlugin 提供 ActionEvent 到 FRE 的转换
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
fn setup_battle_fre_system(
    mut layered_db: ResMut<LayeredFactDatabase>,
    mut battle_rules_handle: ResMut<BattleRulesHandle>,
    asset_server: Res<AssetServer>,
) {
    // Clear local layer from any previous state
    layered_db.clear_local();

    // Set initial battle facts (local layer)
    layered_db.set("battle_is_active", true);
    layered_db.set("battle_turn_count", 0i64);
    layered_db.set("battle_phase", "initializing");

    // Always load battle menu rules
    let menu_path = "battle/rules/battle_menu.rules.ron";
    let menu_handle = asset_server.load::<RuleSetAsset>(menu_path);
    battle_rules_handle.menu_handle = Some(menu_handle);
    info!("Battle FRE: Loading battle menu rules from {}", menu_path);

    // Player data is already in global layer (managed by core::data module)
    // No need to sync here

    let hp = layered_db.get_int("player_hp").unwrap_or(20);
    let hp_max = layered_db.get_int("player_hp_max").unwrap_or(20);
    info!("Battle FRE: Initialized with player HP {}/{}", hp, hp_max);
}

/// System to register battle-specific rules when loaded.
/// Handles both menu rules (always loaded) and custom battle rules (optional).
/// Also populates RuleActionDefs for action execution.
///
/// 当战斗规则加载完成时注册它们的系统。
/// 处理菜单规则（总是加载）和自定义战斗规则（可选）。
/// 同时填充 RuleActionDefs 以执行 action。
fn register_battle_rules_system(
    mut commands: Commands,
    mut battle_rules_handle: ResMut<BattleRulesHandle>,
    rule_set_assets: Res<Assets<RuleSetAsset>>,
    mut registry: ResMut<RuleRegistry>,
    mut fact_db: ResMut<LayeredFactDatabase>,
    existing_action_defs: Option<ResMut<RuleActionDefs>>,
) {
    // Skip if already registered
    if battle_rules_handle.registered {
        return;
    }

    // Wait for menu rules to load (always required)
    let menu_loaded = battle_rules_handle
        .menu_handle
        .as_ref()
        .map(|h| rule_set_assets.get(h).is_some())
        .unwrap_or(false);

    // Wait for custom rules if specified
    let custom_loaded = battle_rules_handle
        .handle
        .as_ref()
        .map(|h| rule_set_assets.get(h).is_some())
        .unwrap_or(true); // true if no custom rules

    if !menu_loaded || !custom_loaded {
        return;
    }

    // Initialize or get RuleActionDefs
    let mut action_defs = match existing_action_defs {
        Some(defs) => defs,
        None => {
            commands.init_resource::<RuleActionDefs>();
            return; // Will run again next frame with the resource available
        }
    };

    // Process menu rules
    if let Some(handle) = &battle_rules_handle.menu_handle {
        if let Some(rule_set) = rule_set_assets.get(handle) {
            // Apply initial facts to View local facts (via ViewRoot in fre_bridge)
            for (key, value) in rule_set.get_initial_facts() {
                let fact_value = match value {
                    FactValueDef::Int(v) => bevy_fact_rule_event::FactValue::Int(*v),
                    FactValueDef::Float(v) => bevy_fact_rule_event::FactValue::Float(*v),
                    FactValueDef::Bool(v) => bevy_fact_rule_event::FactValue::Bool(*v),
                    FactValueDef::String(v) => bevy_fact_rule_event::FactValue::String(v.clone()),
                };
                fact_db.set_local(key.as_str(), fact_value);
                info!("Battle FRE: Set initial fact '{}' from menu rules", key);
            }

            // Register rules and populate action_defs
            let rules_defs = rule_set.get_rule_defs();
            for (idx, rule_def) in rules_defs.iter().enumerate() {
                let rule = rule_def.to_rule_with_index(idx);
                let rule_id = rule_def.generate_id(idx);

                // Store actions by rule ID
                if !rule_def.actions.is_empty() {
                    action_defs
                        .actions_by_rule
                        .insert(rule_id.clone(), rule_def.actions.clone());
                }

                registry.register(rule);
            }
            info!("Battle FRE: Registered {} menu rules", rules_defs.len());
        }
    }

    // Process custom battle rules (if any)
    if let Some(handle) = &battle_rules_handle.handle {
        if let Some(rule_set) = rule_set_assets.get(handle) {
            for (key, value) in rule_set.get_initial_facts() {
                let fact_value = match value {
                    FactValueDef::Int(v) => bevy_fact_rule_event::FactValue::Int(*v),
                    FactValueDef::Float(v) => bevy_fact_rule_event::FactValue::Float(*v),
                    FactValueDef::Bool(v) => bevy_fact_rule_event::FactValue::Bool(*v),
                    FactValueDef::String(v) => bevy_fact_rule_event::FactValue::String(v.clone()),
                };
                fact_db.set_local(key.as_str(), fact_value);
                info!("Battle FRE: Set initial fact '{}' from battle rules", key);
            }

            let rules_defs = rule_set.get_rule_defs();
            let offset = battle_rules_handle
                .menu_handle
                .as_ref()
                .and_then(|h| rule_set_assets.get(h))
                .map(|rs| rs.get_rule_defs().len())
                .unwrap_or(0);

            for (idx, rule_def) in rules_defs.iter().enumerate() {
                let global_idx = offset + idx;
                let rule = rule_def.to_rule_with_index(global_idx);
                let rule_id = rule_def.generate_id(global_idx);

                if !rule_def.actions.is_empty() {
                    action_defs
                        .actions_by_rule
                        .insert(rule_id.clone(), rule_def.actions.clone());
                }

                registry.register(rule);
            }
            info!("Battle FRE: Registered {} custom rules", rules_defs.len());
        }
    }

    battle_rules_handle.registered = true;
    info!("Battle FRE: All rules registered and action_defs populated");
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
    battle_rules_handle.menu_handle = None;
    battle_rules_handle.registered = false;

    info!("Battle FRE: Cleaned up local layer and reset rules handle");
}
