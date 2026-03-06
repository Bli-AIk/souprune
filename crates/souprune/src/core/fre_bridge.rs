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
use eval::{evaluate_conditions, evaluate_local_fact_value, register_condition_evaluator_system};
pub use eval::{evaluate_conditions_layered, evaluate_single_condition};

use bevy::prelude::*;
use bevy_fact_rule_event::{
    ActionHandlerRegistry, FactEvent, FactValue, LayeredFactDatabase, LayeredRuleRegistry,
    RuleActionDef,
};
use leafwing_input_manager::action_state::ActionState;
use std::collections::HashMap;

use crate::app_state::overworld::trigger::RuleActionDefs;
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
fn log_event_rule_matches(event: &FactEvent, rule_groups: &[Vec<&bevy_fact_rule_event::Rule>]) {
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
fn log_condition_not_met(rule: &bevy_fact_rule_event::Rule, view_root: &ViewRoot) {
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
    rule_registry: &LayeredRuleRegistry,
    action_defs: &RuleActionDefs,
    active_view_query: &mut Query<&mut ViewRoot, With<ActiveView>>,
    audio: &bevy_kira_audio::Audio,
    asset_server: &AssetServer,
    global_facts: &bevy_fact_rule_event::LayeredFactDatabase,
    pending_events: &mut bevy_fact_rule_event::PendingFactEvents,
    trigger_history: &mut Option<ResMut<crate::extra::debug::RuleTriggerHistory>>,
    time: &Time,
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

            if !evaluate_conditions(
                &rule.condition_expressions,
                &view_root.local_facts,
                global_facts,
            ) {
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

            // Execute each action from the rule's action definitions
            for action in action_defs
                .actions_by_rule
                .get(&rule.id)
                .into_iter()
                .flatten()
            {
                execute_action(
                    action,
                    &mut view_root.local_facts,
                    global_facts,
                    audio,
                    asset_server,
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
    rule_registry: Res<LayeredRuleRegistry>,
    action_defs: Option<Res<RuleActionDefs>>,
    mut active_view_query: Query<&mut ViewRoot, With<ActiveView>>,
    audio: Res<bevy_kira_audio::Audio>,
    asset_server: Res<AssetServer>,
    global_facts: Res<bevy_fact_rule_event::LayeredFactDatabase>,
    mut pending_events: ResMut<bevy_fact_rule_event::PendingFactEvents>,
    mut trigger_history: Option<ResMut<crate::extra::debug::RuleTriggerHistory>>,
    time: Res<Time>,
) {
    let Some(action_defs) = action_defs else {
        return;
    };

    let events_to_process: Vec<FactEvent> = events.read().cloned().collect();

    for event in &events_to_process {
        process_event_view_actions(
            event,
            &rule_registry,
            &action_defs,
            &mut active_view_query,
            &audio,
            &asset_server,
            &global_facts,
            &mut pending_events,
            &mut trigger_history,
            &time,
        );
    }
}

/// Execute a single FRE action on the ViewRoot's local_facts.
///
/// 在 ViewRoot 的 local_facts 上执行单个 FRE 动作。
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
            local_facts.set(fre_facts::VIEW_CLOSE_REQUESTED, FactValue::Bool(true));
        }
        RuleActionDef::SwitchState(state_name) => {
            debug!("FRE Bridge: SwitchState({})", state_name);
            local_facts.set(
                fre_facts::VIEW_SWITCH_STATE,
                FactValue::String(state_name.clone()),
            );
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
    action: &RuleActionDef,
    rule: &bevy_fact_rule_event::Rule,
    fact_db: &LayeredFactDatabase,
    handler_registry: &ActionHandlerRegistry,
    commands: &mut Commands,
    custom_action_writer: &mut MessageWriter<FreCustomActionEvent>,
) {
    let RuleActionDef::Custom {
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
    rule_registry: &LayeredRuleRegistry,
    action_defs: &RuleActionDefs,
    fact_db: &LayeredFactDatabase,
    handler_registry: &ActionHandlerRegistry,
    commands: &mut Commands,
    custom_action_writer: &mut MessageWriter<FreCustomActionEvent>,
) {
    let rule_groups = rule_registry.get_matching_rules_grouped(event);
    if rule_groups.is_empty() {
        return;
    }

    'outer: for group in rule_groups {
        for rule in group {
            if !evaluate_conditions_layered(&rule.condition_expressions, fact_db) {
                continue;
            }

            let Some(actions) = action_defs.actions_by_rule.get(&rule.id) else {
                continue;
            };

            for action in actions {
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
    rule_registry: Res<LayeredRuleRegistry>,
    action_defs: Res<RuleActionDefs>,
    fact_db: Res<LayeredFactDatabase>,
    handler_registry: Res<ActionHandlerRegistry>,
    mut commands: Commands,
    mut custom_action_writer: MessageWriter<FreCustomActionEvent>,
) {
    for event in events.read() {
        dispatch_event_custom_actions(
            event,
            &rule_registry,
            &action_defs,
            &fact_db,
            &handler_registry,
            &mut commands,
            &mut custom_action_writer,
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
