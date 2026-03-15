//! FRE (Fact-Rule-Event) Bridge Module
//!
//! This module provides the bridge between game systems and the FRE rule engine.
//! It handles:
//! - Converting input actions to FRE events (ActionEvent)
//! - Processing FRE actions (PlaySound, SetLocalFact, CloseView, etc.) on ViewRoot
//! - Managing ActiveView markers
//!
//! FRE（Fact-Rule-Event）桥接模块
//!
//! 此模块提供游戏系统和 FRE 规则引擎之间的桥接。
//! 它处理：
//! - 将输入动作转换为 FRE 事件（ActionEvent）
//! - 在 ViewRoot 上处理 FRE 动作（PlaySound、SetLocalFact、CloseView 等）
//! - 管理 ActiveView 标记

mod eval;
pub use eval::evaluate_single_condition;
use eval::{evaluate_conditions, evaluate_local_fact_value, register_condition_evaluator_system};

use bevy::prelude::*;
use bevy_fact_rule_event::{
    CombinedFactReader, EnumRegistry, FactEvent, FactReader, FactValue, LayeredFactDatabase,
};

use crate::core::game_action::{
    GameActionDef, GameActionHandlerRegistry, GameRule, GameRuleRegistry,
};
use leafwing_input_manager::action_state::ActionState;
use std::collections::HashMap;

use crate::config::SoupruneConfig;
use crate::core::audio;
use crate::core::fre_facts;
use crate::core::input::{Action, ActionRegistry, ActionStateExt};
use crate::core::view::components::{ActiveView, ViewRoot};

/// Event emitted for each FRE Custom action encountered during rule evaluation.
/// Systems can read this event to handle specific action types.
///
/// 在规则评估期间遇到的 FRE Custom action 会发出此事件。
/// 系统可以读取此事件来处理特定的 action 类型。
#[derive(Message, Debug, Clone)]
pub struct FreCustomActionEvent {
    pub action_type: String,
    pub params: HashMap<String, String>,
}

/// System that converts input actions to FRE events.
///
/// This system checks all registered actions and emits FRE events
/// in the format `action:{action_name}:{kind}` where kind is one of:
/// - `just_pressed`
/// - `pressed`
/// - `just_released`
///
/// 将输入动作转换为 FRE 事件的系统。
///
/// 此系统检查所有已注册的动作，并以 `action:{action_name}:{kind}` 格式
/// 发出 FRE 事件，其中 kind 是以下之一：
/// - `just_pressed`
/// - `pressed`
/// - `just_released`
pub fn action_to_fre_event_system(
    registry: Res<ActionRegistry>,
    query: Query<&ActionState<Action>>,
    mut event_writer: MessageWriter<FactEvent>,
) {
    // Find any entity with ActionState - prioritize first found
    // 查找任何带有 ActionState 的实体 - 优先使用第一个找到的
    let Some(action_state) = query.iter().next() else {
        return;
    };

    // Emit events for each action state change
    for action_name in registry.all_actions() {
        // Normalize action name to lowercase for consistent FRE event matching
        // 将动作名称规范化为小写，以实现一致的 FRE 事件匹配
        let action_name_lower = action_name.to_lowercase();

        // JustPressed events
        if action_state.action_just_pressed(&registry, action_name) {
            let event_id = format!("action:{}:just_pressed", action_name_lower);
            info!(
                "FRE Bridge: {} just_pressed, emitting {}",
                action_name, event_id
            );
            event_writer.write(FactEvent::new(event_id));
        }

        // Note: We skip 'pressed' events to avoid spamming every frame
        // Enable if needed for hold-based mechanics
        // 注意：我们跳过 'pressed' 事件以避免每帧发送垃圾信息
        // 如果需要基于按住的机制，可以启用

        // JustReleased events
        if action_state.action_just_released(&registry, action_name) {
            let event_id = format!("action:{}:just_released", action_name_lower);
            debug!(
                "FRE Bridge: {} just_released, emitting {}",
                action_name, event_id
            );
            event_writer.write(FactEvent::new(event_id));
        }
    }
}

