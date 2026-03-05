//! # trigger.rs
//!
//! ## Module Overview
//! FRE-based trigger zones and interactable objects for overworld areas.
//! Handles trigger zone detection, interactable detection, and emits FRE events.
//! Rules are loaded from RON files for data-driven gameplay.
//!
//! ## 模块概述
//! 基于 FRE 的 Overworld 区域触发器和可交互物体。
//! 处理触发区域检测、可交互物体检测，并发出 FRE 事件。
//! 规则从 RON 文件加载以实现数据驱动的游戏玩法。

use crate::app_state::overworld::character::components::PlayerControlled;
use crate::core::basic_components::{Direction, Facing};
use crate::core::collision::Rect2DCollider;
use crate::core::danmaku::PlayPerformanceEvent;
use crate::core::input::{Action, ActionRegistry, ActionStateExt};
use crate::core::map_property_schema::{get_string_property, keys};
use bevy::prelude::*;
use bevy_ecs_tiled::prelude::{TiledMap, TiledMapAsset};
use bevy_fact_rule_event::{
    FactEvent, FactEventId, FactValueDef, FreAsset, LayeredFactDatabase, LayeredRuleRegistry,
    RuleActionDef,
};
use leafwing_input_manager::action_state::ActionState;
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

/// Marker component for interactable objects.
/// These objects can be interacted with when the player faces them and presses confirm.
///
/// 可交互物体的标记组件。
/// 当玩家面向这些物体并按下确认键时，可以与它们交互。
///
/// Dialogue configuration is now handled via FRE rules generated at Tiled map load time.
/// See `object_properties.rs` for rule generation logic.
///
/// 对话配置现在通过在 Tiled 地图加载时生成的 FRE 规则处理。
/// 规则生成逻辑见 `object_properties.rs`。
#[derive(Component, Debug)]
pub struct Interactable {
    /// Unique identifier for this interactable.
    ///
    /// 此可交互物体的唯一标识符。
    pub id: String,

    /// Maximum interaction distance from player.
    ///
    /// 与玩家的最大交互距离。
    pub max_distance: f32,
}

impl Default for Interactable {
    fn default() -> Self {
        Self {
            id: String::new(),
            max_distance: 20.0,
        }
    }
}

impl Interactable {
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            max_distance: 20.0,
        }
    }

    pub fn with_distance(mut self, distance: f32) -> Self {
        self.max_distance = distance;
        self
    }
}

/// Resource to track the currently focused interactable entity.
///
/// 跟踪当前聚焦的可交互实体的资源。
#[derive(Resource, Default)]
pub struct FocusedInteractable {
    pub entity: Option<Entity>,
    pub id: Option<String>,
}

