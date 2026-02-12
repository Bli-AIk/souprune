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

use bevy::prelude::*;
use bevy_fact_rule_event::{
    FactEvent, FactValue, LayeredRuleRegistry, LocalFactValue, RuleActionDef,
};
use leafwing_input_manager::action_state::ActionState;

use crate::app_state::overworld::trigger::RuleActionDefs;
use crate::core::audio;
use crate::core::input::{Action, ActionRegistry, ActionStateExt};
use crate::core::view::components::{ActiveView, ViewRoot};

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
        // JustPressed events
        if action_state.action_just_pressed(&registry, action_name) {
            let event_id = format!("action:{}:just_pressed", action_name);
            debug!(
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
            let event_id = format!("action:{}:just_released", action_name);
            debug!(
                "FRE Bridge: {} just_released, emitting {}",
                action_name, event_id
            );
            event_writer.write(FactEvent::new(event_id));
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
#[cfg(all(feature = "bevy_kira_audio", not(feature = "firewheel")))]
pub fn process_view_actions_system(
    mut events: MessageReader<FactEvent>,
    rule_registry: Res<LayeredRuleRegistry>,
    action_defs: Option<Res<RuleActionDefs>>,
    mut active_view_query: Query<&mut ViewRoot, With<ActiveView>>,
    audio: Res<bevy_kira_audio::Audio>,
    asset_server: Res<AssetServer>,
    global_facts: Res<bevy_fact_rule_event::LayeredFactDatabase>,
    #[cfg(feature = "debug")] mut trigger_history: Option<
        ResMut<crate::extra::debug::RuleTriggerHistory>,
    >,
    #[cfg(feature = "debug")] time: Res<Time>,
) {
    let Some(action_defs) = action_defs else {
        return;
    };

    // Collect all events first to avoid borrow issues
    let events_to_process: Vec<FactEvent> = events.read().cloned().collect();

    for event in &events_to_process {
        // Get all matching rules for this event, grouped by priority
        let rule_groups = rule_registry.get_matching_rules_grouped(event);

        // Debug: Log when we have matching rules for Left/Right events
        if event.id.0.contains("Left") || event.id.0.contains("Right") {
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

        if rule_groups.is_empty() {
            continue;
        }

        // Process rules by priority groups
        // 按优先级分组处理规则
        'outer: for group in rule_groups {
            for rule in group {
                // Get the active ViewRoot to check conditions
                let Ok(mut view_root) = active_view_query.single_mut() else {
                    info!("FRE Bridge: No ActiveView found for rule '{}'", rule.id);
                    continue;
                };

                // Sync dynamic facts from global database before condition evaluation
                // 在条件评估前同步动态 facts
                sync_dynamic_facts(&mut view_root.local_facts, &global_facts);

                // Check condition expressions against local_facts and global_facts
                if !evaluate_conditions(
                    &rule.condition_expressions,
                    &view_root.local_facts,
                    &global_facts,
                ) {
                    // Only log for ACT navigation rules (depth 2 related)
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
                    continue;
                }

                info!(
                    "FRE Bridge: Rule '{}' matched event '{}' (priority: {}, conditions: {}), executing actions",
                    rule.id,
                    event.id.0,
                    rule.priority,
                    rule.condition_expressions.len()
                );

                // Record rule trigger for debug panel visualization
                // 记录规则触发以供调试面板可视化
                #[cfg(feature = "debug")]
                if let Some(ref mut history) = trigger_history {
                    history.record_trigger(&rule.id, time.elapsed_secs_f64());
                }

                // Look up the original action definitions for this rule
                let Some(actions) = action_defs.actions_by_rule.get(&rule.id) else {
                    warn!(
                        "FRE Bridge: No actions found for rule '{}' in action_defs (available: {:?})",
                        rule.id,
                        action_defs
                            .actions_by_rule
                            .keys()
                            .take(5)
                            .collect::<Vec<_>>()
                    );
                    continue;
                };

                // Execute each action (view_root is already mutable)
                for action in actions {
                    execute_action(action, &mut view_root.local_facts, &audio, &asset_server);
                }

                // If this rule consumes the event, stop all matching
                // 如果此规则消费事件，停止所有匹配
                if rule.consume_event {
                    break 'outer;
                }
                // Otherwise, continue checking in this priority group
            }
        }
    }
}

/// System that processes FRE actions that affect ViewRoot.local_facts (Firewheel variant).
///
/// 处理影响 ViewRoot.local_facts 的 FRE 动作的系统（Firewheel 变体）。
#[cfg(feature = "firewheel")]
pub fn process_view_actions_system(
    mut events: MessageReader<FactEvent>,
    rule_registry: Res<LayeredRuleRegistry>,
    action_defs: Option<Res<RuleActionDefs>>,
    mut active_view_query: Query<&mut ViewRoot, With<ActiveView>>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    global_facts: Res<bevy_fact_rule_event::LayeredFactDatabase>,
    #[cfg(feature = "debug")] mut trigger_history: Option<
        ResMut<crate::extra::debug::RuleTriggerHistory>,
    >,
    #[cfg(feature = "debug")] time: Res<Time>,
) {
    let Some(action_defs) = action_defs else {
        return;
    };

    let events_to_process: Vec<FactEvent> = events.read().cloned().collect();

    for event in &events_to_process {
        // Get all matching rules for this event, grouped by priority
        let rule_groups = rule_registry.get_matching_rules_grouped(event);

        if rule_groups.is_empty() {
            continue;
        }

        // Process rules by priority groups
        // 按优先级分组处理规则
        'outer: for group in rule_groups {
            for rule in group {
                let Ok(mut view_root) = active_view_query.single_mut() else {
                    info!("FRE Bridge: No ActiveView found for rule '{}'", rule.id);
                    continue;
                };

                // Sync dynamic facts from global database before condition evaluation
                sync_dynamic_facts(&mut view_root.local_facts, &global_facts);

                if !evaluate_conditions(
                    &rule.condition_expressions,
                    &view_root.local_facts,
                    &global_facts,
                ) {
                    debug!(
                        "FRE Bridge: Conditions not met for rule '{}', local_facts: depth={:?}, selection={:?}",
                        rule.id,
                        view_root.local_facts.get_int("depth"),
                        view_root.local_facts.get_int("selection")
                    );
                    continue;
                }

                info!(
                    "FRE Bridge: Rule '{}' matched event '{}' (priority: {}, conditions: {}), executing actions",
                    rule.id,
                    event.id.0,
                    rule.priority,
                    rule.condition_expressions.len()
                );

                // Record rule trigger for debug panel visualization
                // 记录规则触发以供调试面板可视化
                #[cfg(feature = "debug")]
                if let Some(ref mut history) = trigger_history {
                    history.record_trigger(&rule.id, time.elapsed_secs_f64());
                }

                let Some(actions) = action_defs.actions_by_rule.get(&rule.id) else {
                    warn!(
                        "FRE Bridge: No actions found for rule '{}' in action_defs (available: {:?})",
                        rule.id,
                        action_defs
                            .actions_by_rule
                            .keys()
                            .take(5)
                            .collect::<Vec<_>>()
                    );
                    continue;
                };

                for action in actions {
                    execute_action_firewheel(
                        action,
                        &mut view_root.local_facts,
                        &mut commands,
                        &asset_server,
                    );
                }

                // If this rule consumes the event, stop all matching
                // 如果此规则消费事件，停止所有匹配
                if rule.consume_event {
                    break 'outer;
                }
                // Otherwise, continue checking in this priority group
            }
        }
    }
}

/// Syncs derived facts from global database to local_facts.
/// This allows rules to reference global data in conditions.
///
/// 将派生 facts 从全局数据库同步到 local_facts。
/// 这允许规则在条件中引用全局数据。
///
/// NOTE: This function is now minimal - StringList.len() is evaluated
/// directly via $var.len() syntax in conditions.
///
/// 注意：此函数现在是最小化的 - StringList.len() 通过条件中的
/// $var.len() 语法直接评估。
fn sync_dynamic_facts(
    _local_facts: &mut bevy_fact_rule_event::FactDatabase,
    _global_facts: &bevy_fact_rule_event::LayeredFactDatabase,
) {
    // Currently no facts need to be synced.
    // StringList.len() is evaluated directly via $var.len() syntax.
}

/// Evaluate condition expressions against local and global facts.
/// Returns true if all conditions pass.
///
/// 根据本地和全局 FactDatabase 评估条件表达式。
/// 如果所有条件都通过则返回 true。
fn evaluate_conditions(
    conditions: &[String],
    local_facts: &bevy_fact_rule_event::FactDatabase,
    global_facts: &bevy_fact_rule_event::LayeredFactDatabase,
) -> bool {
    for condition in conditions {
        let result = evaluate_single_condition(condition, local_facts, global_facts);
        if !result {
            // Log failed condition for debugging
            debug!("FRE Bridge: Condition '{}' evaluated to false", condition);
            return false;
        }
    }
    true
}

/// Evaluate a single condition expression.
///
/// Supports:
/// - `$name == N` - equality check
/// - `$name > N`, `$name >= N` - greater than
/// - `$name < N`, `$name <= N` - less than
/// - `$name == true`, `$name == false` - boolean check
/// - `$name.len()` - get length of StringList
///
/// Variable resolution priority: local_facts -> global_facts
///
/// 评估单个条件表达式。
///
/// 支持：
/// - `$name == N` - 相等检查
/// - `$name > N`, `$name >= N` - 大于
/// - `$name < N`, `$name <= N` - 小于
/// - `$name == true`, `$name == false` - 布尔检查
/// - `$name.len()` - 获取 StringList 的长度
///
/// 变量解析优先级：local_facts -> global_facts
pub fn evaluate_single_condition(
    condition: &str,
    local_facts: &bevy_fact_rule_event::FactDatabase,
    global_facts: &bevy_fact_rule_event::LayeredFactDatabase,
) -> bool {
    let condition = condition.trim();

    // Try to parse comparison operators
    if let Some(idx) = condition.find(" == ") {
        let left = condition[..idx].trim();
        let right = condition[idx + 4..].trim();
        // Use compare_ints if left side contains arithmetic operators (%, +, -)
        // 如果左侧包含算术运算符（%, +, -），使用 compare_ints
        if left.contains(" % ") || left.contains(" + ") || left.contains(" - ") {
            return compare_ints(left, right, local_facts, global_facts, |a, b| a == b);
        }
        return compare_values(left, right, local_facts, global_facts, |a, b| a == b);
    }

    if let Some(idx) = condition.find(" != ") {
        let left = condition[..idx].trim();
        let right = condition[idx + 4..].trim();
        // Use compare_ints if left side contains arithmetic operators
        if left.contains(" % ") || left.contains(" + ") || left.contains(" - ") {
            return compare_ints(left, right, local_facts, global_facts, |a, b| a != b);
        }
        return compare_values(left, right, local_facts, global_facts, |a, b| a != b);
    }

    if let Some(idx) = condition.find(" >= ") {
        let left = condition[..idx].trim();
        let right = condition[idx + 4..].trim();
        return compare_ints(left, right, local_facts, global_facts, |a, b| a >= b);
    }

    if let Some(idx) = condition.find(" <= ") {
        let left = condition[..idx].trim();
        let right = condition[idx + 4..].trim();
        return compare_ints(left, right, local_facts, global_facts, |a, b| a <= b);
    }

    if let Some(idx) = condition.find(" > ") {
        let left = condition[..idx].trim();
        let right = condition[idx + 3..].trim();
        return compare_ints(left, right, local_facts, global_facts, |a, b| a > b);
    }

    if let Some(idx) = condition.find(" < ") {
        let left = condition[..idx].trim();
        let right = condition[idx + 3..].trim();
        return compare_ints(left, right, local_facts, global_facts, |a, b| a < b);
    }

    // Default: assume it's a boolean variable that should be true
    if let Some(var_name) = condition.strip_prefix('$') {
        // Check local_facts first, then global_facts
        if let Some(val) = local_facts.get_bool(var_name) {
            return val;
        }
        return global_facts.get_bool(var_name).unwrap_or(false);
    }

    warn!("FRE Bridge: Unknown condition format: {}", condition);
    false
}

fn compare_values<F>(
    left: &str,
    right: &str,
    local_facts: &bevy_fact_rule_event::FactDatabase,
    global_facts: &bevy_fact_rule_event::LayeredFactDatabase,
    cmp: F,
) -> bool
where
    F: Fn(&FactValue, &FactValue) -> bool,
{
    let left_val = resolve_value(left, local_facts, global_facts);
    let right_val = resolve_value(right, local_facts, global_facts);

    match (left_val, right_val) {
        (Some(l), Some(r)) => cmp(&l, &r),
        _ => false,
    }
}

fn compare_ints<F>(
    left: &str,
    right: &str,
    local_facts: &bevy_fact_rule_event::FactDatabase,
    global_facts: &bevy_fact_rule_event::LayeredFactDatabase,
    cmp: F,
) -> bool
where
    F: Fn(i64, i64) -> bool,
{
    let left_val = resolve_int(left, local_facts, global_facts);
    let right_val = resolve_int(right, local_facts, global_facts);

    match (left_val, right_val) {
        (Some(l), Some(r)) => cmp(l, r),
        _ => false,
    }
}

fn resolve_value(
    expr: &str,
    local_facts: &bevy_fact_rule_event::FactDatabase,
    global_facts: &bevy_fact_rule_event::LayeredFactDatabase,
) -> Option<FactValue> {
    // Check for .len() suffix (e.g., $player_inventory.len())
    if let Some(base) = expr.strip_suffix(".len()") {
        if let Some(var_name) = base.strip_prefix('$') {
            // Check local_facts first
            if let Some(val) = local_facts.get_by_str(var_name)
                && let FactValue::StringList(list) = val
            {
                return Some(FactValue::Int(list.len() as i64));
            }
            // Check global_facts
            if let Some(val) = global_facts.get_by_str(var_name)
                && let FactValue::StringList(list) = val
            {
                return Some(FactValue::Int(list.len() as i64));
            }
        }
        return None;
    }

    if let Some(var_name) = expr.strip_prefix('$') {
        // Check local_facts first
        if let Some(val) = local_facts.get_by_str(var_name) {
            return Some(val.clone());
        }
        // Check global_facts
        return global_facts.get_by_str(var_name).cloned();
    }

    // Try parsing as integer
    if let Ok(v) = expr.parse::<i64>() {
        return Some(FactValue::Int(v));
    }

    // Try parsing as boolean
    if expr == "true" {
        return Some(FactValue::Bool(true));
    }
    if expr == "false" {
        return Some(FactValue::Bool(false));
    }

    // Try parsing as float
    if let Ok(v) = expr.parse::<f64>() {
        return Some(FactValue::Float(v));
    }

    None
}

fn resolve_int(
    expr: &str,
    local_facts: &bevy_fact_rule_event::FactDatabase,
    global_facts: &bevy_fact_rule_event::LayeredFactDatabase,
) -> Option<i64> {
    let expr = expr.trim();

    // Check for expression patterns: $var + N, $var - N, $var % N
    // 支持的表达式模式：$var + N, $var - N, $var % N
    if let Some(idx) = expr.find(" + ") {
        let left = expr[..idx].trim();
        let right = expr[idx + 3..].trim();
        let left_val = resolve_int(left, local_facts, global_facts)?;
        let right_val = resolve_int(right, local_facts, global_facts)?;
        return Some(left_val + right_val);
    }

    if let Some(idx) = expr.find(" - ") {
        let left = expr[..idx].trim();
        let right = expr[idx + 3..].trim();
        let left_val = resolve_int(left, local_facts, global_facts)?;
        let right_val = resolve_int(right, local_facts, global_facts)?;
        return Some(left_val - right_val);
    }

    // Modulo operation (e.g., $act_selection % 2)
    // 模运算（例如 $act_selection % 2）
    if let Some(idx) = expr.find(" % ") {
        let left = expr[..idx].trim();
        let right = expr[idx + 3..].trim();
        let left_val = resolve_int(left, local_facts, global_facts)?;
        let right_val = resolve_int(right, local_facts, global_facts)?;
        if right_val != 0 {
            return Some(left_val % right_val);
        }
        return None; // Avoid division by zero
    }

    // Check for .len() suffix (e.g., $player_inventory.len())
    if let Some(base) = expr.strip_suffix(".len()") {
        if let Some(var_name) = base.strip_prefix('$') {
            // Check local_facts first
            if let Some(val) = local_facts.get_by_str(var_name)
                && let FactValue::StringList(list) = val
            {
                return Some(list.len() as i64);
            }
            // Check global_facts
            if let Some(val) = global_facts.get_by_str(var_name)
                && let FactValue::StringList(list) = val
            {
                return Some(list.len() as i64);
            }
        }
        return None;
    }

    // Simple variable reference
    if let Some(var_name) = expr.strip_prefix('$') {
        // Check local_facts first
        if let Some(val) = local_facts.get_int(var_name) {
            return Some(val);
        }
        // Check if it's a StringList - return its length
        // This allows conditions like `$enemy_selection < $enemy_names - 1`
        // where $enemy_names is a StringList
        if let Some(val) = local_facts.get_by_str(var_name)
            && let FactValue::StringList(list) = val
        {
            return Some(list.len() as i64);
        }
        if let Some(val) = local_facts.get_by_str(var_name)
            && let FactValue::IntList(list) = val
        {
            return Some(list.len() as i64);
        }
        // Check global_facts
        if let Some(val) = global_facts.get_int(var_name) {
            return Some(val);
        }
        // Check if it's a StringList in global_facts
        if let Some(val) = global_facts.get_by_str(var_name)
            && let FactValue::StringList(list) = val
        {
            return Some(list.len() as i64);
        }
        if let Some(val) = global_facts.get_by_str(var_name)
            && let FactValue::IntList(list) = val
        {
            return Some(list.len() as i64);
        }
        return None;
    }

    expr.parse::<i64>().ok()
}

/// Execute a single FRE action on the ViewRoot's local_facts.
///
/// 在 ViewRoot 的 local_facts 上执行单个 FRE 动作。
#[cfg(all(feature = "bevy_kira_audio", not(feature = "firewheel")))]
fn execute_action(
    action: &RuleActionDef,
    local_facts: &mut bevy_fact_rule_event::FactDatabase,
    audio: &bevy_kira_audio::Audio,
    asset_server: &AssetServer,
) {
    match action {
        RuleActionDef::PlaySound(sound_name) => {
            debug!("FRE Bridge: PlaySound({})", sound_name);
            audio::play_sound(audio, asset_server, sound_name);
        }
        RuleActionDef::PlaySoundFullPath(path) => {
            debug!("FRE Bridge: PlaySoundFullPath({})", path);
            audio::play_sound_full_path(audio, asset_server, path);
        }
        RuleActionDef::SetLocalFact(key, value) => {
            let fact_value = evaluate_local_fact_value(value, local_facts);
            debug!("FRE Bridge: SetLocalFact({}, {:?})", key, fact_value);
            local_facts.set(key.as_str(), fact_value);
        }
        RuleActionDef::CloseView => {
            debug!("FRE Bridge: CloseView");
            // Set a fact that the View system can react to
            local_facts.set("_close_requested", FactValue::Bool(true));
        }
        RuleActionDef::SwitchState(state_name) => {
            debug!("FRE Bridge: SwitchState({})", state_name);
            // Set a fact that the state transition system can react to
            local_facts.set("_switch_state", FactValue::String(state_name.clone()));
        }
        RuleActionDef::EmitEvent(event_id) => {
            debug!("FRE Bridge: EmitEvent({})", event_id);
            // Events are handled by the rule outputs, not here
        }
        RuleActionDef::Custom {
            action_type,
            params,
        } => {
            debug!(
                "FRE Bridge: Custom action {} with params {:?}",
                action_type, params
            );
            // Custom actions are handled by other systems (e.g., PlayDanmaku)
        }
        // Actions not relevant for View local_facts - handled by global FRE system
        RuleActionDef::Log { message } => {
            info!("FRE Bridge: Log: {}", message);
        }
        RuleActionDef::SetResource { .. } | RuleActionDef::SpawnEntity { .. } => {
            // These are handled by the global FRE system, not View-specific
        }
    }
}

/// Execute a single FRE action on the ViewRoot's local_facts (Firewheel variant).
///
/// 在 ViewRoot 的 local_facts 上执行单个 FRE 动作（Firewheel 变体）。
#[cfg(feature = "firewheel")]
fn execute_action_firewheel(
    action: &RuleActionDef,
    local_facts: &mut bevy_fact_rule_event::FactDatabase,
    commands: &mut Commands,
    asset_server: &AssetServer,
) {
    match action {
        RuleActionDef::PlaySound(sound_name) => {
            debug!("FRE Bridge: PlaySound({})", sound_name);
            audio::play_sound(commands, asset_server, sound_name);
        }
        RuleActionDef::PlaySoundFullPath(path) => {
            debug!("FRE Bridge: PlaySoundFullPath({})", path);
            audio::play_sound_full_path(commands, asset_server, path);
        }
        RuleActionDef::SetLocalFact(key, value) => {
            let fact_value = evaluate_local_fact_value(value, local_facts);
            debug!("FRE Bridge: SetLocalFact({}, {:?})", key, fact_value);
            local_facts.set(key.as_str(), fact_value);
        }
        RuleActionDef::CloseView => {
            debug!("FRE Bridge: CloseView");
            local_facts.set("_close_requested", FactValue::Bool(true));
        }
        RuleActionDef::SwitchState(state_name) => {
            debug!("FRE Bridge: SwitchState({})", state_name);
            local_facts.set("_switch_state", FactValue::String(state_name.clone()));
        }
        RuleActionDef::EmitEvent(event_id) => {
            debug!("FRE Bridge: EmitEvent({})", event_id);
        }
        RuleActionDef::Custom {
            action_type,
            params,
        } => {
            debug!(
                "FRE Bridge: Custom action {} with params {:?}",
                action_type, params
            );
        }
        // Actions not relevant for View local_facts - handled by global FRE system
        RuleActionDef::Log { message } => {
            info!("FRE Bridge: Log: {}", message);
        }
        RuleActionDef::SetResource { .. } | RuleActionDef::SpawnEntity { .. } => {
            // These are handled by the global FRE system, not View-specific
        }
    }
}

/// Evaluate a LocalFactValue to a FactValue.
///
/// 将 LocalFactValue 评估为 FactValue。
fn evaluate_local_fact_value(
    value: &LocalFactValue,
    facts: &bevy_fact_rule_event::FactDatabase,
) -> FactValue {
    match value {
        LocalFactValue::Int(v) => FactValue::Int(*v),
        LocalFactValue::Float(v) => FactValue::Float(*v),
        LocalFactValue::Bool(v) => FactValue::Bool(*v),
        LocalFactValue::String(v) => FactValue::String(v.clone()),
        LocalFactValue::Expr(expr) => {
            if let Some(result) = evaluate_simple_expression(expr, facts) {
                result
            } else {
                warn!(
                    "FRE Bridge: Failed to evaluate expression '{}', using 0",
                    expr
                );
                FactValue::Int(0)
            }
        }
    }
}

/// Simple expression evaluator for basic arithmetic.
///
/// Supports:
/// - `$name` - local fact reference
/// - `$name + 1`, `$name - 1` - simple increment/decrement
///
/// 简单的表达式评估器，用于基本算术。
///
/// 支持：
/// - `$name` - 局部 fact 引用
/// - `$name + 1`、`$name - 1` - 简单增减
fn evaluate_simple_expression(
    expr: &str,
    facts: &bevy_fact_rule_event::FactDatabase,
) -> Option<FactValue> {
    let expr = expr.trim();

    // Handle simple variable reference: $name
    if expr.starts_with('$') && !expr.contains(' ') {
        let var_name = &expr[1..];
        return facts.get_by_str(var_name).cloned();
    }

    // Handle simple addition: $name + N
    if let Some(idx) = expr.find(" + ") {
        let left = expr[..idx].trim();
        let right = expr[idx + 3..].trim();

        if let Some(var_name) = left.strip_prefix('$')
            && let Some(FactValue::Int(left_val)) = facts.get_by_str(var_name)
            && let Ok(right_val) = right.parse::<i64>()
        {
            return Some(FactValue::Int(left_val + right_val));
        }
    }

    // Handle simple subtraction: $name - N
    if let Some(idx) = expr.find(" - ") {
        let left = expr[..idx].trim();
        let right = expr[idx + 3..].trim();

        if let Some(var_name) = left.strip_prefix('$')
            && let Some(FactValue::Int(left_val)) = facts.get_by_str(var_name)
            && let Ok(right_val) = right.parse::<i64>()
        {
            return Some(FactValue::Int(left_val - right_val));
        }
    }

    // Try parsing as literal integer
    if let Ok(v) = expr.parse::<i64>() {
        return Some(FactValue::Int(v));
    }

    None
}

/// System to handle SwitchState requests from ViewRoot.local_facts.
///
/// 处理来自 ViewRoot.local_facts 的 SwitchState 请求的系统。
pub fn handle_switch_state_system(
    mut active_view_query: Query<&mut ViewRoot, With<ActiveView>>,
    mut next_state: ResMut<NextState<crate::app_state::overworld::OverworldSubState>>,
) {
    for mut view_root in active_view_query.iter_mut() {
        if let Some(FactValue::String(state_name)) =
            view_root.local_facts.get_by_str("_switch_state")
        {
            let state_name = state_name.clone();
            info!("FRE Bridge: Switching to state '{}'", state_name);
            next_state.set(crate::app_state::overworld::OverworldSubState::new(
                &state_name,
            ));
            view_root.local_facts.remove("_switch_state");
        }
    }
}

/// Plugin for FRE-View bridge systems.
///
/// FRE-View 桥接系统的插件。
pub struct FREBridgePlugin;

impl Plugin for FREBridgePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                action_to_fre_event_system,
                process_view_actions_system,
                handle_switch_state_system,
            )
                .chain(),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_fact_rule_event::{FactDatabase, LayeredFactDatabase};

    #[test]
    fn test_evaluate_simple_expression_variable() {
        let mut facts = FactDatabase::new();
        facts.set("selection", FactValue::Int(3));

        let result = evaluate_simple_expression("$selection", &facts);
        assert_eq!(result, Some(FactValue::Int(3)));
    }

    #[test]
    fn test_evaluate_simple_expression_addition() {
        let mut facts = FactDatabase::new();
        facts.set("selection", FactValue::Int(3));

        let result = evaluate_simple_expression("$selection + 1", &facts);
        assert_eq!(result, Some(FactValue::Int(4)));
    }

    #[test]
    fn test_evaluate_simple_expression_subtraction() {
        let mut facts = FactDatabase::new();
        facts.set("selection", FactValue::Int(3));

        let result = evaluate_simple_expression("$selection - 1", &facts);
        assert_eq!(result, Some(FactValue::Int(2)));
    }

    #[test]
    fn test_evaluate_simple_expression_literal() {
        let facts = FactDatabase::new();

        let result = evaluate_simple_expression("42", &facts);
        assert_eq!(result, Some(FactValue::Int(42)));
    }

    #[test]
    fn test_evaluate_single_condition_equals() {
        let mut facts = FactDatabase::new();
        let global_facts = LayeredFactDatabase::default();
        facts.set("depth", FactValue::Int(0));

        assert!(evaluate_single_condition(
            "$depth == 0",
            &facts,
            &global_facts
        ));
        assert!(!evaluate_single_condition(
            "$depth == 1",
            &facts,
            &global_facts
        ));
    }

    #[test]
    fn test_evaluate_single_condition_greater_than() {
        let mut facts = FactDatabase::new();
        let global_facts = LayeredFactDatabase::default();
        facts.set("selection", FactValue::Int(3));

        assert!(evaluate_single_condition(
            "$selection > 0",
            &facts,
            &global_facts
        ));
        assert!(!evaluate_single_condition(
            "$selection > 3",
            &facts,
            &global_facts
        ));
    }

    #[test]
    fn test_evaluate_single_condition_less_than() {
        let mut facts = FactDatabase::new();
        let global_facts = LayeredFactDatabase::default();
        facts.set("selection", FactValue::Int(3));

        assert!(evaluate_single_condition(
            "$selection < 5",
            &facts,
            &global_facts
        ));
        assert!(!evaluate_single_condition(
            "$selection < 3",
            &facts,
            &global_facts
        ));
    }

    #[test]
    fn test_evaluate_conditions_all_pass() {
        let mut facts = FactDatabase::new();
        let global_facts = LayeredFactDatabase::default();
        facts.set("depth", FactValue::Int(0));
        facts.set("selection", FactValue::Int(1));

        let conditions = vec!["$depth == 0".to_string(), "$selection < 5".to_string()];

        assert!(evaluate_conditions(&conditions, &facts, &global_facts));
    }

    #[test]
    fn test_evaluate_conditions_one_fails() {
        let mut facts = FactDatabase::new();
        let global_facts = LayeredFactDatabase::default();
        facts.set("depth", FactValue::Int(0));
        facts.set("selection", FactValue::Int(1));

        let conditions = vec![
            "$depth == 1".to_string(), // This fails
            "$selection < 5".to_string(),
        ];

        assert!(!evaluate_conditions(&conditions, &facts, &global_facts));
    }

    #[test]
    fn test_evaluate_string_list_len() {
        let local_facts = FactDatabase::new();
        let mut global_facts = LayeredFactDatabase::default();
        global_facts.set_global(
            "player_inventory",
            FactValue::StringList(vec![
                "item1".to_string(),
                "item2".to_string(),
                "item3".to_string(),
            ]),
        );

        // Test $player_inventory.len() resolves to 3
        let conditions = vec!["$player_inventory.len() == 3".to_string()];
        assert!(evaluate_conditions(
            &conditions,
            &local_facts,
            &global_facts
        ));

        // Test comparison with arithmetic
        let conditions2 = vec!["$player_inventory.len() > 2".to_string()];
        assert!(evaluate_conditions(
            &conditions2,
            &local_facts,
            &global_facts
        ));
    }
}