/// System that syncs Bevy state values to FRE facts.
///
/// This allows FRE rules to check current state using `$state:sequence_sub_state` and `$state:app_state`.
/// The `@` prefix indicates these are state-derived facts rather than user-defined facts.
///
/// 将 Bevy 状态值同步到 FRE facts 的系统。
///
/// 这允许 FRE 规则使用 `$state:sequence_sub_state` 和 `$state:app_state` 检查当前状态。
/// `@` 前缀表示这些是状态派生的 facts，而不是用户定义的 facts。
/// Run condition: Check if any state has changed
/// 运行条件：检查是否有任何状态变化
fn state_facts_need_sync(
    sub_state: Option<Res<State<crate::app_state::SequenceSubState>>>,
    app_state: Option<Res<State<crate::app_state::AppState>>>,
    facts: Res<bevy_fact_rule_event::LayeredFactDatabase>,
) -> bool {
    if let Some(ref state) = sub_state
        && state.is_changed()
    {
        return true;
    }
    if let Some(ref state) = app_state
        && state.is_changed()
    {
        return true;
    }
    // 编辑器 Stop 时 clear_local() 会清除状态 facts，需要在下次 Play 时重新同步
    if sub_state.is_some()
        && facts
            .get_by_str(fre_facts::STATE_SEQUENCE_SUB_STATE)
            .is_none()
    {
        return true;
    }
    if app_state.is_some() && facts.get_by_str(fre_facts::STATE_APP_STATE).is_none() {
        return true;
    }
    false
}

pub fn sync_state_to_facts_system(
    sub_state: Option<Res<State<crate::app_state::SequenceSubState>>>,
    app_state: Option<Res<State<crate::app_state::AppState>>>,
    mut facts: ResMut<bevy_fact_rule_event::LayeredFactDatabase>,
) {
    // Sync SequenceSubState (replaces OverworldSubState)
    if let Some(state) = sub_state {
        let state_name = state.get().name().to_string();
        facts.set(
            fre_facts::STATE_SEQUENCE_SUB_STATE,
            FactValue::String(state_name),
        );
    }

    // Sync AppState
    if let Some(state) = app_state {
        let state_name = format!("{:?}", state.get());
        facts.set(fre_facts::STATE_APP_STATE, FactValue::String(state_name));
    }
}

/// Log matching rule details for Left/Right navigation events.
fn log_event_rule_matches(event: &FactEvent, rule_groups: &[Vec<&GameRule>]) {
    if !event.id.0.contains("Left") && !event.id.0.contains("Right") {
        return;
    }
    debug!(
        "FRE Bridge: Event '{}' has {} rule groups matching",
        event.id.0,
        rule_groups.len()
    );
    for (i, group) in rule_groups.iter().enumerate() {
        for rule in group {
            debug!(
                "  Group {}: rule '{}' with {} conditions: {:?}",
                i,
                rule.id,
                rule.condition_expressions.len(),
                rule.condition_expressions
            );
        }
    }
}

/// Log debug info when rule conditions are not met.
fn log_condition_not_met(rule: &GameRule, view_root: &ViewRoot) {
    if rule.id.contains("act") || rule.id.contains("depth_2") {
        debug!(
            "FRE Bridge: Conditions not met for rule '{}', local_facts: depth={:?}, menu_context={:?}, act_selection={:?}, act_count={:?}",
            rule.id,
            view_root.local_facts.get_int("depth"),
            view_root.local_facts.get_int("menu_context"),
            view_root.local_facts.get_int("act_selection"),
            view_root.local_facts.get_int("act_count")
        );
    } else {
        debug!(
            "FRE Bridge: Conditions not met for rule '{}', local_facts: depth={:?}, selection={:?}",
            rule.id,
            view_root.local_facts.get_int("depth"),
            view_root.local_facts.get_int("selection")
        );
    }
}

/// Process all matching rules for a single FRE event on view actions.
fn process_event_view_actions(
    event: &FactEvent,
    rule_registry: &GameRuleRegistry,
    active_view_query: &mut Query<&mut ViewRoot, With<ActiveView>>,
    audio: &bevy_kira_audio::Audio,
    asset_server: &AssetServer,
    global_facts: &mut bevy_fact_rule_event::LayeredFactDatabase,
    pending_events: &mut bevy_fact_rule_event::PendingFactEvents,
    trigger_history: &mut Option<ResMut<crate::extra::debug::RuleTriggerHistory>>,
    time: &Time,
    enum_registry: &EnumRegistry,
    item_registry: &crate::core::item::ItemRegistry,
    souprune_config: &SoupruneConfig,
) {
    let rule_groups = rule_registry.get_matching_rules_grouped(event);
    log_event_rule_matches(event, &rule_groups);

    // Process rules by priority groups
    // 按优先级分组处理规则
    'outer: for group in rule_groups {
        for rule in group {
            let Ok(mut view_root) = active_view_query.single_mut() else {
                info!("FRE Bridge: No ActiveView found for rule '{}'", rule.id);
                continue;
            };

            let combined = CombinedFactReader::new(&view_root.local_facts, global_facts);
            if !evaluate_conditions(&rule.condition_expressions, &combined, enum_registry) {
                log_condition_not_met(rule, &view_root);
                continue;
            }

            info!(
                "FRE Bridge: Rule '{}' matched event '{}' (priority: {}, conditions: {}, outputs_len: {}, outputs: {:?}), executing actions",
                rule.id,
                event.id.0,
                rule.priority,
                rule.condition_expressions.len(),
                rule.outputs.len(),
                rule.outputs
            );

            // Record rule trigger for debug panel visualization
            // 记录规则触发以供调试面板可视化
            if let Some(history) = trigger_history {
                history.record_trigger(&rule.id, time.elapsed_secs_f64());
            }

            // Execute each action from the rule's actions
            for action in &rule.actions {
                execute_action(
                    action,
                    &mut view_root.local_facts,
                    global_facts,
                    audio,
                    asset_server,
                    enum_registry,
                    item_registry,
                    souprune_config,
                );
            }

            // Queue output events with deduplication
            // 使用去重队列输出事件
            for output_id in &rule.outputs {
                pending_events.queue_output(&rule.id, FactEvent::new(output_id.clone()));
            }

            if rule.consume_event {
                break 'outer;
            }
        }
    }
}

