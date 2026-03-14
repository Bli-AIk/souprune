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

use crate::app_state::battle::BattleUpdate;
use crate::core::fre_facts;
use crate::core::input::{Action, PlayerInputSettings};
use crate::core::sequencer::SequenceRulesHandle;
use bevy::prelude::*;
use bevy_fact_rule_event::{FreAsset, LayeredFactDatabase, LayeredRuleRegistry};
use leafwing_input_manager::action_state::ActionState;

pub use action_handlers::{
    apply_pending_damage_system, has_pending_damage, setup_battle_action_handlers_system,
};
pub use bridge::{
    ActOptionsTracker, ChapterCompletedEvent, copy_enemy_act_data_system,
    emit_chapter_completed_events_system, has_chapter_completed_events,
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

/// Plugin for Battle FRE integration.
///
/// 战斗 FRE 集成插件。
pub struct BattleFREPlugin;

impl Plugin for BattleFREPlugin {
    fn build(&self, app: &mut App) {
        let schedule = crate::game_schedule(app);
        app.add_message::<ChapterCompletedEvent>()
            .init_resource::<ActOptionsTracker>()
            .configure_sets(schedule, BattleFRESet.in_set(BattleUpdate))
            .add_systems(
                schedule,
                (setup_battle_fre_system, setup_battle_action_handlers_system)
                    .run_if(super::on_entering_battle),
            )
            .add_systems(
                schedule,
                cleanup_battle_fre_system.run_if(super::on_exiting_battle),
            )
            .add_systems(
                schedule,
                (
                    register_battle_rules_system,
                    emit_chapter_completed_events_system.run_if(has_chapter_completed_events),
                    apply_pending_damage_system.run_if(has_pending_damage),
                    copy_enemy_act_data_system,
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
        crate::app_state::battle::battle_scoped(),
        player_input.get_merged_map(),
        ActionState::<Action>::default(),
    ));
    info!("Battle FRE: Spawned input entity for ActionState");

    // NOTE: Battle menu rules (battle_menu.fre.ron) are loaded via View's requires mechanism.
    // The View layout declares: requires: [File("battle/rules/battle_menu.fre.ron")]
    // 注意：战斗菜单规则通过 View 的 requires 机制加载。
    // View 布局声明：requires: [File("battle/rules/battle_menu.fre.ron")]

    // Player data is already in global layer (managed by core::data module)
    // No need to sync here

    let hp = layered_db.get_int(fre_facts::PLAYER_HP).unwrap_or(20);
    let hp_max = layered_db.get_int(fre_facts::PLAYER_HP_MAX).unwrap_or(20);
    info!("Battle FRE: Initialized with player HP {}/{}", hp, hp_max);
}

/// System to register battle-specific rules when loaded.
/// Handles custom battle rules (optional).
/// NOTE: Battle menu rules are loaded via View's requires mechanism.
///
/// 当战斗规则加载完成时注册它们的系统。
/// 处理自定义战斗规则（可选）。
/// 注意：战斗菜单规则通过 View 的 requires 机制加载。
fn register_battle_rules_system(
    mut sequence_rules_handle: ResMut<SequenceRulesHandle>,
    fre_assets: Res<Assets<FreAsset>>,
    mut registry: ResMut<LayeredRuleRegistry>,
    mut fact_db: ResMut<LayeredFactDatabase>,
    mut enum_registry: ResMut<bevy_fact_rule_event::EnumRegistry>,
) {
    // Skip if already registered
    if sequence_rules_handle.registered {
        return;
    }

    // Wait for custom rules if specified
    let custom_loaded = sequence_rules_handle
        .handle
        .as_ref()
        .map(|h| fre_assets.get(h).is_some())
        .unwrap_or(true); // true if no custom rules

    if !custom_loaded {
        return;
    }

    // Process custom battle rules (if any)
    if let Some(handle) = &sequence_rules_handle.handle
        && let Some(fre_asset) = fre_assets.get(handle)
    {
        // Register enums from battle rules
        enum_registry.register_from_asset(fre_asset);

        for (key, value) in fre_asset.resolve_facts(&enum_registry) {
            fact_db.set_local(key.as_str(), value);
            info!("Battle FRE: Set fact '{}' from battle rules", key);
        }

        // Register rules (actions are now part of Rule struct)
        fre_asset.register_rules_layered(&mut registry);
        info!(
            "Battle FRE: Registered {} custom rules",
            fre_asset.get_rule_defs().len()
        );
    }

    sequence_rules_handle.registered = true;
    info!("Battle FRE: All rules registered");
}

/// System to clean up FRE state when exiting battle.
/// Optionally promotes important facts to global layer.
///
/// 退出战斗时清理 FRE 状态的系统。
/// 可选地将重要事实提升到全局层。
fn cleanup_battle_fre_system(
    mut layered_db: ResMut<LayeredFactDatabase>,
    mut registry: ResMut<LayeredRuleRegistry>,
    mut sequence_rules_handle: ResMut<SequenceRulesHandle>,
) {
    // TODO: Optionally promote certain facts to global layer
    // (e.g., battle results, experience gained)

    // Clear local layer facts
    layered_db.clear_local();

    // Clear local layer rules
    registry.clear_local();

    // Reset rules handle for next battle
    sequence_rules_handle.handle = None;
    sequence_rules_handle.registered = false;

    info!("Battle FRE: Cleaned up local layer (facts and rules) and reset rules handle");
}
