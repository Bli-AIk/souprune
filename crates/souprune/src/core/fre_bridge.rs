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
/// This allows FRE rules to check current state using `$@overworld_state` and `$@app_state`.
/// The `@` prefix indicates these are state-derived facts rather than user-defined facts.
///
/// 将 Bevy 状态值同步到 FRE facts 的系统。
///
/// 这允许 FRE 规则使用 `$@overworld_state` 和 `$@app_state` 检查当前状态。
/// `@` 前缀表示这些是状态派生的 facts，而不是用户定义的 facts。
/// Run condition: Check if any state has changed
/// 运行条件：检查是否有任何状态变化
fn state_facts_need_sync(
    overworld_state: Option<Res<State<crate::app_state::overworld::OverworldSubState>>>,
    app_state: Option<Res<State<crate::app_state::AppState>>>,
) -> bool {
    // Check if either state has changed this frame
    if let Some(ref state) = overworld_state {
        if state.is_changed() {
            return true;
        }
    }
    if let Some(ref state) = app_state {
        if state.is_changed() {
            return true;
        }
    }
    false
}

pub fn sync_state_to_facts_system(
    overworld_state: Option<Res<State<crate::app_state::overworld::OverworldSubState>>>,
    app_state: Option<Res<State<crate::app_state::AppState>>>,
    mut facts: ResMut<bevy_fact_rule_event::LayeredFactDatabase>,
) {
    // Sync OverworldSubState
    if let Some(state) = overworld_state {
        let state_name = state.get().name().to_string();
        facts.set("@overworld_state", FactValue::String(state_name));
    }

    // Sync AppState
    if let Some(state) = app_state {
        let state_name = format!("{:?}", state.get());
        facts.set("@app_state", FactValue::String(state_name));
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
    mut pending_events: ResMut<bevy_fact_rule_event::PendingFactEvents>,
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
                #[cfg(feature = "debug")]
                if let Some(ref mut history) = trigger_history {
                    history.record_trigger(&rule.id, time.elapsed_secs_f64());
                }

                // Look up the original action definitions for this rule
                // Rules may have modifications without actions, so process both
                // 查找此规则的原始动作定义
                // 规则可能只有 modifications 而没有 actions，所以两者都要处理
                let actions = action_defs.actions_by_rule.get(&rule.id);

                // Execute each action (view_root is already mutable)
                if let Some(actions) = actions {
                    for action in actions {
                        execute_action(
                            action,
                            &mut view_root.local_facts,
                            &global_facts,
                            &audio,
                            &asset_server,
                        );
                    }
                }

                // Queue output events with deduplication.
                // This system can correctly evaluate conditions with local facts,
                // so it should handle outputs for rules that reference local facts.
                // The queue_output method ensures no duplicates if process_rules_system
                // already queued the same output.
                //
                // 使用去重队列输出事件。
                // 此系统可以正确评估包含 local facts 的条件，
                // 因此它应该处理引用 local facts 的规则的 outputs。
                // queue_output 方法确保如果 process_rules_system 已排队相同输出则不会重复。
                for output_id in &rule.outputs {
                    pending_events.queue_output(&rule.id, FactEvent::new(output_id.clone()));
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
    mut pending_events: ResMut<bevy_fact_rule_event::PendingFactEvents>,
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
                #[cfg(feature = "debug")]
                if let Some(ref mut history) = trigger_history {
                    history.record_trigger(&rule.id, time.elapsed_secs_f64());
                }

                // Look up the original action definitions for this rule
                // Rules may have modifications without actions, so process both
                // 查找此规则的原始动作定义
                // 规则可能只有 modifications 而没有 actions，所以两者都要处理
                let actions = action_defs.actions_by_rule.get(&rule.id);

                if let Some(actions) = actions {
                    for action in actions {
                        execute_action_firewheel(
                            action,
                            &mut view_root.local_facts,
                            &global_facts,
                            &mut commands,
                            &asset_server,
                        );
                    }
                }

                // Queue output events with deduplication.
                // This system can correctly evaluate conditions with local facts,
                // so it should handle outputs for rules that reference local facts.
                // The queue_output method ensures no duplicates if process_rules_system
                // already queued the same output.
                //
                // 使用去重队列输出事件。
                // 此系统可以正确评估包含 local facts 的条件，
                // 因此它应该处理引用 local facts 的规则的 outputs。
                // queue_output 方法确保如果 process_rules_system 已排队相同输出则不会重复。
                for output_id in &rule.outputs {
                    pending_events.queue_output(&rule.id, FactEvent::new(output_id.clone()));
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

/// Evaluate condition expressions against only global facts (no local facts).
/// Returns true if all conditions pass or if no conditions exist.
///
/// 只根据全局 FactDatabase 评估条件表达式。
/// 如果所有条件都通过或没有条件，则返回 true。
pub fn evaluate_conditions_layered(
    conditions: &[String],
    global_facts: &bevy_fact_rule_event::LayeredFactDatabase,
) -> bool {
    let empty_local = bevy_fact_rule_event::FactDatabase::new();
    for condition in conditions {
        let result = evaluate_single_condition(condition, &empty_local, global_facts);
        if !result {
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
    // Check for .len() suffix (e.g., $player:inventory.len())
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

    // Try parsing as string literal (single or double quotes)
    // 尝试解析字符串字面量（单引号或双引号）
    if (expr.starts_with('\'') && expr.ends_with('\''))
        || (expr.starts_with('"') && expr.ends_with('"'))
    {
        let inner = &expr[1..expr.len() - 1];
        return Some(FactValue::String(inner.to_string()));
    }

    None
}

fn resolve_int(
    expr: &str,
    local_facts: &bevy_fact_rule_event::FactDatabase,
    global_facts: &bevy_fact_rule_event::LayeredFactDatabase,
) -> Option<i64> {
    let expr = expr.trim();

    // Check for dynamic key pattern: $${prefix}.suffix
    // 检查动态键名模式：$${prefix}.suffix
    if expr.starts_with("$${") && expr.contains('}') {
        if let Some(val) = try_evaluate_dynamic_key(expr, local_facts) {
            return match val {
                FactValue::Int(v) => Some(v),
                FactValue::StringList(list) => Some(list.len() as i64),
                FactValue::IntList(list) => Some(list.len() as i64),
                _ => None,
            };
        }
        // Try global_facts if not found in local_facts
        if let Some(val) = try_evaluate_dynamic_key_global(expr, global_facts) {
            return match val {
                FactValue::Int(v) => Some(v),
                FactValue::StringList(list) => Some(list.len() as i64),
                FactValue::IntList(list) => Some(list.len() as i64),
                _ => None,
            };
        }
        return None;
    }

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

    // Check for .len() suffix (e.g., $player:inventory.len())
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
    global_facts: &bevy_fact_rule_event::LayeredFactDatabase,
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
            let fact_value = evaluate_local_fact_value(value, local_facts, global_facts);
            info!("FRE Bridge: SetLocalFact({}, {:?})", key, fact_value);
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
    }
}

/// Execute a single FRE action on the ViewRoot's local_facts (Firewheel variant).
///
/// 在 ViewRoot 的 local_facts 上执行单个 FRE 动作（Firewheel 变体）。
#[cfg(feature = "firewheel")]
fn execute_action_firewheel(
    action: &RuleActionDef,
    local_facts: &mut bevy_fact_rule_event::FactDatabase,
    global_facts: &bevy_fact_rule_event::LayeredFactDatabase,
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
            let fact_value = evaluate_local_fact_value(value, local_facts, global_facts);
            info!("FRE Bridge: SetLocalFact({}, {:?})", key, fact_value);
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
    }
}

/// Evaluate a LocalFactValue to a FactValue.
///
/// 将 LocalFactValue 评估为 FactValue。
fn evaluate_local_fact_value(
    value: &LocalFactValue,
    local_facts: &bevy_fact_rule_event::FactDatabase,
    global_facts: &bevy_fact_rule_event::LayeredFactDatabase,
) -> FactValue {
    match value {
        LocalFactValue::Int(v) => FactValue::Int(*v),
        LocalFactValue::Float(v) => FactValue::Float(*v),
        LocalFactValue::Bool(v) => FactValue::Bool(*v),
        LocalFactValue::String(v) => FactValue::String(v.clone()),
        LocalFactValue::Expr(expr) => {
            if let Some(result) =
                evaluate_simple_expression_with_global(expr, local_facts, global_facts)
            {
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

/// Simple expression evaluator that checks both local and global facts.
///
/// 检查本地和全局 facts 的简单表达式评估器。
fn evaluate_simple_expression_with_global(
    expr: &str,
    local_facts: &bevy_fact_rule_event::FactDatabase,
    global_facts: &bevy_fact_rule_event::LayeredFactDatabase,
) -> Option<FactValue> {
    let expr = expr.trim();

    // Handle dynamic key name: $${prefix_var}.suffix
    // First check local_facts for the prefix, then global_facts
    if expr.starts_with("$${") && expr.contains('}') {
        // Try local_facts first
        if let Some(result) = try_evaluate_dynamic_key(expr, local_facts) {
            return Some(result);
        }
        // Try with prefix from local_facts but looking up data from global_facts
        if let Some(result) = try_evaluate_dynamic_key_mixed(expr, local_facts, global_facts) {
            return Some(result);
        }
    }

    // For other expressions, try local_facts first
    if let Some(result) = evaluate_simple_expression(expr, local_facts) {
        return Some(result);
    }

    // Try global_facts for simple variable reference
    if expr.starts_with('$') && !expr.contains(' ') && !expr.contains('[') && !expr.contains('{') {
        let var_name = &expr[1..];
        if let Some(val) = global_facts.get_by_str(var_name) {
            return Some(val.clone());
        }
    }

    None
}

/// Try to evaluate a dynamic key expression with prefix from local_facts and data from global_facts.
///
/// 尝试从 local_facts 获取前缀，从 global_facts 获取数据的动态键名表达式。
fn try_evaluate_dynamic_key_mixed(
    expr: &str,
    local_facts: &bevy_fact_rule_event::FactDatabase,
    global_facts: &bevy_fact_rule_event::LayeredFactDatabase,
) -> Option<FactValue> {
    // Pattern: $${prefix_var}.suffix or $${prefix_var}.suffix[$index]
    if !expr.starts_with("$${") || !expr.contains('}') {
        return None;
    }

    // Extract prefix variable name (between ${ and })
    let brace_end = expr.find('}')?;
    let prefix_var = &expr[3..brace_end];

    // Get the prefix value from local_facts (must be a String)
    let prefix_value = match local_facts.get_by_str(prefix_var)? {
        FactValue::String(s) => s.clone(),
        _ => {
            return None;
        }
    };

    // Extract suffix (after })
    let suffix = &expr[brace_end + 1..];

    // Check if suffix contains array indexing
    if let Some(bracket_start) = suffix.find('[')
        && suffix.ends_with(']')
    {
        let array_suffix = &suffix[..bracket_start];
        let index_expr = &suffix[bracket_start + 1..suffix.len() - 1];

        let full_key = format!("{}{}", prefix_value, array_suffix);

        // Evaluate the index from local_facts
        let index: usize = if let Some(index_var_name) = index_expr.strip_prefix('$') {
            let index_value = local_facts.get_int(index_var_name)?;
            if index_value < 0 {
                return None;
            }
            index_value as usize
        } else {
            index_expr.parse::<usize>().ok()?
        };

        // Get the array from global_facts
        return match global_facts.get_by_str(&full_key)? {
            FactValue::StringList(list) => {
                if index < list.len() {
                    Some(FactValue::String(list[index].clone()))
                } else {
                    None
                }
            }
            FactValue::IntList(list) => {
                if index < list.len() {
                    Some(FactValue::Int(list[index]))
                } else {
                    None
                }
            }
            _ => None,
        };
    }

    // No array indexing, look up from global_facts
    let full_key = format!("{}{}", prefix_value, suffix);
    global_facts.get_by_str(&full_key).cloned()
}

/// Simple expression evaluator for basic arithmetic.
///
/// Supports:
/// - `$name` - local fact reference
/// - `$name + 1`, `$name - 1` - simple increment/decrement
/// - `$array[$index]` - array indexing (StringList/IntList)
/// - `$${prefix}.suffix` - dynamic key name (prefix resolved from fact)
///
/// 简单的表达式评估器，用于基本算术。
///
/// 支持：
/// - `$name` - 局部 fact 引用
/// - `$name + 1`、`$name - 1` - 简单增减
/// - `$array[$index]` - 数组索引（StringList/IntList）
/// - `$${prefix}.suffix` - 动态键名（prefix 从 fact 解析）
fn evaluate_simple_expression(
    expr: &str,
    facts: &bevy_fact_rule_event::FactDatabase,
) -> Option<FactValue> {
    let expr = expr.trim();

    // Handle dynamic key name: $${prefix_var}.suffix
    // Example: $${current_enemy_id}.action_labels → $froggit.action_labels
    if let Some(result) = try_evaluate_dynamic_key(expr, facts) {
        return Some(result);
    }

    // Handle array indexing: $array[$index]
    if let Some(result) = try_evaluate_array_index(expr, facts) {
        return Some(result);
    }

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

/// Try to evaluate a dynamic key expression: `$${prefix_var}.suffix`
///
/// This allows looking up facts with keys constructed dynamically.
/// Example: `$${current_enemy_id}.action_labels`
/// - First resolves `current_enemy_id` → "froggit"
/// - Then looks up `froggit.action_labels`
///
/// 尝试评估动态键名表达式：`$${prefix_var}.suffix`
///
/// 这允许使用动态构建的键查找 facts。
/// 示例：`$${current_enemy_id}.action_labels`
/// - 首先解析 `current_enemy_id` → "froggit"
/// - 然后查找 `froggit.action_labels`
///
/// Also supports array indexing after dynamic key:
/// - `$${current_enemy_id}.action_params[$act_selection]`
/// - First resolves to key like `froggit.action_params`
/// - Then indexes into that array
fn try_evaluate_dynamic_key(
    expr: &str,
    facts: &bevy_fact_rule_event::FactDatabase,
) -> Option<FactValue> {
    // Pattern: $${prefix_var}.suffix or $${prefix_var} or $${prefix_var}.suffix[$index]
    // Must start with $${ and contain }
    if !expr.starts_with("$${") || !expr.contains('}') {
        return None;
    }

    // Extract prefix variable name (between ${ and })
    let brace_end = expr.find('}')?;
    let prefix_var = &expr[3..brace_end];

    // Get the prefix value (must be a String)
    let prefix_value = match facts.get_by_str(prefix_var)? {
        FactValue::String(s) => s.clone(),
        _ => {
            warn!(
                "Dynamic key prefix '{}' is not a String: {:?}",
                prefix_var,
                facts.get_by_str(prefix_var)
            );
            return None;
        }
    };

    // Extract suffix (after })
    let suffix = &expr[brace_end + 1..];

    // Check if suffix contains array indexing: .array_name[$index]
    if let Some(bracket_start) = suffix.find('[')
        && suffix.ends_with(']')
    {
        // Extract array suffix and index expression
        let array_suffix = &suffix[..bracket_start]; // e.g., ".action_params"
        let index_expr = &suffix[bracket_start + 1..suffix.len() - 1]; // e.g., "$act_selection"

        // Build the full array key
        let full_key = format!("{}{}", prefix_value, array_suffix);

        // Evaluate the index
        let index: usize = if let Some(index_var_name) = index_expr.strip_prefix('$') {
            let index_value = facts.get_int(index_var_name)?;
            if index_value < 0 {
                warn!(
                    "Dynamic key array index '{}' evaluated to negative value: {}",
                    expr, index_value
                );
                return None;
            }
            index_value as usize
        } else {
            index_expr.parse::<usize>().ok()?
        };

        // Get the array and index into it
        return match facts.get_by_str(&full_key)? {
            FactValue::StringList(list) => {
                if index < list.len() {
                    Some(FactValue::String(list[index].clone()))
                } else {
                    warn!(
                        "Dynamic key array index {} out of bounds for '{}' (len={})",
                        index,
                        full_key,
                        list.len()
                    );
                    None
                }
            }
            FactValue::IntList(list) => {
                if index < list.len() {
                    Some(FactValue::Int(list[index]))
                } else {
                    None
                }
            }
            _ => None,
        };
    }

    // No array indexing, just look up the full key
    let full_key = format!("{}{}", prefix_value, suffix);
    facts.get_by_str(&full_key).cloned()
}

/// Try to evaluate a dynamic key expression using global facts: `$${prefix_var}.suffix`
///
/// Similar to `try_evaluate_dynamic_key` but for LayeredFactDatabase.
///
/// 类似于 `try_evaluate_dynamic_key`，但用于 LayeredFactDatabase。
fn try_evaluate_dynamic_key_global(
    expr: &str,
    facts: &bevy_fact_rule_event::LayeredFactDatabase,
) -> Option<FactValue> {
    // Pattern: $${prefix_var}.suffix or $${prefix_var}
    // Must start with $${ and contain }
    if !expr.starts_with("$${") || !expr.contains('}') {
        return None;
    }

    // Extract prefix variable name (between ${ and })
    let brace_end = expr.find('}')?;
    let prefix_var = &expr[3..brace_end];

    // Get the prefix value (must be a String)
    let prefix_value = match facts.get_by_str(prefix_var)? {
        FactValue::String(s) => s.clone(),
        _ => {
            warn!(
                "Dynamic key prefix '{}' is not a String in global facts: {:?}",
                prefix_var,
                facts.get_by_str(prefix_var)
            );
            return None;
        }
    };

    // Extract suffix (after })
    let suffix = &expr[brace_end + 1..];

    // Build the full key: prefix_value + suffix
    let full_key = format!("{}{}", prefix_value, suffix);

    // Look up the fact with the constructed key
    facts.get_by_str(&full_key).cloned()
}

/// Try to evaluate an array indexing expression: `$array[$index]`
///
/// Supports:
/// - StringList indexing → returns String
/// - IntList indexing → returns Int
/// - Index can be a literal number or a $variable reference
///
/// 尝试评估数组索引表达式：`$array[$index]`
fn try_evaluate_array_index(
    expr: &str,
    facts: &bevy_fact_rule_event::FactDatabase,
) -> Option<FactValue> {
    // Pattern: $array[$index]
    // Find the pattern: starts with $, has [, ends with ]
    if !expr.starts_with('$') || !expr.contains('[') || !expr.ends_with(']') {
        return None;
    }

    // Extract array name and index expression
    let bracket_start = expr.find('[')?;
    let array_name = &expr[1..bracket_start]; // Remove leading $
    let index_expr = &expr[bracket_start + 1..expr.len() - 1]; // Content between [ and ]

    // Evaluate the index expression
    let index: usize = if let Some(index_var_name) = index_expr.strip_prefix('$') {
        // Index is a variable reference
        let index_value = facts.get_int(index_var_name)?;
        if index_value < 0 {
            warn!(
                "Array index expression '{}' evaluated to negative value: {}",
                expr, index_value
            );
            return None;
        }
        index_value as usize
    } else {
        // Index is a literal number
        index_expr.parse::<usize>().ok()?
    };

    // Get the array value and index into it
    match facts.get_by_str(array_name)? {
        FactValue::StringList(list) => {
            if index < list.len() {
                Some(FactValue::String(list[index].clone()))
            } else {
                warn!(
                    "Array index {} out of bounds for StringList '{}' (len={})",
                    index,
                    array_name,
                    list.len()
                );
                None
            }
        }
        FactValue::IntList(list) => {
            if index < list.len() {
                Some(FactValue::Int(list[index]))
            } else {
                warn!(
                    "Array index {} out of bounds for IntList '{}' (len={})",
                    index,
                    array_name,
                    list.len()
                );
                None
            }
        }
        FactValue::FloatList(list) => {
            if index < list.len() {
                Some(FactValue::Float(list[index]))
            } else {
                warn!(
                    "Array index {} out of bounds for FloatList '{}' (len={})",
                    index,
                    array_name,
                    list.len()
                );
                None
            }
        }
        FactValue::BoolList(list) => {
            if index < list.len() {
                Some(FactValue::Bool(list[index]))
            } else {
                warn!(
                    "Array index {} out of bounds for BoolList '{}' (len={})",
                    index,
                    array_name,
                    list.len()
                );
                None
            }
        }
        _ => {
            warn!(
                "Cannot index into non-array fact '{}' (type: {:?})",
                array_name,
                facts.get_by_str(array_name)
            );
            None
        }
    }
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
        app.add_systems(Startup, register_condition_evaluator_system)
            .add_systems(
                Update,
                (
                    sync_state_to_facts_system.run_if(state_facts_need_sync),
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
            "player:inventory",
            FactValue::StringList(vec![
                "item1".to_string(),
                "item2".to_string(),
                "item3".to_string(),
            ]),
        );

        // Test $player:inventory.len() resolves to 3
        let conditions = vec!["$player:inventory.len() == 3".to_string()];
        assert!(evaluate_conditions(
            &conditions,
            &local_facts,
            &global_facts
        ));

        // Test comparison with arithmetic
        let conditions2 = vec!["$player:inventory.len() > 2".to_string()];
        assert!(evaluate_conditions(
            &conditions2,
            &local_facts,
            &global_facts
        ));
    }
}

// ============================================================================
// Condition Evaluator Implementation
// ============================================================================

/// Souprune's implementation of condition evaluation.
/// This evaluator is registered with the FRE system to evaluate rule conditions.
///
/// Souprune 的条件评估实现。
/// 此评估器注册到 FRE 系统以评估规则条件。
pub struct SoupruneConditionEvaluator;

impl bevy_fact_rule_event::ConditionEvaluatorTrait for SoupruneConditionEvaluator {
    fn evaluate(
        &self,
        conditions: &[String],
        facts: &bevy_fact_rule_event::LayeredFactDatabase,
    ) -> bool {
        evaluate_conditions_layered(conditions, facts)
    }
}

/// System to register the Souprune condition evaluator with the FRE system.
/// Should run at startup, after FREPlugin is built.
///
/// 将 Souprune 条件评估器注册到 FRE 系统的系统。
/// 应在启动时运行，在 FREPlugin 构建之后。
pub fn register_condition_evaluator_system(mut commands: Commands) {
    commands.insert_resource(bevy_fact_rule_event::ConditionEvaluator::new(
        SoupruneConditionEvaluator,
    ));
}