/// System that processes FRE actions that affect ViewRoot.local_facts.
///
/// This system listens for FRE events, checks which rules match,
/// and executes View-related actions (SetLocalFact, PlaySound, etc.)
/// on the ActiveView's ViewRoot.
///
/// 处理影响 ViewRoot.local_facts 的 FRE 动作的系统。
///
/// 此系统监听 FRE 事件，检查匹配的规则，
/// 并在 ActiveView 的 ViewRoot 上执行 View 相关动作
/// （SetLocalFact、PlaySound 等）。
pub fn process_view_actions_system(
    mut events: MessageReader<FactEvent>,
    rule_registry: Res<GameRuleRegistry>,
    mut active_view_query: Query<&mut ViewRoot, With<ActiveView>>,
    audio: Res<bevy_kira_audio::Audio>,
    asset_server: Res<AssetServer>,
    mut global_facts: ResMut<bevy_fact_rule_event::LayeredFactDatabase>,
    mut pending_events: ResMut<bevy_fact_rule_event::PendingFactEvents>,
    mut trigger_history: Option<ResMut<crate::extra::debug::RuleTriggerHistory>>,
    time: Res<Time>,
    enum_registry: Res<EnumRegistry>,
    item_registry: Res<crate::core::item::ItemRegistry>,
    souprune_config: Res<SoupruneConfig>,
) {
    let events_to_process: Vec<FactEvent> = events.read().cloned().collect();

    for event in &events_to_process {
        process_event_view_actions(
            event,
            &rule_registry,
            &mut active_view_query,
            &audio,
            &asset_server,
            &mut global_facts,
            &mut pending_events,
            &mut trigger_history,
            &time,
            &enum_registry,
            &item_registry,
            &souprune_config,
        );
    }
}

