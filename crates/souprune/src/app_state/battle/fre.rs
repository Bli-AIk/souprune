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
//! NOTE: Battle UI navigation has been moved to FRE rules in battle_menu.fre.ron.
//! The old hardcoded navigation system has been removed.
//!
//! 注意：战斗 UI 导航已移至 battle_menu.fre.ron 中的 FRE 规则。
//! 旧的硬编码导航系统已被移除。

mod action_handlers;
mod bridge;

use crate::app_state::AppState;
use crate::app_state::battle::BattleUpdate;
use crate::app_state::overworld::trigger::RuleActionDefs;
use crate::core::input::{Action, PlayerInputSettings};
use bevy::prelude::*;
use bevy_fact_rule_event::{FactValueDef, FreAsset, LayeredFactDatabase, LayeredRuleRegistry};
use leafwing_input_manager::action_state::ActionState;

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

/// Marker component for the battle input entity.
/// This entity carries ActionState for FRE input processing during battle.
///
/// 战斗输入实体的标记组件。
/// 此实体在战斗期间携带 ActionState 用于 FRE 输入处理。
#[derive(Component)]
pub struct BattleInputEntity;

/// Resource to track the battle-specific rules handle.
///
/// 跟踪战斗特定规则句柄的资源。
#[derive(Resource, Default)]
pub struct BattleRulesHandle {
    /// Handle for custom battle rules (from battle asset rules_file)
    pub handle: Option<Handle<FreAsset>>,
    /// Handle for battle menu rules (always loaded)
    pub menu_handle: Option<Handle<FreAsset>>,
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
                    // Note: Battle UI navigation is now handled by FRE rules in battle_menu.fre.ron
                    // The core::fre_bridge::FREBridgePlugin provides ActionEvent-to-FRE conversion
                    // 注意：战斗 UI 导航现在由 battle_menu.fre.ron 中的 FRE 规则处理
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
    mut commands: Commands,
    mut layered_db: ResMut<LayeredFactDatabase>,
    mut battle_rules_handle: ResMut<BattleRulesHandle>,
    asset_server: Res<AssetServer>,
    player_input: Res<PlayerInputSettings>,
) {
    // Clear local layer from any previous state
    layered_db.clear_local();

    // Set initial battle facts (local layer)
    layered_db.set("battle_is_active", true);
    layered_db.set("battle_turn_count", 0i64);
    layered_db.set("battle_phase", "initializing");

    // Spawn input entity for FRE to receive ActionState events
    // 生成输入实体，用于 FRE 接收 ActionState 事件
    commands.spawn((
        Name::new("BattleInputEntity"),
        BattleInputEntity,
        crate::app_state::battle::BattleEntity,
        player_input.get_merged_map(),
        ActionState::<Action>::default(),
    ));
    info!("Battle FRE: Spawned input entity for ActionState");

    // Always load battle menu rules
    let menu_path = "battle/rules/battle_menu.fre.ron";
    let menu_handle = asset_server.load::<FreAsset>(menu_path);
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
    fre_assets: Res<Assets<FreAsset>>,
    mut registry: ResMut<LayeredRuleRegistry>,
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
        .map(|h| fre_assets.get(h).is_some())
        .unwrap_or(false);

    // Wait for custom rules if specified
    let custom_loaded = battle_rules_handle
        .handle
        .as_ref()
        .map(|h| fre_assets.get(h).is_some())
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

    // Process menu rules - facts only, rules are registered via View's requires mechanism
    // 处理菜单规则 - 仅处理 facts，规则通过 View 的 requires 机制注册
    if let Some(handle) = &battle_rules_handle.menu_handle
        && let Some(fre_asset) = fre_assets.get(handle)
    {
        // NOTE: Menu rules are now registered as View-scoped rules via the View layout's
        // requires: [File("battle/rules/battle_menu.fre.ron")] declaration.
        // We only apply facts here for backward compatibility with any direct Local layer access.
        // 注意：菜单规则现在通过 View 布局的 requires 声明注册为 View 作用域规则。
        // 此处仅应用 facts 以兼容任何直接访问 Local 层的情况。
        for (key, value) in fre_asset.get_facts() {
            let fact_value = match value {
                FactValueDef::Int(v) => bevy_fact_rule_event::FactValue::Int(*v),
                FactValueDef::Float(v) => bevy_fact_rule_event::FactValue::Float(*v),
                FactValueDef::Bool(v) => bevy_fact_rule_event::FactValue::Bool(*v),
                FactValueDef::String(v) => bevy_fact_rule_event::FactValue::String(v.clone()),
                FactValueDef::StringList(v) => {
                    bevy_fact_rule_event::FactValue::StringList(v.clone())
                }
                FactValueDef::IntList(v) => bevy_fact_rule_event::FactValue::IntList(v.clone()),
            };
            fact_db.set_local(key.as_str(), fact_value);
            trace!(
                "Battle FRE: Set fact '{}' from menu rules to Local layer",
                key
            );
        }
        info!(
            "Battle FRE: Applied {} facts from menu rules (rules registered via View)",
            fre_asset.get_facts().len()
        );
    }

    // Process custom battle rules (if any)
    if let Some(handle) = &battle_rules_handle.handle
        && let Some(fre_asset) = fre_assets.get(handle)
    {
        for (key, value) in fre_asset.get_facts() {
            let fact_value = match value {
                FactValueDef::Int(v) => bevy_fact_rule_event::FactValue::Int(*v),
                FactValueDef::Float(v) => bevy_fact_rule_event::FactValue::Float(*v),
                FactValueDef::Bool(v) => bevy_fact_rule_event::FactValue::Bool(*v),
                FactValueDef::String(v) => bevy_fact_rule_event::FactValue::String(v.clone()),
                FactValueDef::StringList(v) => {
                    bevy_fact_rule_event::FactValue::StringList(v.clone())
                }
                FactValueDef::IntList(v) => bevy_fact_rule_event::FactValue::IntList(v.clone()),
            };
            fact_db.set_local(key.as_str(), fact_value);
            info!("Battle FRE: Set fact '{}' from battle rules", key);
        }

        let rules_defs = fre_asset.get_rule_defs();
        let scope = fre_asset.scope();
        let offset = battle_rules_handle
            .menu_handle
            .as_ref()
            .and_then(|h| fre_assets.get(h))
            .map(|rs| rs.get_rule_defs().len())
            .unwrap_or(0);

        for (idx, rule_def) in rules_defs.iter().enumerate() {
            let global_idx = offset + idx;
            let rule = rule_def.to_rule_with_index(global_idx, scope);
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
    mut registry: ResMut<LayeredRuleRegistry>,
    mut battle_rules_handle: ResMut<BattleRulesHandle>,
) {
    // TODO: Optionally promote certain facts to global layer
    // (e.g., battle results, experience gained)

    // Clear local layer facts
    layered_db.clear_local();

    // Clear local layer rules
    registry.clear_local();

    // Reset rules handle for next battle
    battle_rules_handle.handle = None;
    battle_rules_handle.menu_handle = None;
    battle_rules_handle.registered = false;

    info!("Battle FRE: Cleaned up local layer (facts and rules) and reset rules handle");
}
