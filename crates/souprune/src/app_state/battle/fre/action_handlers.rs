//! # action_handlers.rs
//!
//! Battle-specific FRE action handlers.
//!
//! 战斗特定的 FRE 动作处理器。
//!
//! This module registers custom action handlers for battle-related
//! actions that can be triggered from FRE rules.
//!
//! 本模块注册战斗相关动作的自定义处理器，
//! 这些动作可以从 FRE 规则中触发。

use bevy::prelude::*;
use bevy_fact_rule_event::{ActionHandlerRegistry, LayeredFactDatabase, RuleActionDef};

/// System to set up battle-specific action handlers.
/// Called when entering Battle state.
///
/// 设置战斗特定动作处理器的系统。
/// 在进入 Battle 状态时调用。
pub fn setup_battle_action_handlers_system(mut handler_registry: ResMut<ActionHandlerRegistry>) {
    // DealDamage - Apply damage to an entity
    handler_registry.register("DealDamage", |action, _db, _commands| {
        if let RuleActionDef::Custom { params, .. } = action {
            let target = params.get("target").map(String::as_str).unwrap_or("player");
            let amount_str = params.get("amount").map(String::as_str).unwrap_or("0");
            let amount: i64 = amount_str.parse().unwrap_or(0);

            info!(
                "Battle FRE Action: DealDamage to '{}' for {} damage",
                target, amount
            );

            // Note: Actual damage application should be done through a separate system
            // that reads from FactDatabase, as we can't safely modify facts here
            // The damage amount should be stored in a pending damage fact
        }
    });

    // Heal - Apply healing to an entity
    handler_registry.register("Heal", |action, _db, _commands| {
        if let RuleActionDef::Custom { params, .. } = action {
            let target = params.get("target").map(String::as_str).unwrap_or("player");
            let amount_str = params.get("amount").map(String::as_str).unwrap_or("0");
            let amount: i64 = amount_str.parse().unwrap_or(0);

            info!("Battle FRE Action: Heal '{}' for {} HP", target, amount);
        }
    });

    // TriggerDialogue - Deprecated, use FRE rule modifications instead
    // 已弃用，请改用 FRE 规则的 modifications
    //
    // Example FRE rule for simple text dialogue:
    // ```ron
    // (
    //     id: "battle_narration",
    //     trigger: "battle:show_narration",
    //     modifications: [
    //         SetFact(key: "dialogue:simple_text_active", value: Bool(true)),
    //         SetFact(key: "dialogue:simple_text", value: String("* Your soul is filled with determination.")),
    //         SetFact(key: "dialogue:has_typewriter", value: Bool(true)),
    //     ],
    // )
    // ```
    //
    // Example FRE rule for Mortar dialogue:
    // ```ron
    // (
    //     id: "start_mortar_dialogue",
    //     trigger: "npc:interact",
    //     modifications: [
    //         SetFact(key: "dialogue:pending_mortar_path", value: String("dialogue/npc.mortar")),
    //         SetFact(key: "dialogue:pending_mortar_node", value: String("greeting")),
    //         SetFact(key: "dialogue:has_typewriter", value: Bool(true)),
    //     ],
    // )
    // ```
    handler_registry.register("TriggerDialogue", |action, _db, _commands| {
        if let RuleActionDef::Custom { params, .. } = action {
            let text = params.get("text").map(String::as_str).unwrap_or("");
            let path = params.get("path").map(String::as_str).unwrap_or("");
            let node = params.get("node").map(String::as_str).unwrap_or("");

            warn!(
                "TriggerDialogue action is deprecated. Use FRE rule modifications instead. \
                Received: text='{}', path='{}', node='{}'",
                text, path, node
            );

            // Note: Cannot modify facts from action handler.
            // Use FRE rule modifications to set dialogue:simple_text_active,
            // dialogue:simple_text, dialogue:pending_mortar_path, dialogue:pending_mortar_node
        }
    });

    // PlayDanmaku - Play a danmaku performance
    handler_registry.register("PlayDanmaku", |action, _db, _commands| {
        if let RuleActionDef::Custom { params, .. } = action {
            let performance_path = params
                .get("performance")
                .map(String::as_str)
                .unwrap_or("default.performance.ron");

            info!("Battle FRE Action: PlayDanmaku '{}'", performance_path);

            // TODO: Emit danmaku play event
        }
    });

    // EndBattle - End the current battle
    handler_registry.register("EndBattle", |action, _db, _commands| {
        if let RuleActionDef::Custom { params, .. } = action {
            let result = params
                .get("result")
                .map(String::as_str)
                .unwrap_or("victory");

            info!("Battle FRE Action: EndBattle with result '{}'", result);

            // TODO: Emit battle end event or set state
        }
    });

    // SetPhase - Change the current battle phase
    handler_registry.register("SetPhase", |action, _db, _commands| {
        if let RuleActionDef::Custom { params, .. } = action {
            let phase = params.get("phase").map(String::as_str).unwrap_or("unknown");

            info!("Battle FRE Action: SetPhase to '{}'", phase);

            // Note: Phase change should be done through a system that modifies LayeredFactDatabase
        }
    });

    // IncrementTurn - Increment the turn counter
    handler_registry.register("IncrementTurn", |_action, _db, _commands| {
        info!("Battle FRE Action: IncrementTurn");
        // Note: Turn increment should be done through fact modifications in rules,
        // not through custom actions
    });

    // SpawnEnemy - Spawn an enemy entity
    handler_registry.register("SpawnEnemy", |action, _db, _commands| {
        if let RuleActionDef::Custom { params, .. } = action {
            let enemy_type = params
                .get("enemy_type")
                .map(String::as_str)
                .unwrap_or("unknown");
            let position_x: f32 = params
                .get("position_x")
                .and_then(|s| s.parse().ok())
                .unwrap_or(0.0);
            let position_y: f32 = params
                .get("position_y")
                .and_then(|s| s.parse().ok())
                .unwrap_or(0.0);

            info!(
                "Battle FRE Action: SpawnEnemy '{}' at ({}, {})",
                enemy_type, position_x, position_y
            );

            // TODO: Spawn enemy entity through commands
        }
    });

    // DespawnEnemy - Despawn an enemy entity
    handler_registry.register("DespawnEnemy", |action, _db, _commands| {
        if let RuleActionDef::Custom { params, .. } = action {
            let enemy_index: usize = params
                .get("enemy_index")
                .and_then(|s| s.parse().ok())
                .unwrap_or(0);

            info!("Battle FRE Action: DespawnEnemy index {}", enemy_index);

            // TODO: Despawn enemy entity
        }
    });

    info!("Battle FRE: Action handlers registered");
}