/// Execute a single FRE action on the ViewRoot's local_facts.
/// StartDialogue writes to the global LayeredFactDatabase instead.
///
/// 在 ViewRoot 的 local_facts 上执行单个 FRE 动作。
/// StartDialogue 写入全局 LayeredFactDatabase。
fn execute_action(
    action: &GameActionDef,
    local_facts: &mut bevy_fact_rule_event::FactDatabase,
    global_facts: &mut bevy_fact_rule_event::LayeredFactDatabase,
    audio: &bevy_kira_audio::Audio,
    asset_server: &AssetServer,
    enum_registry: &EnumRegistry,
    item_registry: &crate::core::item::ItemRegistry,
    souprune_config: &SoupruneConfig,
) {
    match action {
        GameActionDef::PlaySound(sound_name) => {
            debug!("FRE Bridge: PlaySound({})", sound_name);
            audio::play_sound(audio, asset_server, sound_name);
        }
        GameActionDef::PlaySoundFullPath(path) => {
            debug!("FRE Bridge: PlaySoundFullPath({})", path);
            audio::play_sound_full_path(audio, asset_server, path);
        }
        GameActionDef::SetLocalFact(key, value) => {
            let combined = CombinedFactReader::new(local_facts, global_facts);
            let fact_value = evaluate_local_fact_value(key, value, &combined, enum_registry);
            info!("FRE Bridge: SetLocalFact({}, {:?})", key, fact_value);
            local_facts.set(key.as_str(), fact_value);
        }
        GameActionDef::CloseView => {
            debug!("FRE Bridge: CloseView");
            local_facts.set(fre_facts::VIEW_CLOSE_REQUESTED, FactValue::Bool(true));
        }
        GameActionDef::SwitchState(state_name) => {
            debug!("FRE Bridge: SwitchState({})", state_name);
            local_facts.set(
                fre_facts::VIEW_SWITCH_STATE,
                FactValue::String(state_name.clone()),
            );
        }
        GameActionDef::EmitEvent(event_id) => {
            debug!("FRE Bridge: EmitEvent({})", event_id);
        }
        GameActionDef::StartDialogue {
            mortar,
            node,
            view,
            typewriter,
            focus,
            voice,
        } => {
            info!(
                "FRE Bridge: StartDialogue(mortar: {}, node: {})",
                mortar, node
            );
            global_facts.set_local(
                fre_facts::DIALOGUE_PENDING_MORTAR_PATH,
                FactValue::String(mortar.clone()),
            );
            global_facts.set_local(
                fre_facts::DIALOGUE_PENDING_MORTAR_NODE,
                FactValue::String(node.clone()),
            );
            if let Some(view_path) = view {
                global_facts.set_local(
                    fre_facts::DIALOGUE_PENDING_VIEW,
                    FactValue::String(view_path.clone()),
                );
            }
            global_facts.set_local(
                fre_facts::DIALOGUE_HAS_TYPEWRITER,
                FactValue::Bool(*typewriter),
            );
            global_facts.set_local(fre_facts::DIALOGUE_HAS_FOCUS, FactValue::Bool(*focus));
            if let Some(voice_path) = voice {
                global_facts.set_local(
                    fre_facts::DIALOGUE_VOICE,
                    FactValue::String(voice_path.clone()),
                );
            }
            global_facts.set_local(fre_facts::DIALOGUE_PENDING_START, FactValue::Bool(true));
        }
        GameActionDef::Custom {
            action_type,
            params,
        } => {
            debug!(
                "FRE Bridge: Custom action {} with params {:?}",
                action_type, params
            );
        }
        GameActionDef::Log { message } => {
            info!("FRE Bridge: Log: {}", message);
        }
        GameActionDef::UseItem { index_expr } => {
            execute_use_item(
                index_expr,
                local_facts,
                global_facts,
                audio,
                asset_server,
                enum_registry,
                item_registry,
                &souprune_config.game.dialogue_view_default,
                &souprune_config.game.dialogue_voice_default,
            );
        }
        GameActionDef::CheckItem { index_expr } => {
            execute_check_item(
                index_expr,
                local_facts,
                global_facts,
                enum_registry,
                item_registry,
                &souprune_config.game.dialogue_view_default,
                &souprune_config.game.dialogue_voice_default,
            );
        }
        GameActionDef::DropItem { index_expr } => {
            execute_drop_item(
                index_expr,
                local_facts,
                global_facts,
                enum_registry,
                item_registry,
                &souprune_config.game.dialogue_view_default,
                &souprune_config.game.dialogue_voice_default,
            );
        }
    }
}

// ============================================================================
// Item Action Handlers
// ============================================================================

/// Resolve an index expression (e.g., "$item_selection") to a usize index.
fn resolve_index_expr(
    index_expr: &str,
    local_facts: &bevy_fact_rule_event::FactDatabase,
    global_facts: &bevy_fact_rule_event::LayeredFactDatabase,
    enum_registry: &EnumRegistry,
) -> Option<usize> {
    use bevy_fact_rule_event::LocalFactValue;
    let combined = CombinedFactReader::new(local_facts, global_facts);
    let value = evaluate_local_fact_value(
        "_index",
        &LocalFactValue::Expr(index_expr.to_string()),
        &combined,
        enum_registry,
    );
    match value {
        FactValue::Int(i) if i >= 0 => Some(i as usize),
        _ => {
            warn!(
                "FRE Bridge: index_expr '{}' resolved to {:?}, expected non-negative Int",
                index_expr, value
            );
            None
        }
    }
}

/// Get inventory item_id at index, if valid.
fn get_inventory_item_id(
    index: usize,
    global_facts: &bevy_fact_rule_event::LayeredFactDatabase,
) -> Option<String> {
    global_facts
        .get_string_list("player:inventory")
        .and_then(|inv| inv.get(index).cloned())
}

