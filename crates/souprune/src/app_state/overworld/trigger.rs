//! # trigger.rs
//!
//! ## Module Overview
//! FRE-based trigger zones for overworld areas.
//! Handles trigger zone detection and emits FRE events.
//! Rules are loaded from RON files for data-driven gameplay.
//!
//! ## 模块概述
//! 基于 FRE 的 Overworld 区域触发器。
//! 处理触发区域检测并发出 FRE 事件。
//! 规则从 RON 文件加载以实现数据驱动的游戏玩法。

use crate::app_state::overworld::OverworldEntity;
use crate::app_state::overworld::character::components::PlayerControlled;
use crate::core::collision::Rect2DCollider;
use crate::core::danmaku::PlayPerformanceEvent;
use crate::core::map_property_schema::{get_string_property, keys};
use bevy::prelude::*;
use bevy_ecs_tiled::prelude::{TiledMap, TiledMapAsset};
use bevy_fact_rule_event::{
    ActionHandlerRegistry, FactEvent, FactEventId, FactValueDef, LayeredFactDatabase,
    RuleActionDef, RuleRegistry, RuleSetAsset,
};
use std::collections::HashMap;

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

/// Resource to track loaded rule set handles.
///
/// 跟踪已加载规则集句柄的资源。
#[derive(Resource, Default)]
pub struct LoadedRuleSets {
    pub handles: Vec<Handle<RuleSetAsset>>,
    pub initialized: bool,
}

/// Resource to store the mapping from rule IDs to their action definitions.
/// This is populated when rules are registered and used for custom action handling.
///
/// 存储规则 ID 到其 action 定义的映射的资源。
/// 在规则注册时填充，用于自定义 action 处理。
#[derive(Resource, Default)]
pub struct RuleActionDefs {
    /// Maps rule ID to its action definitions
    pub actions_by_rule: HashMap<String, Vec<RuleActionDef>>,
}

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
    info!("FRE: Walk into the cyan box (press F3 to see it) to trigger danmaku!");
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

/// System to load FRE rules from map's `rules_file` property.
/// The rules file path is read from the Tiled map's custom properties.
/// If no `rules_file` property exists, no rules are loaded for this map.
///
/// 从地图的 `rules_file` 属性加载 FRE 规则的系统。
/// 规则文件路径从 Tiled 地图的自定义属性中读取。
/// 如果不存在 `rules_file` 属性，则不为此地图加载任何规则。
/// 同时加载 UI 导航规则（backpack.rules.ron）。
pub fn load_fre_rules_system(
    asset_server: Res<AssetServer>,
    mut loaded_rule_sets: ResMut<LoadedRuleSets>,
    tiled_maps: Query<&TiledMap>,
    tiled_map_assets: Res<Assets<TiledMapAsset>>,
) {
    if loaded_rule_sets.initialized {
        return;
    }

    // Always load UI navigation rules
    let backpack_rules_path = "overworld/rules/backpack.rules.ron";
    let backpack_handle: Handle<RuleSetAsset> = asset_server.load(backpack_rules_path);
    loaded_rule_sets.handles.push(backpack_handle);
    info!("FRE: Loading UI navigation rules: {}", backpack_rules_path);

    // Try to find rules_file property in loaded maps (using schema key constant)
    for tiled_map in tiled_maps.iter() {
        if let Some(map_asset) = tiled_map_assets.get(&tiled_map.0)
            && let Some(rules_path) =
                get_string_property(&map_asset.map.properties, keys::RULES_FILE)
        {
            let rules_path_owned = rules_path.to_string();
            let handle: Handle<RuleSetAsset> = asset_server.load(&rules_path_owned);
            loaded_rule_sets.handles.push(handle);
            loaded_rule_sets.initialized = true;
            info!(
                "FRE: Loading rules from map property '{}': {}",
                keys::RULES_FILE,
                rules_path_owned
            );
            return;
        }
    }

    // No rules_file property found - mark as initialized but with no map-specific rules
    // UI rules are still loaded
    if !tiled_maps.is_empty() {
        for tiled_map in tiled_maps.iter() {
            if tiled_map_assets.get(&tiled_map.0).is_some() {
                loaded_rule_sets.initialized = true;
                info!("FRE: No rules_file property found in map, only UI rules loaded");
                return;
            }
        }
    }
}

