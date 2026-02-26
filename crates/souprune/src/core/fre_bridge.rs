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
pub use eval::{evaluate_conditions_layered, evaluate_single_condition};
use eval::{evaluate_conditions, evaluate_local_fact_value, register_condition_evaluator_system};

use bevy::prelude::*;
use bevy_fact_rule_event::{
    FactEvent, FactValue, LayeredRuleRegistry, RuleActionDef,
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
    if let Some(ref state) = overworld_state
        && state.is_changed()
    {
        return true;
    }
    if let Some(ref state) = app_state
        && state.is_changed()
    {
        return true;
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