/// Start a dialogue for an item action (OnUse/OnCheck/OnDrop).
/// Uses item's mortar file if available, otherwise falls back to defaults.
///
/// `item_data` contains pre-computed values for mortar variables/functions:
/// locale_key, description, heal_amount, item_value.
fn start_item_dialogue(
    item: &crate::core::item::Item,
    node_name: &str,
    default_node: &str,
    global_facts: &mut bevy_fact_rule_event::LayeredFactDatabase,
    dialogue_view_default: &str,
    dialogue_voice_default: &str,
    item_data: ItemDialogueData,
) {
    let (mortar_path, node) = if let Some(mortar) = &item.mortar {
        (mortar.as_str(), node_name)
    } else {
        ("items/_defaults.mortar", default_node)
    };

    start_item_dialogue_with_path(
        mortar_path,
        node,
        global_facts,
        dialogue_view_default,
        dialogue_voice_default,
        item_data,
    );
}

/// Start a dialogue with an explicit mortar path and node.
fn start_item_dialogue_with_path(
    mortar_path: &str,
    node: &str,
    global_facts: &mut bevy_fact_rule_event::LayeredFactDatabase,
    dialogue_view_default: &str,
    dialogue_voice_default: &str,
    item_data: ItemDialogueData,
) {
    info!(
        "FRE Bridge: Item dialogue — mortar: {}, node: {}",
        mortar_path, node
    );
    global_facts.set_local(
        fre_facts::DIALOGUE_PENDING_VIEW,
        FactValue::String(dialogue_view_default.to_string()),
    );
    global_facts.set_local(
        fre_facts::DIALOGUE_PENDING_MORTAR_PATH,
        FactValue::String(mortar_path.to_string()),
    );
    global_facts.set_local(
        fre_facts::DIALOGUE_PENDING_MORTAR_NODE,
        FactValue::String(node.to_string()),
    );
    global_facts.set_local(fre_facts::DIALOGUE_HAS_TYPEWRITER, FactValue::Bool(true));
    global_facts.set_local(fre_facts::DIALOGUE_HAS_FOCUS, FactValue::Bool(true));
    if !dialogue_voice_default.is_empty() {
        global_facts.set_local(
            fre_facts::DIALOGUE_VOICE,
            FactValue::String(dialogue_voice_default.to_string()),
        );
    }

    // Store item data for mortar variable/function resolution
    global_facts.set_local(
        fre_facts::DIALOGUE_ITEM_NAME,
        FactValue::String(item_data.locale_key),
    );
    global_facts.set_local(
        fre_facts::DIALOGUE_ITEM_DESCRIPTION,
        FactValue::String(item_data.description),
    );
    global_facts.set_local(
        fre_facts::DIALOGUE_ITEM_HEAL_AMOUNT,
        FactValue::Int(item_data.heal_amount),
    );
    global_facts.set_local(
        fre_facts::DIALOGUE_ITEM_VALUE,
        FactValue::Int(item_data.item_value),
    );

    global_facts.set_local(fre_facts::DIALOGUE_PENDING_START, FactValue::Bool(true));
}

/// Pre-computed item data for mortar dialogue variables and functions.
struct ItemDialogueData {
    locale_key: String,
    description: String,
    heal_amount: i64,
    item_value: i64,
}

/// Compute the stat value for an item (used by mortar function `get_item_value()`).
fn compute_item_value(item: &crate::core::item::Item) -> i64 {
    use crate::core::item::ItemType;
    match &item.item_type {
        ItemType::Food { effects, .. } => effects
            .iter()
            .find_map(|e| match e {
                crate::core::item::ItemEffect::Heal { amount } => Some(*amount as i64),
                _ => None,
            })
            .unwrap_or(0),
        ItemType::Weapon { damage, .. } => *damage as i64,
        ItemType::Armor { defense } => *defense as i64,
        ItemType::KeyItem => 0,
    }
}