/// System to register rules from loaded assets.
///
/// 从已加载的资产注册规则的系统。
pub fn register_loaded_rules_system(
    loaded_rule_sets: Res<LoadedRuleSets>,
    rule_set_assets: Res<Assets<RuleSetAsset>>,
    mut registry: ResMut<RuleRegistry>,
    mut fact_db: ResMut<LayeredFactDatabase>,
    mut action_defs: ResMut<RuleActionDefs>,
    mut registered: Local<bool>,
) {
    if *registered || !loaded_rule_sets.initialized {
        return;
    }

    // Wait until all rule sets are loaded
    let all_loaded = loaded_rule_sets
        .handles
        .iter()
        .all(|h| rule_set_assets.get(h).is_some());

    if !all_loaded {
        return;
    }

    for handle in &loaded_rule_sets.handles {
        if let Some(rule_set) = rule_set_assets.get(handle) {
            // Apply initial facts to Local layer (room/scene specific)
            for (key, value) in rule_set.get_initial_facts() {
                let fact_value = match value {
                    FactValueDef::Int(v) => bevy_fact_rule_event::FactValue::Int(*v),
                    FactValueDef::Float(v) => bevy_fact_rule_event::FactValue::Float(*v),
                    FactValueDef::Bool(v) => bevy_fact_rule_event::FactValue::Bool(*v),
                    FactValueDef::String(v) => bevy_fact_rule_event::FactValue::String(v.clone()),
                    FactValueDef::StringList(v) => {
                        bevy_fact_rule_event::FactValue::StringList(v.clone())
                    }
                };
                fact_db.set_local(key.as_str(), fact_value);
                info!("FRE: Set initial fact '{}' to Local layer from RON", key);
            }

            // Store action definitions for each rule (for custom action handling)
            // Use the same ID generation logic as to_rule_with_index()
            for (idx, rule_def) in rule_set.get_rule_defs().iter().enumerate() {
                let rule_id = rule_def.generate_id(idx);
                action_defs
                    .actions_by_rule
                    .insert(rule_id, rule_def.actions.clone());
            }

            // Register all rules
            rule_set.register_rules(&mut registry);
            info!("FRE: Rules registered from RON asset");
        }
    }

    *registered = true;
    info!(
        "FRE: All {} rule sets registered",
        loaded_rule_sets.handles.len()
    );
}

/// System to setup custom action handlers for game-specific actions.
///
/// 设置游戏特定动作的自定义动作处理程序的系统。
pub fn setup_action_handlers_system(world: &mut World) {
    // Initialize the pending danmaku resource
    world.init_resource::<PendingDanmakuActions>();

    let mut handler_registry = world.resource_mut::<ActionHandlerRegistry>();

    // Register the SetPlayerHP action handler
    handler_registry.register("SetPlayerHP", |action, _db, _commands| {
        if let RuleActionDef::Custom { params, .. } = action
            && let Some(value_str) = params.get("value")
            && let Ok(hp) = value_str.parse::<usize>()
        {
            info!("FRE Action: SetPlayerHP requested with value {}", hp);
            // Note: Actual HP change is handled by apply_hp_change_system
            // because we can't access PlayerData from Commands
        }
    });

    // Note: PlayDanmaku is handled via PendingDanmakuActions resource
    // The actual registration happens below
    handler_registry.register("PlayDanmaku", |action, _db, _commands| {
        if let RuleActionDef::Custom { params, .. } = action {
            if let Some(path) = params.get("path") {
                info!("FRE Action: PlayDanmaku registered with path: {}", path);
                // Actual playback is handled by play_danmaku_from_actions_system
                // which reads from PendingDanmakuActions
            } else {
                warn!("FRE Action: PlayDanmaku missing 'path' parameter");
            }
        }
    });

    info!("FRE: Custom action handlers registered");
}

/// Resource to store pending danmaku play requests from FRE actions.
///
/// 存储来自 FRE action 的待播放弹幕请求的资源。
#[derive(Resource, Default)]
pub struct PendingDanmakuActions {
    pub requests: Vec<String>,
}