/// System to apply pending damage from FRE actions.
/// This system reads damage facts and applies them to entity HP.
///
/// 应用 FRE 动作中待处理伤害的系统。
/// 此系统读取伤害事实并将其应用于实体 HP。
pub fn apply_pending_damage_system(mut layered_db: ResMut<LayeredFactDatabase>) {
    // Check for pending player damage
    if let Some(pending_damage) = layered_db.get_int("pending_player_damage")
        && pending_damage != 0
    {
        let current_hp = layered_db.get_int_or("player:hp", 100);
        let new_hp = (current_hp - pending_damage).max(0);

        layered_db.set("player:hp", new_hp);
        layered_db.remove("pending_player_damage");

        info!(
            "Battle FRE: Applied {} damage to player (HP: {} -> {})",
            pending_damage, current_hp, new_hp
        );
    }

    // Check for pending enemy damage (enemy_0 as example)
    if let Some(pending_damage) = layered_db.get_int("pending_enemy_0_damage")
        && pending_damage != 0
    {
        let current_hp = layered_db.get_int_or("enemy_0_hp", 100);
        let new_hp = (current_hp - pending_damage).max(0);

        layered_db.set("enemy_0_hp", new_hp);
        layered_db.remove("pending_enemy_0_damage");

        info!(
            "Battle FRE: Applied {} damage to enemy_0 (HP: {} -> {})",
            pending_damage, current_hp, new_hp
        );
    }
}
