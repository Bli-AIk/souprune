//! # action_handlers.rs
//!
//! Fixed-scene FRE action handlers.
//!
//! fixed-scene的 FRE 动作处理器。
//!
//! This module registers custom action handlers for fixed-scene
//! actions that can be triggered from FRE rules.
//!
//! 本模块注册fixed-scene动作的自定义处理器，
//! 这些动作可以从 FRE 规则中触发。

use bevy::prelude::*;
use bevy_fact_rule_event::LayeredFactDatabase;

use crate::core::game_action::{GameActionDef, GameActionHandlerRegistry};

/// System to set up fixed-scene action handlers.
/// Called when entering fixed-scene mode.
///
/// 设置fixed-scene动作处理器的系统。
/// 在进入 fixed-scene mode时调用。
pub fn setup_battle_action_handlers_system(
    mut handler_registry: ResMut<GameActionHandlerRegistry>,
) {
    // DealDamage - Apply damage to an entity
    handler_registry.register("DealDamage", |action, _db, _commands| {
        if let GameActionDef::Custom { params, .. } = action {
            let target = params.get("target").map(String::as_str).unwrap_or("player");
            let amount_str = params.get("amount").map(String::as_str).unwrap_or("0");
            let amount: i64 = amount_str.parse().unwrap_or(0);

            info!(
                "FixedScene FRE Action: DealDamage to '{}' for {} damage",
                target, amount
            );

            // Note: Actual damage application should be done through a separate system
            // that reads from FactDatabase, as we can't safely modify facts here
            // The damage amount should be stored in a pending damage fact
        }
    });

    // Heal - Apply healing to an entity
    handler_registry.register("Heal", |action, _db, _commands| {
        if let GameActionDef::Custom { params, .. } = action {
            let target = params.get("target").map(String::as_str).unwrap_or("player");
            let amount_str = params.get("amount").map(String::as_str).unwrap_or("0");
            let amount: i64 = amount_str.parse().unwrap_or(0);

            info!("FixedScene FRE Action: Heal '{}' for {} HP", target, amount);
        }
    });

    // PlayDanmaku - Play a danmaku performance
    handler_registry.register("PlayDanmaku", |action, _db, _commands| {
        if let GameActionDef::Custom { params, .. } = action {
            let performance_path = params
                .get("performance")
                .map(String::as_str)
                .unwrap_or("default.performance.ron");

            info!("FixedScene FRE Action: PlayDanmaku '{}'", performance_path);

            // TODO: Emit danmaku play event
        }
    });

    // EndScene - End the current scene
    handler_registry.register("EndScene", |action, _db, _commands| {
        if let GameActionDef::Custom { params, .. } = action {
            let result = params
                .get("result")
                .map(String::as_str)
                .unwrap_or("victory");

            info!("FixedScene FRE Action: EndScene with result '{}'", result);

            // TODO: Emit scene end event or set state
        }
    });

    // SetPhase - Change the current scene phase
    handler_registry.register("SetPhase", |action, _db, _commands| {
        if let GameActionDef::Custom { params, .. } = action {
            let phase = params.get("phase").map(String::as_str).unwrap_or("unknown");

            info!("FixedScene FRE Action: SetPhase to '{}'", phase);

            // Note: Phase change should be done through a system that modifies LayeredFactDatabase
        }
    });

    // IncrementTurn - Increment the turn counter
    handler_registry.register("IncrementTurn", |_action, _db, _commands| {
        info!("FixedScene FRE Action: IncrementTurn");
        // Note: Turn increment should be done through fact modifications in rules,
        // not through custom actions
    });

    // SpawnEnemy - Spawn an enemy entity
    handler_registry.register("SpawnEnemy", |action, _db, _commands| {
        if let GameActionDef::Custom { params, .. } = action {
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
                "FixedScene FRE Action: SpawnEnemy '{}' at ({}, {})",
                enemy_type, position_x, position_y
            );

            // TODO: Spawn enemy entity through commands
        }
    });

    // DespawnEnemy - Despawn an enemy entity
    handler_registry.register("DespawnEnemy", |action, _db, _commands| {
        if let GameActionDef::Custom { params, .. } = action {
            let enemy_index: usize = params
                .get("enemy_index")
                .and_then(|s| s.parse().ok())
                .unwrap_or(0);

            info!("FixedScene FRE Action: DespawnEnemy index {}", enemy_index);

            // TODO: Despawn enemy entity
        }
    });

    info!("FixedScene FRE: Action handlers registered");
}

/// Run condition: Check if there's pending damage to apply.
/// 运行条件：检查是否有待处理的伤害。
pub fn has_pending_damage(layered_db: Res<LayeredFactDatabase>) -> bool {
    layered_db
        .get_int("player:pending_damage")
        .is_some_and(|d| d != 0)
        || layered_db
            .get_int("enemy:0:pending_damage")
            .is_some_and(|d| d != 0)
}

/// System to apply pending damage from FRE actions.
/// This system reads damage facts and applies them to entity HP.
///
/// 应用 FRE 动作中待处理伤害的系统。
/// 此系统读取伤害事实并将其应用于实体 HP。
pub fn apply_pending_damage_system(mut layered_db: ResMut<LayeredFactDatabase>) {
    // Check for pending player damage
    if let Some(pending_damage) = layered_db.get_int("player:pending_damage")
        && pending_damage != 0
    {
        let current_hp = layered_db.get_int_or("player:hp", 100);
        let new_hp = (current_hp - pending_damage).max(0);

        layered_db.set("player:hp", new_hp);
        layered_db.remove("player:pending_damage");

        info!(
            "FixedScene FRE: Applied {} damage to player (HP: {} -> {})",
            pending_damage, current_hp, new_hp
        );
    }

    // Check for pending enemy damage (enemy_0 as example)
    if let Some(pending_damage) = layered_db.get_int("enemy:0:pending_damage")
        && pending_damage != 0
    {
        let current_hp = layered_db.get_int_or("enemy:0:hp", 100);
        let new_hp = (current_hp - pending_damage).max(0);

        layered_db.set("enemy:0:hp", new_hp);
        layered_db.remove("enemy:0:pending_damage");

        info!(
            "FixedScene FRE: Applied {} damage to enemy_0 (HP: {} -> {})",
            pending_damage, current_hp, new_hp
        );
    }
}