/// Resource to track loaded rule set handles.
///
/// 跟踪已加载规则集句柄的资源。
#[derive(Resource, Default)]
pub struct LoadedRuleSets {
    pub handles: Vec<Handle<FreAsset>>,
    pub initialized: bool,
    pub registered: bool,
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
///
/// NOTE: UI navigation rules (backpack.fre.ron) are now loaded via View's `requires`
/// when the View is spawned, not here.
/// 注意：UI 导航规则（backpack.fre.ron）现在通过 View 的 `requires` 在 View 生成时加载，
/// 而不是在这里。
pub fn load_fre_rules_system(
    asset_server: Res<AssetServer>,
    mut loaded_rule_sets: ResMut<LoadedRuleSets>,
    tiled_maps: Query<&TiledMap>,
    tiled_map_assets: Res<Assets<TiledMapAsset>>,
) {
    if loaded_rule_sets.initialized {
        return;
    }

    // NOTE: backpack.fre.ron is no longer loaded here.
    // It's loaded via View's `requires` in undertale_backpack.view.ron.
    // 注意：backpack.fre.ron 不再在此处加载。
    // 它通过 undertale_backpack.view.ron 中 View 的 `requires` 加载。

    // Try to find rules_file property in loaded maps (using schema key constant)
    for tiled_map in tiled_maps.iter() {
        if let Some(map_asset) = tiled_map_assets.get(&tiled_map.0)
            && let Some(rules_path) =
                get_string_property(&map_asset.map.properties, keys::RULES_FILE)
        {
            let rules_path_owned = rules_path.to_string();
            let handle: Handle<FreAsset> = asset_server.load(&rules_path_owned);
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
    // 没找到 rules_file 属性 - 标记为已初始化但没有地图特定规则
    if !tiled_maps.is_empty() {
        for tiled_map in tiled_maps.iter() {
            if tiled_map_assets.get(&tiled_map.0).is_some() {
                loaded_rule_sets.initialized = true;
                info!("FRE: No rules_file property found in map");
                return;
            }
        }
    }
}

/// System to register rules from loaded assets.
///
/// 从已加载的资产注册规则的系统。
pub fn register_loaded_rules_system(
    mut loaded_rule_sets: ResMut<LoadedRuleSets>,
    fre_assets: Res<Assets<FreAsset>>,
    mut registry: ResMut<LayeredRuleRegistry>,
    mut fact_db: ResMut<LayeredFactDatabase>,
    mut action_defs: ResMut<RuleActionDefs>,
) {
    if loaded_rule_sets.registered || !loaded_rule_sets.initialized {
        return;
    }

    // Wait until all rule sets are loaded
    let all_loaded = loaded_rule_sets
        .handles
        .iter()
        .all(|h| fre_assets.get(h).is_some());

    if !all_loaded {
        return;
    }

    for handle in &loaded_rule_sets.handles {
        if let Some(fre_asset) = fre_assets.get(handle) {
            // Apply facts to Local layer (room/scene specific)
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
                info!("FRE: Set fact '{}' to Local layer from FRE file", key);
            }

            // Store action definitions for each rule (for custom action handling)
            // Use the same ID generation logic as to_rule_with_index()
            for (idx, rule_def) in fre_asset.get_rule_defs().iter().enumerate() {
                let rule_id = rule_def.generate_id(idx);
                action_defs
                    .actions_by_rule
                    .insert(rule_id, rule_def.actions.clone());
            }

            // Register all rules to layered registry
            fre_asset.register_rules_layered(&mut registry);
            info!("FRE: Rules registered from FRE asset");
        }
    }

    loaded_rule_sets.registered = true;
    info!(
        "FRE: All {} rule sets registered",
        loaded_rule_sets.handles.len()
    );
}

/// Resource to store pending danmaku play requests from FRE actions.
///
/// 存储来自 FRE action 的待播放弹幕请求的资源。
#[derive(Resource, Default)]
pub struct PendingDanmakuActions {
    pub requests: Vec<String>,
}

/// Resource to store pending view spawn/despawn requests from registered FRE action handlers.
/// These are accumulated via `commands.queue()` in handler closures, then processed
/// by `apply_pending_view_actions_system` which has full ECS access to send messages.
///
/// 存储来自注册 FRE action handler 的待处理 View 生成/销毁请求的资源。
/// 通过 handler 闭包中的 `commands.queue()` 积累，
/// 然后由 `apply_pending_view_actions_system` 处理（该系统拥有完整 ECS 访问权限以发送消息）。
#[derive(Resource, Default)]
pub struct PendingViewActions {
    pub spawn_requests: Vec<crate::core::view::SpawnViewRequest>,
    pub despawn_requests: Vec<crate::core::view::DespawnViewRequest>,
}

/// System to setup overworld-specific action handlers in ActionHandlerRegistry.
/// Registers handlers for: SetMode, SetSubState, EnterChaseState, ExitChaseState,
/// SpawnView, DespawnView, PlayDanmaku, SetPlayerHP.
///
/// 设置 Overworld 专用 action handler 的系统。
/// 注册处理器：SetMode、SetSubState、EnterChaseState、ExitChaseState、
/// SpawnView、DespawnView、PlayDanmaku、SetPlayerHP。
pub fn setup_action_handlers_system(world: &mut World) {
    world.init_resource::<PendingDanmakuActions>();
    world.init_resource::<PendingViewActions>();

    let mut handler_registry = world.resource_mut::<bevy_fact_rule_event::ActionHandlerRegistry>();

    handler_registry.register("SetMode", |action, _db, commands| {
        if let bevy_fact_rule_event::RuleActionDef::Custom { params, .. } = action {
            if let Some(mode) = params.get("mode").cloned() {
                info!("FRE: Setting mode to '{}' via registered handler", mode);
                commands.queue(move |world: &mut World| {
                    world.resource_mut::<crate::app_state::SequenceMode>().0 = Some(mode);
                });
            } else {
                warn!("FRE: SetMode action missing 'mode' param");
            }
        }
    });

    handler_registry.register("SetSubState", |action, _db, commands| {
        if let bevy_fact_rule_event::RuleActionDef::Custom { params, .. } = action {
            if let Some(state) = params.get("state").cloned() {
                info!(
                    "FRE: Setting sub-state to '{}' via registered handler",
                    state
                );
                commands.queue(move |world: &mut World| {
                    world
                        .resource_mut::<NextState<crate::app_state::SequenceSubState>>()
                        .set(crate::app_state::SequenceSubState::new(&state));
                });
            } else {
                warn!("FRE: SetSubState action missing 'state' param");
            }
        }
    });

    handler_registry.register("EnterChaseState", |_action, _db, commands| {
        commands.queue(move |world: &mut World| {
            let chase_enabled = world.resource::<super::chase::ChaseEnabled>().0;
            if !chase_enabled {
                warn!("FRE: EnterChaseState action ignored - chase not enabled");
                return;
            }
            let chase_state_name = world.resource::<super::chase::ChaseStateName>().0.clone();
            let Some(state_name) = chase_state_name else {
                warn!("FRE: EnterChaseState action ignored - no chase state name configured");
                return;
            };
            info!(
                "FRE: Entering chase state '{}' via registered handler",
                state_name
            );
            world
                .resource_mut::<NextState<crate::app_state::SequenceSubState>>()
                .set(crate::app_state::SequenceSubState::new(&state_name));
        });
    });

    handler_registry.register("ExitChaseState", |_action, _db, commands| {
        commands.queue(move |world: &mut World| {
            let chase_enabled = world.resource::<super::chase::ChaseEnabled>().0;
            if !chase_enabled {
                warn!("FRE: ExitChaseState action ignored - chase not enabled");
                return;
            }
            info!("FRE: Exiting chase state via registered handler");
            world
                .resource_mut::<NextState<crate::app_state::SequenceSubState>>()
                .set(crate::app_state::SequenceSubState::default());
        });
    });

    handler_registry.register("SpawnView", |action, _db, commands| {
        if let bevy_fact_rule_event::RuleActionDef::Custom { params, .. } = action {
            if let Some(path) = params.get("path").cloned() {
                info!("FRE: Spawning view '{}' via registered handler", path);
                commands.queue(move |world: &mut World| {
                    world
                        .resource_mut::<PendingViewActions>()
                        .spawn_requests
                        .push(crate::core::view::SpawnViewRequest {
                            path,
                            mode_scope: None,
                            bindings: None,
                        });
                });
            } else {
                warn!("FRE: SpawnView action missing 'path' param");
            }
        }
    });

    handler_registry.register("DespawnView", |action, _db, commands| {
        if let bevy_fact_rule_event::RuleActionDef::Custom { params, .. } = action {
            let path = params.get("path").cloned();
            info!(
                "FRE: Despawning view(s) via registered handler (path: {:?})",
                path
            );
            commands.queue(move |world: &mut World| {
                world
                    .resource_mut::<PendingViewActions>()
                    .despawn_requests
                    .push(crate::core::view::DespawnViewRequest { path });
            });
        }
    });

    handler_registry.register("PlayDanmaku", |action, _db, commands| {
        if let bevy_fact_rule_event::RuleActionDef::Custom { params, .. } = action {
            if let Some(path) = params.get("path").cloned() {
                info!("FRE: PlayDanmaku '{}' via registered handler", path);
                commands.queue(move |world: &mut World| {
                    world
                        .resource_mut::<PendingDanmakuActions>()
                        .requests
                        .push(path);
                });
            } else {
                warn!("FRE: PlayDanmaku action missing 'path' param");
            }
        }
    });

    handler_registry.register("SetPlayerHP", |action, _db, _commands| {
        if let bevy_fact_rule_event::RuleActionDef::Custom { params, .. } = action
            && let Some(value_str) = params.get("value")
            && let Ok(hp) = value_str.parse::<usize>()
        {
            info!("FRE Action: SetPlayerHP requested with value {}", hp);
        }
    });

    info!("FRE: Overworld action handlers registered (8 handlers)");
}

/// System that handles unregistered Custom FRE actions
/// dispatched via `FreCustomActionEvent`.
/// Only logs unhandled actions since registered ones are processed
/// by ActionHandlerRegistry in dispatch_custom_actions_system.
///
/// 处理未注册的 Custom FRE action 的系统。
/// 仅记录未处理的 action，已注册的由 dispatch_custom_actions_system 中的
/// ActionHandlerRegistry 处理。
pub fn handle_overworld_custom_actions_system(
    mut events: MessageReader<crate::core::fre_bridge::FreCustomActionEvent>,
) {
    for event in events.read() {
        debug!(
            "FRE: Unhandled custom action '{}' with params {:?}",
            event.action_type, event.params
        );
    }
}

/// System to apply pending view actions from registered FRE action handlers.
/// Reads from PendingViewActions and sends SpawnViewRequest/DespawnViewRequest messages.
///
/// 应用来自注册 FRE action handler 的待处理 View 操作的系统。
/// 从 PendingViewActions 读取并发送 SpawnViewRequest/DespawnViewRequest 消息。
pub fn apply_pending_view_actions_system(
    mut pending: ResMut<PendingViewActions>,
    mut spawn_writer: MessageWriter<crate::core::view::SpawnViewRequest>,
    mut despawn_writer: MessageWriter<crate::core::view::DespawnViewRequest>,
) {
    for request in pending.spawn_requests.drain(..) {
        spawn_writer.write(request);
    }
    for request in pending.despawn_requests.drain(..) {
        despawn_writer.write(request);
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

/// System to detect interactable objects in front of the player.
/// Updates FocusedInteractable resource when player faces an interactable.
///
/// 检测玩家面前可交互物体的系统。
/// 当玩家面向可交互物体时更新 FocusedInteractable 资源。
#[allow(clippy::type_complexity)]
/// Check if a ray from `origin` in direction `dir` intersects an AABB defined by `center` and `half_size`.
/// Returns the distance to intersection if hit, or None if no intersection within `max_dist`.
fn ray_aabb_intersection(
    origin: Vec2,
    dir: Vec2,
    center: Vec2,
    half_size: Vec2,
    max_dist: f32,
) -> Option<f32> {
    let min = center - half_size;
    let max = center + half_size;

    // Handle each axis
    let (mut t_min, mut t_max) = (0.0_f32, max_dist);

    // X axis
    if dir.x.abs() < 1e-6 {
        // Ray parallel to Y axis
        if origin.x < min.x || origin.x > max.x {
            return None;
        }
    } else {
        let inv_d = 1.0 / dir.x;
        let mut t1 = (min.x - origin.x) * inv_d;
        let mut t2 = (max.x - origin.x) * inv_d;
        if t1 > t2 {
            std::mem::swap(&mut t1, &mut t2);
        }
        t_min = t_min.max(t1);
        t_max = t_max.min(t2);
        if t_min > t_max {
            return None;
        }
    }

    // Y axis
    if dir.y.abs() < 1e-6 {
        // Ray parallel to X axis
        if origin.y < min.y || origin.y > max.y {
            return None;
        }
    } else {
        let inv_d = 1.0 / dir.y;
        let mut t1 = (min.y - origin.y) * inv_d;
        let mut t2 = (max.y - origin.y) * inv_d;
        if t1 > t2 {
            std::mem::swap(&mut t1, &mut t2);
        }
        t_min = t_min.max(t1);
        t_max = t_max.min(t2);
        if t_min > t_max {
            return None;
        }
    }

    // Check if intersection is within valid range
    if t_min >= 0.0 && t_min <= max_dist {
        Some(t_min)
    } else if t_max >= 0.0 && t_max <= max_dist {
        Some(t_max)
    } else {
        None
    }
}

pub fn interactable_detection_system(
    player_query: Query<(&Transform, &Facing, &Rect2DCollider), With<PlayerControlled>>,
    interactables: Query<(Entity, &Transform, &Interactable, Option<&Rect2DCollider>)>,
    mut focused: ResMut<FocusedInteractable>,
    mut logged_once: Local<bool>,
) {
    let Ok((player_transform, facing, player_collider)) = player_query.single() else {
        return;
    };

    // Log interactable count once
    if !*logged_once {
        let count = interactables.iter().count();
        info!(
            "Interactable detection: found {} interactable entities",
            count
        );
        *logged_once = true;
    }

    let player_pos = player_transform.translation.truncate() + player_collider.offset;

    // Normalize facing direction to 4 cardinal directions only
    // Player sprite only has 4 directions, so we should only cast rays in cardinal directions
    // 将朝向规范化为仅 4 个主方向
    // 玩家 sprite 只有 4 个方向，因此我们只应该在主方向发射射线
    let facing_dir = match facing.value {
        Direction::Up | Direction::UpLeft | Direction::UpRight => Vec2::Y,
        Direction::Down | Direction::DownLeft | Direction::DownRight => -Vec2::Y,
        Direction::Left => -Vec2::X,
        Direction::Right => Vec2::X,
    };

    // Find the closest interactable that the ray intersects
    let mut best_match: Option<(Entity, String, f32)> = None;

    for (entity, interactable_transform, interactable, opt_collider) in interactables.iter() {
        // Get interactable center position and size
        let (center, half_size) = match opt_collider {
            Some(collider) => (
                interactable_transform.translation.truncate() + collider.offset,
                collider.size / 2.0,
            ),
            None => {
                // No collider - use a small default area
                (
                    interactable_transform.translation.truncate(),
                    Vec2::splat(8.0),
                )
            }
        };

        // Ray-AABB intersection test
        if let Some(hit_dist) = ray_aabb_intersection(
            player_pos,
            facing_dir,
            center,
            half_size,
            interactable.max_distance,
        ) {
            // Update best match if closer
            if best_match.is_none() || hit_dist < best_match.as_ref().unwrap().2 {
                best_match = Some((entity, interactable.id.clone(), hit_dist));
            }
        }
    }

    // Update focused interactable
    match best_match {
        Some((entity, id, _)) => {
            if focused.entity != Some(entity) {
                focused.entity = Some(entity);
                focused.id = Some(id.clone());
                debug!("FRE: Player can interact with '{}'", id);
            }
        }
        None => {
            if focused.entity.is_some() {
                debug!("FRE: No interactable in range");
                focused.entity = None;
                focused.id = None;
            }
        }
    }
}

/// System to handle player interaction when confirm is pressed.
/// Emits FRE event `interact_{id}` for rule-based handling.
/// Dialogue configuration is handled by FRE rules generated from Tiled map properties.
///
/// 当按下确认键时处理玩家交互的系统。
/// 发出 FRE 事件 `interact_{id}` 用于基于规则的处理。
/// 对话配置由从 Tiled 地图属性生成的 FRE 规则处理。
pub fn handle_interaction_input_system(
    registry: Res<ActionRegistry>,
    query: Query<&ActionState<Action>, With<PlayerControlled>>,
    focused: Res<FocusedInteractable>,
    interactables: Query<&Interactable>,
    current_state: Res<State<crate::app_state::SequenceSubState>>,
    state_config: Option<Res<crate::core::state_config::LoadedStateConfig>>,
    mut event_writer: MessageWriter<FactEvent>,
) {
    // Only handle interaction when player can interact in current state
    let can_interact = state_config
        .as_ref()
        .and_then(|config| config.0.states.get(&current_state.0))
        .map(|def| def.can_interact())
        .unwrap_or(true);

    if !can_interact {
        return;
    }

    let Ok(action_state) = query.single() else {
        return;
    };

    // Check if confirm was just pressed
    if !action_state.action_just_pressed(&registry, "Confirm") {
        return;
    }

    // Check if there's a focused interactable
    let Some(entity) = focused.entity else {
        return;
    };

    let Some(ref interactable_id) = focused.id else {
        return;
    };

    // Verify the entity still has Interactable component
    if interactables.get(entity).is_err() {
        warn!(
            "FRE: Focused entity {:?} has no Interactable component",
            entity
        );
        return;
    }

    // Emit interaction event for FRE rule-based handling
    // FRE rules handle dialogue configuration (view spawn, Mortar start, etc.)
    let event_id = format!("interact_{}", interactable_id);
    info!(
        "FRE: Player interacting with '{}', emitting '{}'",
        interactable_id, event_id
    );
    event_writer.write(FactEvent::new(event_id));
}