/// Execute item effects (Heal, PlayAudio, SpawnChildItem, SetFact).
/// Returns the actual amount of HP healed (0 if no heal occurred).
fn apply_item_effects(
    item: &crate::core::item::Item,
    index: usize,
    global_facts: &mut bevy_fact_rule_event::LayeredFactDatabase,
    audio: &bevy_kira_audio::Audio,
    asset_server: &AssetServer,
) -> i64 {
    use crate::core::item::{ItemEffect, ItemType};

    let effects = match &item.item_type {
        ItemType::Food { effects, .. } => effects.as_slice(),
        _ => return 0,
    };

    let mut spawn_child: Option<String> = None;
    let mut total_healed: i64 = 0;

    for effect in effects {
        match effect {
            ItemEffect::Heal { amount } => {
                let hp = global_facts.get_int("player:hp").unwrap_or(0);
                let hp_max = global_facts.get_int("player:hp_max").unwrap_or(20);
                let new_hp = (hp + *amount as i64).min(hp_max);
                total_healed += new_hp - hp;
                info!("FRE Bridge: Heal {} → HP {}/{}", amount, new_hp, hp_max);
                global_facts.set_global("player:hp", FactValue::Int(new_hp));
            }
            ItemEffect::PlayAudio { clip_path } => {
                audio::play_sound_full_path(audio, asset_server, clip_path);
            }
            ItemEffect::SpawnChildItem { item_id } => {
                spawn_child = Some(item_id.clone());
            }
            ItemEffect::SetFact { key, value } => {
                let fact_value: FactValue = value.clone().into();
                global_facts.set_global(key, fact_value);
            }
        }
    }

    // Handle inventory mutation: consume or replace with child item
    let mut inventory = global_facts
        .get_string_list("player:inventory")
        .map(|s| s.to_vec())
        .unwrap_or_default();
    if index < inventory.len() {
        if let Some(child_id) = spawn_child {
            info!(
                "FRE Bridge: SpawnChildItem '{}' at index {}",
                child_id, index
            );
            inventory[index] = child_id;
        } else if let ItemType::Food {
            consumable: true, ..
        } = &item.item_type
        {
            info!("FRE Bridge: Consuming item at index {}", index);
            inventory.remove(index);
        }
        global_facts.set_global("player:inventory", FactValue::StringList(inventory));
    }
    total_healed
}

/// Default OnUse node name for each item type.
fn default_use_node(item_type: &crate::core::item::ItemType) -> &'static str {
    use crate::core::item::ItemType;
    match item_type {
        ItemType::Food { .. } => "OnUseFoodDefault",
        ItemType::Weapon { .. } => "OnUseWeaponDefault",
        ItemType::Armor { .. } => "OnUseArmorDefault",
        ItemType::KeyItem => "OnUseKeyItemDefault",
    }
}

/// Default OnCheck node name for each item type.
fn default_check_node(item_type: &crate::core::item::ItemType) -> &'static str {
    use crate::core::item::ItemType;
    match item_type {
        ItemType::Food { .. } => "OnCheckFoodDefault",
        ItemType::Weapon { .. } => "OnCheckWeaponDefault",
        ItemType::Armor { .. } => "OnCheckArmorDefault",
        ItemType::KeyItem => "OnCheckDefault",
    }
}

/// UseItem action: dispatch by item type, execute effects, start dialogue.
fn execute_use_item(
    index_expr: &str,
    local_facts: &mut bevy_fact_rule_event::FactDatabase,
    global_facts: &mut bevy_fact_rule_event::LayeredFactDatabase,
    audio: &bevy_kira_audio::Audio,
    asset_server: &AssetServer,
    enum_registry: &EnumRegistry,
    item_registry: &crate::core::item::ItemRegistry,
    dialogue_view_default: &str,
    dialogue_voice_default: &str,
) {
    use crate::core::item::ItemType;

    let Some(index) = resolve_index_expr(index_expr, local_facts, global_facts, enum_registry)
    else {
        return;
    };
    let Some(item_id) = get_inventory_item_id(index, global_facts) else {
        warn!("FRE Bridge: UseItem — no item at index {}", index);
        return;
    };
    let Some(item) = item_registry.get(&item_id) else {
        warn!("FRE Bridge: UseItem — item '{}' not in registry", item_id);
        return;
    };

    info!(
        "FRE Bridge: UseItem '{}' (type: {:?}) at index {}",
        item_id,
        std::mem::discriminant(&item.item_type),
        index
    );

    let mut actual_healed: i64 = 0;

    match &item.item_type {
        ItemType::Food { .. } => {
            actual_healed = apply_item_effects(item, index, global_facts, audio, asset_server);
        }
        ItemType::Weapon { .. } => {
            // Swap: current weapon goes to inventory, new weapon gets equipped
            let old_weapon = global_facts
                .get_string("player:weapon")
                .unwrap_or_default()
                .to_string();
            global_facts.set_global("player:weapon", FactValue::String(item_id.clone()));
            let mut inventory = global_facts
                .get_string_list("player:inventory")
                .map(|s| s.to_vec())
                .unwrap_or_default();
            if index < inventory.len() {
                if old_weapon.is_empty() {
                    inventory.remove(index);
                } else {
                    inventory[index] = old_weapon;
                }
                global_facts.set_global("player:inventory", FactValue::StringList(inventory));
            }
            info!("FRE Bridge: Equipped weapon '{}'", item_id);
        }
        ItemType::Armor { .. } => {
            let old_armor = global_facts
                .get_string("player:armor")
                .unwrap_or_default()
                .to_string();
            global_facts.set_global("player:armor", FactValue::String(item_id.clone()));
            let mut inventory = global_facts
                .get_string_list("player:inventory")
                .map(|s| s.to_vec())
                .unwrap_or_default();
            if index < inventory.len() {
                if old_armor.is_empty() {
                    inventory.remove(index);
                } else {
                    inventory[index] = old_armor;
                }
                global_facts.set_global("player:inventory", FactValue::StringList(inventory));
            }
            info!("FRE Bridge: Equipped armor '{}'", item_id);
        }
        ItemType::KeyItem => {
            // No state change for key items
        }
    }

    let default_node = default_use_node(&item.item_type);
    let locale_key = format!("{}:{}", item.locale.file, item.locale.name);
    start_item_dialogue(
        item,
        "OnUse",
        default_node,
        global_facts,
        dialogue_view_default,
        dialogue_voice_default,
        ItemDialogueData {
            locale_key,
            description: item.description.clone(),
            heal_amount: actual_healed,
            item_value: compute_item_value(item),
        },
    );
}

