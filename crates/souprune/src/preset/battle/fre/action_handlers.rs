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
use bevy_fact_rule_event::LayeredFactDatabase;
use bevy_tween::interpolation::EaseKind;

use crate::core::battle_box::{GapPolicy, MergeBattleBoxes, SplitAxis, SplitBattleBox};
use crate::core::game_action::{GameActionDef, GameActionHandlerRegistry};

/// System to set up battle-specific action handlers.
/// Called when entering Battle state.
///
/// 设置战斗特定动作处理器的系统。
/// 在进入 Battle 状态时调用。
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
        if let GameActionDef::Custom { params, .. } = action {
            let target = params.get("target").map(String::as_str).unwrap_or("player");
            let amount_str = params.get("amount").map(String::as_str).unwrap_or("0");
            let amount: i64 = amount_str.parse().unwrap_or(0);

            info!("Battle FRE Action: Heal '{}' for {} HP", target, amount);
        }
    });

    // PlayDanmaku - Play a danmaku performance
    handler_registry.register("PlayDanmaku", |action, _db, _commands| {
        if let GameActionDef::Custom { params, .. } = action {
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
        if let GameActionDef::Custom { params, .. } = action {
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
        if let GameActionDef::Custom { params, .. } = action {
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
                "Battle FRE Action: SpawnEnemy '{}' at ({}, {})",
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

            info!("Battle FRE Action: DespawnEnemy index {}", enemy_index);

            // TODO: Despawn enemy entity
        }
    });

    // SplitBattleBox - Split a battle box into two
    handler_registry.register("SplitBattleBox", |action, _db, commands| {
        let GameActionDef::Custom { params, .. } = action else {
            return;
        };
        let source = params
            .get("source")
            .cloned()
            .unwrap_or_else(|| "main".to_string());
        let left = params
            .get("left")
            .cloned()
            .unwrap_or_else(|| "left".to_string());
        let right = params
            .get("right")
            .cloned()
            .unwrap_or_else(|| "right".to_string());
        let axis = match params.get("axis").map(String::as_str) {
            Some("Horizontal") => SplitAxis::Horizontal,
            _ => SplitAxis::Vertical,
        };
        let position: f32 = params
            .get("position")
            .and_then(|s| s.parse().ok())
            .unwrap_or(0.0);
        let gap: f32 = params
            .get("gap")
            .and_then(|s| s.parse().ok())
            .unwrap_or(0.0);
        let gap_policy = match params.get("gap_policy").map(String::as_str) {
            Some("Includes") => GapPolicy::Includes,
            _ => GapPolicy::Expands,
        };
        let duration: f32 = params
            .get("duration")
            .and_then(|s| s.parse().ok())
            .unwrap_or(0.0);

        info!(
            "Battle FRE Action: SplitBattleBox '{}' → '{}' + '{}' (axis={:?}, gap_policy={:?}, duration={})",
            source, left, right, axis, gap_policy, duration
        );

        let msg = SplitBattleBox {
            source_box: source,
            result_boxes: (left, right),
            split_axis: axis,
            split_position: position,
            gap,
            gap_policy,
            duration,
            easing: EaseKind::Linear,
        };
        commands.queue(move |world: &mut World| {
            world.write_message(msg);
        });
    });

    // MergeBattleBoxes - Merge two battle boxes into one
    handler_registry.register("MergeBattleBoxes", |action, _db, commands| {
        let GameActionDef::Custom { params, .. } = action else {
            return;
        };
        let box_a = params
            .get("box_a")
            .cloned()
            .unwrap_or_else(|| "left".to_string());
        let box_b = params
            .get("box_b")
            .cloned()
            .unwrap_or_else(|| "right".to_string());
        let result = params
            .get("result")
            .cloned()
            .unwrap_or_else(|| "main".to_string());
        let duration: f32 = params
            .get("duration")
            .and_then(|s| s.parse().ok())
            .unwrap_or(0.0);
        let gap_policy = match params.get("gap_policy").map(String::as_str) {
            Some("Includes") => GapPolicy::Includes,
            _ => GapPolicy::Expands,
        };

        info!(
            "Battle FRE Action: MergeBattleBoxes '{}' + '{}' → '{}' (gap_policy={:?}, duration={})",
            box_a, box_b, result, gap_policy, duration
        );

        let msg = MergeBattleBoxes {
            source_boxes: (box_a, box_b),
            result_box: result,
            gap_policy,
            duration,
            easing: EaseKind::Linear,
        };
        commands.queue(move |world: &mut World| {
            world.write_message(msg);
        });
    });

    info!("Battle FRE: Action handlers registered");
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
            "Battle FRE: Applied {} damage to player (HP: {} -> {})",
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
            "Battle FRE: Applied {} damage to enemy_0 (HP: {} -> {})",
            pending_damage, current_hp, new_hp
        );
    }
}
