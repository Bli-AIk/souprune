//! # trigger.rs
//!
//! ## Module Overview
//! FRE-based trigger zones for overworld areas.
//! Handles trigger zone detection and emits FRE events.
//!
//! ## 模块概述
//! 基于 FRE 的 Overworld 区域触发器。
//! 处理触发区域检测并发出 FRE 事件。

use crate::app_state::overworld::OverworldEntity;
use crate::app_state::overworld::character::components::PlayerControlled;
use crate::core::collision::Rect2DCollider;
use crate::core::data::PlayerData;
use bevy::prelude::*;
use bevy_fact_rule_event::{
    FactDatabase, FactEvent, FactEventId, FactModification, FactValue, Rule, RuleAction,
    RuleCondition, RuleRegistry,
};

/// Marker component for trigger zones.
///
/// 触发区域的标记组件。
#[derive(Component, Debug)]
pub struct TriggerZone {
    /// Unique identifier for this trigger.
    ///
    /// 此触发器的唯一标识符。
    pub id: String,

    /// Event to emit when player enters this zone.
    ///
    /// 玩家进入此区域时发出的事件。
    pub enter_event: String,

    /// Whether the player is currently inside this zone.
    ///
    /// 玩家当前是否在此区域内。
    pub player_inside: bool,
}

impl TriggerZone {
    pub fn new(id: impl Into<String>) -> Self {
        let id = id.into();
        let enter_event = format!("trigger_enter_{}", id);
        Self {
            id,
            enter_event,
            player_inside: false,
        }
    }
}

/// Marker for the demo trigger zone to avoid duplicate spawning.
///
/// 演示触发区域的标记，用于避免重复生成。
#[derive(Component)]
pub struct DemoTriggerSpawned;

/// System to spawn a demo trigger zone for testing FRE.
/// This creates a 64x64 trigger zone near the player spawn point.
///
/// 生成用于测试 FRE 的演示触发区域。
/// 在玩家出生点附近创建一个 64x64 的触发区域。
pub fn spawn_demo_trigger_zone_system(
    mut commands: Commands,
    existing_triggers: Query<&DemoTriggerSpawned>,
    player_query: Query<&Transform, Added<PlayerControlled>>,
) {
    // Only spawn once
    if !existing_triggers.is_empty() {
        return;
    }

    // Wait for player to spawn, then create trigger near them
    let Ok(player_transform) = player_query.single() else {
        return;
    };

    // Create trigger zone 48 pixels to the right of player spawn
    let trigger_pos = Vec3::new(
        player_transform.translation.x + 48.0,
        player_transform.translation.y,
        0.0,
    );

    commands.spawn((
        OverworldEntity(),
        TriggerZone::new("demo_trigger"),
        Rect2DCollider::new(Vec2::new(64.0, 64.0), Vec2::ZERO),
        Transform::from_translation(trigger_pos),
        Visibility::Hidden,
        DemoTriggerSpawned,
        Name::new("DemoTriggerZone"),
    ));

    info!(
        "FRE: Spawned demo trigger zone at ({:.1}, {:.1})",
        trigger_pos.x, trigger_pos.y
    );
    info!("FRE: Walk into the cyan box (press F3 to see it) 3 times to set HP to 42");
}

/// System to detect player entering/exiting trigger zones and emit FRE events.
///
/// 检测玩家进入/离开触发区域并发出 FRE 事件的系统。
#[allow(clippy::type_complexity)]
pub fn trigger_zone_detection_system(
    mut triggers: Query<(&Transform, &Rect2DCollider, &mut TriggerZone)>,
    player: Query<(&Transform, &Rect2DCollider, Entity), With<PlayerControlled>>,
    mut event_writer: MessageWriter<FactEvent>,
) {
    let Ok((player_transform, player_collider, player_entity)) = player.single() else {
        return;
    };

    let player_pos = player_transform.translation.truncate() + player_collider.offset;
    let player_half_size = player_collider.size * 0.5;

    for (trigger_transform, trigger_collider, mut trigger) in triggers.iter_mut() {
        let trigger_pos = trigger_transform.translation.truncate() + trigger_collider.offset;
        let trigger_half_size = trigger_collider.size * 0.5;

        // AABB overlap check
        let overlap = (player_pos.x - trigger_pos.x).abs()
            < (player_half_size.x + trigger_half_size.x)
            && (player_pos.y - trigger_pos.y).abs() < (player_half_size.y + trigger_half_size.y);

        if overlap && !trigger.player_inside {
            // Player just entered
            trigger.player_inside = true;
            info!("FRE: Player entered trigger zone '{}'", trigger.id);
            event_writer.write(
                FactEvent::with_entity(trigger.enter_event.clone(), player_entity)
                    .with_data("trigger_id", &trigger.id),
            );
        } else if !overlap && trigger.player_inside {
            // Player just exited
            trigger.player_inside = false;
            info!("FRE: Player exited trigger zone '{}'", trigger.id);
        }
    }
}

/// Setup the demo FRE rules for the area visit counter gameplay.
///
/// 设置区域访问计数器游戏玩法的演示 FRE 规则。
pub fn setup_demo_fre_rules(mut registry: ResMut<RuleRegistry>, mut fact_db: ResMut<FactDatabase>) {
    // Initialize the visit counter
    fact_db.set("demo_area_visit_count", 0i64);

    // Rule 1: Increment counter on every entry
    let increment_rule = Rule::builder("demo_increment_visit", "trigger_enter_demo_trigger")
        .condition(RuleCondition::Always)
        .modify(FactModification::Increment(
            "demo_area_visit_count".to_string(),
            1,
        ))
        .output("demo_visit_updated")
        .priority(10)
        .build();

    // Rule 2: On the 3rd visit, set player HP to 42
    let hp_rule = Rule::builder("demo_set_hp_on_third_visit", "demo_visit_updated")
        .condition(RuleCondition::Equals(
            "demo_area_visit_count".to_string(),
            FactValue::Int(3),
        ))
        .action(RuleAction::new(|_event, _db, _commands| {
            info!("FRE Action: Requesting HP change to 42 (will be applied by separate system)");
        }))
        .output("demo_hp_change_requested")
        .priority(5)
        .build();

    registry.register(increment_rule);
    registry.register(hp_rule);

    info!("FRE: Demo rules registered");
}

/// System to apply HP changes when the FRE rule triggers.
///
/// 当 FRE 规则触发时应用 HP 变化的系统。
pub fn apply_hp_change_system(
    mut events: MessageReader<FactEvent>,
    mut player_data: ResMut<PlayerData>,
    fact_db: Res<FactDatabase>,
) {
    for event in events.read() {
        if event.id == FactEventId::new("demo_hp_change_requested") {
            let visit_count = fact_db.get_int_or("demo_area_visit_count", 0);
            info!(
                "FRE: Setting player HP to 42 (visit count: {})",
                visit_count
            );
            player_data.hp = 42;
        }
    }
}

/// System to log fact database changes (debug).
///
/// 记录事实数据库变化的系统（调试用）。
pub fn log_fact_changes_system(mut events: MessageReader<FactEvent>, fact_db: Res<FactDatabase>) {
    for event in events.read() {
        if event.id == FactEventId::new("demo_visit_updated") {
            let count = fact_db.get_int_or("demo_area_visit_count", 0);
            info!("FRE: demo_area_visit_count = {}", count);
        }
    }
}