/// CheckItem action: start dialogue with OnCheck node, no state change.
fn execute_check_item(
    index_expr: &str,
    local_facts: &bevy_fact_rule_event::FactDatabase,
    global_facts: &mut bevy_fact_rule_event::LayeredFactDatabase,
    enum_registry: &EnumRegistry,
    item_registry: &crate::core::item::ItemRegistry,
    dialogue_view_default: &str,
    dialogue_voice_default: &str,
) {
    let Some(index) = resolve_index_expr(index_expr, local_facts, global_facts, enum_registry)
    else {
        return;
    };
    let Some(item_id) = get_inventory_item_id(index, global_facts) else {
        warn!("FRE Bridge: CheckItem — no item at index {}", index);
        return;
    };
    let Some(item) = item_registry.get(&item_id) else {
        warn!("FRE Bridge: CheckItem — item '{}' not in registry", item_id);
        return;
    };

    info!("FRE Bridge: CheckItem '{}'", item_id);
    let default_node = default_check_node(&item.item_type);
    let locale_key = format!("{}:{}", item.locale.file, item.locale.name);
    // CheckItem always uses the default template (stats + description).
    // Per-item mortar files are only for OnUse/OnDrop custom behavior.
    //
    // CheckItem 始终使用默认模板（数值 + 描述）。
    // 每个物品的 mortar 文件仅用于 OnUse/OnDrop 自定义行为。
    start_item_dialogue_with_path(
        "items/_defaults.mortar",
        default_node,
        global_facts,
        dialogue_view_default,
        dialogue_voice_default,
        ItemDialogueData {
            locale_key,
            description: item.description.clone(),
            heal_amount: 0,
            item_value: compute_item_value(item),
        },
    );
}

/// DropItem action: remove from inventory and start OnDrop dialogue.
/// KeyItem is non-droppable — this action is silently ignored.
fn execute_drop_item(
    index_expr: &str,
    local_facts: &bevy_fact_rule_event::FactDatabase,
    global_facts: &mut bevy_fact_rule_event::LayeredFactDatabase,
    enum_registry: &EnumRegistry,
    item_registry: &crate::core::item::ItemRegistry,
    dialogue_view_default: &str,
    dialogue_voice_default: &str,
) {
    use crate::core::item::ItemType;

    let Some(index) = resolve_index_expr(index_expr, local_facts, global_facts, enum_registry)
    else {
        return;
    };
    let Some(item_id) = get_inventory_item_id(index, global_facts) else {
        warn!("FRE Bridge: DropItem — no item at index {}", index);
        return;
    };
    let Some(item) = item_registry.get(&item_id) else {
        warn!("FRE Bridge: DropItem — item '{}' not in registry", item_id);
        return;
    };

    if matches!(item.item_type, ItemType::KeyItem) {
        info!(
            "FRE Bridge: DropItem — '{}' is a KeyItem (non-droppable), ignoring",
            item_id
        );
        return;
    }

    info!("FRE Bridge: DropItem '{}' at index {}", item_id, index);
    let mut inventory = global_facts
        .get_string_list("player:inventory")
        .map(|s| s.to_vec())
        .unwrap_or_default();
    if index < inventory.len() {
        inventory.remove(index);
        global_facts.set_global("player:inventory", FactValue::StringList(inventory));
    }

    let locale_key = format!("{}:{}", item.locale.file, item.locale.name);
    start_item_dialogue(
        item,
        "OnDrop",
        "OnDropDefault",
        global_facts,
        dialogue_view_default,
        dialogue_voice_default,
        ItemDialogueData {
            locale_key,
            description: item.description.clone(),
            heal_amount: 0,
            item_value: compute_item_value(item),
        },
    );
}
///
/// 处理来自 ViewRoot.local_facts 的 SwitchState 请求的系统。
pub fn handle_switch_state_system(
    mut active_view_query: Query<&mut ViewRoot, With<ActiveView>>,
    mut next_state: ResMut<NextState<crate::app_state::SequenceSubState>>,
) {
    for mut view_root in active_view_query.iter_mut() {
        if let Some(FactValue::String(state_name)) = view_root
            .local_facts
            .get_by_str(fre_facts::VIEW_SWITCH_STATE)
        {
            let state_name = state_name.clone();
            info!("FRE Bridge: Switching to state '{}'", state_name);
            next_state.set(crate::app_state::SequenceSubState::new(&state_name));
            view_root.local_facts.remove(fre_facts::VIEW_SWITCH_STATE);
        }
    }
}