/// System to collect PlayDanmaku actions from executed rules.
/// This runs after rule evaluation and checks which rules were triggered.
///
/// 从已执行规则中收集 PlayDanmaku action 的系统。
/// 在规则评估后运行，检查哪些规则被触发。
pub fn collect_danmaku_actions_system(
    mut events: MessageReader<FactEvent>,
    mut rule_registry: ResMut<RuleRegistry>,
    fact_db: Res<LayeredFactDatabase>,
    action_defs: Res<RuleActionDefs>,
    mut pending: ResMut<PendingDanmakuActions>,
) {
    for event in events.read() {
        // Get all matching rules for this event
        let matching_rules = rule_registry.get_matching_rules(event);

        for rule in matching_rules {
            // Check if rule's condition is met
            if rule.condition.evaluate(&*fact_db) {
                // Look up the original action definitions for this rule
                if let Some(actions) = action_defs.actions_by_rule.get(&rule.id) {
                    for action in actions {
                        if let RuleActionDef::Custom {
                            action_type,
                            params,
                        } = action
                            && action_type == "PlayDanmaku"
                            && let Some(path) = params.get("path")
                        {
                            pending.requests.push(path.clone());
                        }
                    }
                }
            }
        }
    }
}

/// System to play danmaku from pending FRE PlayDanmaku actions.
///
/// 播放来自 FRE PlayDanmaku action 的弹幕。
pub fn play_danmaku_from_actions_system(
    mut performance_writer: MessageWriter<PlayPerformanceEvent>,
    player_query: Query<&Transform, With<PlayerControlled>>,
    mut pending: ResMut<PendingDanmakuActions>,
) {
    if pending.requests.is_empty() {
        return;
    }

    let spawn_pos = player_query
        .iter()
        .next()
        .map(|t| t.translation.truncate())
        .unwrap_or(Vec2::ZERO);

    for path in pending.requests.drain(..) {
        info!(
            "FRE: Playing danmaku performance: {} at {:?}",
            path, spawn_pos
        );
        performance_writer.write(PlayPerformanceEvent::new(&path).at_position(spawn_pos));
    }
}

/// System to log fact database changes (debug).
///
/// 记录事实数据库变化的系统（调试用）。
pub fn log_fact_changes_system(
    mut events: MessageReader<FactEvent>,
    fact_db: Res<LayeredFactDatabase>,
) {
    for event in events.read() {
        if event.id == FactEventId::new("demo_visit_updated") {
            let count = fact_db.get_int_or("demo_area_visit_count", 0);
            info!("FRE: demo_area_visit_count = {}", count);
        }
    }
}

/// System to handle chase state transitions based on FRE actions.
/// Reads EnterChaseState and ExitChaseState actions from rule definitions.
///
/// 根据 FRE action 处理追逐战状态转换的系统。
/// 从规则定义中读取 EnterChaseState 和 ExitChaseState action。
pub fn handle_chase_state_actions_system(
    mut events: MessageReader<FactEvent>,
    mut rule_registry: ResMut<RuleRegistry>,
    fact_db: Res<LayeredFactDatabase>,
    action_defs: Res<RuleActionDefs>,
    chase_enabled: Res<super::chase::ChaseEnabled>,
    chase_state_name: Res<super::chase::ChaseStateName>,
    mut next_state: ResMut<NextState<crate::app_state::overworld::OverworldSubState>>,
) {
    for event in events.read() {
        // Get all matching rules for this event
        let matching_rules = rule_registry.get_matching_rules(event);

        for rule in matching_rules {
            // Check if rule's condition is met
            if rule.condition.evaluate(&*fact_db) {
                // Look up the original action definitions for this rule
                if let Some(actions) = action_defs.actions_by_rule.get(&rule.id) {
                    for action in actions {
                        if let RuleActionDef::Custom { action_type, .. } = action {
                            match action_type.as_str() {
                                "EnterChaseState" => {
                                    if chase_enabled.0 {
                                        if let Some(ref state_name) = chase_state_name.0 {
                                            info!(
                                                "FRE: Entering chase state '{}' via action",
                                                state_name
                                            );
                                            next_state.set(
                                                crate::app_state::overworld::OverworldSubState::new(
                                                    state_name.clone(),
                                                ),
                                            );
                                        } else {
                                            warn!(
                                                "FRE: EnterChaseState action ignored - no chase state name configured"
                                            );
                                        }
                                    } else {
                                        warn!(
                                            "FRE: EnterChaseState action ignored - chase not enabled"
                                        );
                                    }
                                }
                                "ExitChaseState" => {
                                    if chase_enabled.0 {
                                        info!("FRE: Exiting chase state via action");
                                        // Return to default state (empty string means default)
                                        next_state.set(
                                            crate::app_state::overworld::OverworldSubState::default(
                                            ),
                                        );
                                    } else {
                                        warn!(
                                            "FRE: ExitChaseState action ignored - chase not enabled"
                                        );
                                    }
                                }
                                _ => {}
                            }
                        }
                    }
                }
            }
        }
    }
}