/// Dispatch a single Custom action via handler registry or as an event.
fn dispatch_single_custom_action(
    action: &GameActionDef,
    rule: &GameRule,
    fact_db: &LayeredFactDatabase,
    handler_registry: &GameActionHandlerRegistry,
    commands: &mut Commands,
    custom_action_writer: &mut MessageWriter<FreCustomActionEvent>,
) {
    let GameActionDef::Custom {
        action_type,
        params,
    } = action
    else {
        return;
    };

    if handler_registry.has_handler(action_type) {
        debug!(
            "FRE: Executing registered handler for '{}' (rule: '{}')",
            action_type, rule.id
        );
        handler_registry.execute(action, fact_db, commands);
    } else {
        debug!(
            "FRE: Dispatching unhandled custom action '{}' (rule: '{}')",
            action_type, rule.id
        );
        custom_action_writer.write(FreCustomActionEvent {
            action_type: action_type.clone(),
            params: params.clone(),
        });
    }
}

/// Process all matching rules for a single FRE event, dispatching Custom actions.
fn dispatch_event_custom_actions(
    event: &FactEvent,
    rule_registry: &GameRuleRegistry,
    fact_db: &LayeredFactDatabase,
    handler_registry: &GameActionHandlerRegistry,
    commands: &mut Commands,
    custom_action_writer: &mut MessageWriter<FreCustomActionEvent>,
    enum_registry: &EnumRegistry,
) {
    let rule_groups = rule_registry.get_matching_rules_grouped(event);
    if rule_groups.is_empty() {
        return;
    }

    'outer: for group in rule_groups {
        for rule in group {
            if !evaluate_conditions(&rule.condition_expressions, fact_db, enum_registry) {
                continue;
            }

            for action in &rule.actions {
                dispatch_single_custom_action(
                    action,
                    rule,
                    fact_db,
                    handler_registry,
                    commands,
                    custom_action_writer,
                );
            }

            if rule.consume_event {
                break 'outer;
            }
        }
    }
}

/// System that reads FRE events, matches rules, evaluates global conditions,
/// and dispatches Custom actions as `FreCustomActionEvent` messages.
///
/// This is the unified global dispatch for Custom actions, replacing
/// per-subsystem rule evaluation (handle_chase_state_actions_system,
/// collect_danmaku_actions_system, etc.).
///
/// FRE 事件全局 Custom action 分发系统。
/// 读取 FRE 事件，匹配规则，评估全局条件，
/// 将 Custom action 作为 `FreCustomActionEvent` 消息分发。
pub fn dispatch_custom_actions_system(
    mut events: MessageReader<FactEvent>,
    rule_registry: Res<GameRuleRegistry>,
    fact_db: Res<LayeredFactDatabase>,
    handler_registry: Res<GameActionHandlerRegistry>,
    mut commands: Commands,
    mut custom_action_writer: MessageWriter<FreCustomActionEvent>,
    enum_registry: Res<EnumRegistry>,
) {
    for event in events.read() {
        dispatch_event_custom_actions(
            event,
            &rule_registry,
            &fact_db,
            &handler_registry,
            &mut commands,
            &mut custom_action_writer,
            &enum_registry,
        );
    }
}

/// Plugin for FRE-View bridge systems.
///
/// FRE-View 桥接系统的插件。
pub struct FREBridgePlugin;

impl Plugin for FREBridgePlugin {
    fn build(&self, app: &mut App) {
        let schedule = crate::game_schedule(app);
        app.add_message::<FreCustomActionEvent>()
            .add_systems(Startup, register_condition_evaluator_system)
            .add_systems(
                schedule,
                (
                    sync_state_to_facts_system.run_if(state_facts_need_sync),
                    action_to_fre_event_system,
                    process_view_actions_system,
                    dispatch_custom_actions_system,
                    handle_switch_state_system,
                )
                    .chain(),
            );
    }
}
