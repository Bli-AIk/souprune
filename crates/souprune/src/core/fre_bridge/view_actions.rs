//! Applies FRE rule results to the currently active View and related UI-side state.
//!
//! 把 FRE 规则计算结果应用到当前激活的 View 以及相关的界面侧状态上。
//!
//! Acts as the UI-facing executor in the FRE bridge. It listens for fact
//! events, finds rules that target the active view, evaluates them against
//! local and global facts, and then performs actions such as playing sounds,
//! mutating local view facts, switching state, starting dialogue, or queuing
//! follow-up outputs.
//!
//! FRE bridge 面向 UI 的执行层。它监听事实事件，找出作用于当前
//! 活跃 View 的规则，在局部与全局事实之上评估它们，然后执行播放音效、
//! 修改 View 局部事实、切换状态、启动对话、继续排队输出事件等动作。

use super::extensions::{ViewActionExecCtx, ViewActionExtensions};
use super::{evaluate_conditions, evaluate_local_fact_value};
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use bevy_fact_rule_event::{
    CombinedFactReader, EnumRegistry, FactEvent, FactValue, LayeredFactDatabase, PendingFactEvents,
};

use crate::config::SoupruneConfig;
use crate::core::audio;
use crate::core::fre_bridge::FreCustomActionEvent;
use crate::core::fre_facts;
use crate::core::game_action::{GameActionDef, GameRule, GameRuleRegistry};
use crate::core::mode::SequenceSubState;
use crate::core::view::components::{ActiveView, ViewRoot};

#[derive(SystemParam)]
pub struct ViewActionSystemParams<'w> {
    audio: Res<'w, bevy_kira_audio::Audio>,
    asset_server: Res<'w, AssetServer>,
    audio_cache: ResMut<'w, audio::AudioSourceCache>,
    global_facts: ResMut<'w, LayeredFactDatabase>,
    pending_events: ResMut<'w, PendingFactEvents>,
    trigger_history: Option<ResMut<'w, crate::extra::debug::RuleTriggerHistory>>,
    time: Res<'w, Time>,
    enum_registry: Res<'w, EnumRegistry>,
    souprune_config: Res<'w, SoupruneConfig>,
    event_trace: ResMut<'w, crate::core::trace::EventTraceLog>,
    fact_history: ResMut<'w, crate::core::trace::FactChangeHistory>,
    frame_count: Res<'w, bevy::diagnostic::FrameCount>,
    extensions: Res<'w, ViewActionExtensions>,
    custom_action_writer: MessageWriter<'w, FreCustomActionEvent>,
}

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

fn log_condition_not_met(rule: &GameRule, view_root: &ViewRoot) {
    if rule.id.contains("act") || rule.id.contains("depth_2") {
        debug!(
            "FRE Bridge: Conditions not met for rule '{}', local_facts: depth={:?}, menu_context={:?}, act_selection={:?}, act_count={:?}",
            rule.id,
            view_root.local_state().get_int("depth"),
            view_root.local_state().get_int("menu_context"),
            view_root.local_state().get_int("act_selection"),
            view_root.local_state().get_int("act_count")
        );
    } else {
        debug!(
            "FRE Bridge: Conditions not met for rule '{}', local_facts: depth={:?}, selection={:?}",
            rule.id,
            view_root.local_state().get_int("depth"),
            view_root.local_state().get_int("selection")
        );
    }
}

fn process_event_view_actions(
    event: &FactEvent,
    rule_registry: &GameRuleRegistry,
    active_view_query: &mut Query<&mut ViewRoot, With<ActiveView>>,
    audio: &bevy_kira_audio::Audio,
    asset_server: &AssetServer,
    audio_cache: &mut audio::AudioSourceCache,
    global_facts: &mut LayeredFactDatabase,
    pending_events: &mut PendingFactEvents,
    trigger_history: &mut Option<ResMut<crate::extra::debug::RuleTriggerHistory>>,
    time: &Time,
    enum_registry: &EnumRegistry,
    souprune_config: &SoupruneConfig,
    event_trace: &mut crate::core::trace::EventTraceLog,
    fact_history: &mut crate::core::trace::FactChangeHistory,
    frame_number: u64,
    extensions: &ViewActionExtensions,
    custom_action_writer: &mut MessageWriter<FreCustomActionEvent>,
) {
    let rule_groups = rule_registry.get_matching_rules_grouped(event);
    log_event_rule_matches(event, &rule_groups);

    'outer: for group in rule_groups {
        for rule in group {
            let Ok(mut view_root) = active_view_query.single_mut() else {
                info!("FRE Bridge: No ActiveView found for rule '{}'", rule.id);
                continue;
            };

            let combined = CombinedFactReader::new(view_root.local_state(), global_facts);
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

            if let Some(history) = trigger_history {
                history.record_trigger(&rule.id, time.elapsed_secs_f64());
            }

            // Record the rule match in the event trace as a causal child of the FactEvent.
            let parent_idx = event_trace.current_frame_events.len().checked_sub(1);
            event_trace.record(
                crate::core::trace::EventPhase::Logic,
                "RuleTriggered",
                format!(
                    "rule '{}' → {} actions, {} outputs",
                    rule.id,
                    rule.actions.len(),
                    rule.outputs.len()
                ),
                time.elapsed_secs_f64(),
                parent_idx,
            );

            for action in &rule.actions {
                execute_action(
                    action,
                    &mut view_root,
                    global_facts,
                    audio,
                    asset_server,
                    audio_cache,
                    enum_registry,
                    souprune_config,
                    fact_history,
                    frame_number,
                    &rule.id,
                    extensions,
                    custom_action_writer,
                );
            }

            for output_id in &rule.outputs {
                pending_events.queue_output(&rule.id, FactEvent::new(output_id.clone()));
            }

            if rule.consume_event {
                break 'outer;
            }
        }
    }
}

pub fn process_view_actions_system(
    mut events: MessageReader<FactEvent>,
    rule_registry: Res<GameRuleRegistry>,
    mut active_view_query: Query<&mut ViewRoot, With<ActiveView>>,
    mut params: ViewActionSystemParams,
) {
    let events_to_process: Vec<FactEvent> = events.read().cloned().collect();

    for event in &events_to_process {
        let ts = params.time.elapsed_secs_f64();
        params.event_trace.record(
            crate::core::trace::EventPhase::Logic,
            "FactEvent",
            format!("event '{}'", event.id.0),
            ts,
            None,
        );

        process_event_view_actions(
            event,
            &rule_registry,
            &mut active_view_query,
            &params.audio,
            &params.asset_server,
            &mut params.audio_cache,
            &mut params.global_facts,
            &mut params.pending_events,
            &mut params.trigger_history,
            &params.time,
            &params.enum_registry,
            &params.souprune_config,
            &mut params.event_trace,
            &mut params.fact_history,
            params.frame_count.0 as u64,
            &params.extensions,
            &mut params.custom_action_writer,
        );
    }
}

fn execute_action(
    action: &GameActionDef,
    view_root: &mut ViewRoot,
    global_facts: &mut LayeredFactDatabase,
    audio: &bevy_kira_audio::Audio,
    asset_server: &AssetServer,
    audio_cache: &mut audio::AudioSourceCache,
    enum_registry: &EnumRegistry,
    souprune_config: &SoupruneConfig,
    fact_history: &mut crate::core::trace::FactChangeHistory,
    frame_number: u64,
    rule_id: &str,
    extensions: &ViewActionExtensions,
    custom_action_writer: &mut MessageWriter<FreCustomActionEvent>,
) {
    if execute_view_state_action(
        action,
        view_root,
        global_facts,
        enum_registry,
        fact_history,
        frame_number,
        rule_id,
    )
    .is_some()
    {
        return;
    }

    match action {
        GameActionDef::PlaySound(sound_name) => {
            debug!("FRE Bridge: PlaySound({})", sound_name);
            audio::play_sound(audio, asset_server, audio_cache, sound_name);
        }
        GameActionDef::PlaySoundFullPath(path) => {
            debug!("FRE Bridge: PlaySoundFullPath({})", path);
            audio::play_sound_full_path(audio, asset_server, audio_cache, path);
        }
        GameActionDef::SetLocalFact(_, _)
        | GameActionDef::CloseView
        | GameActionDef::SwitchState(_) => {}
        GameActionDef::EmitEvent(event_id) => {
            debug!("FRE Bridge: EmitEvent({})", event_id);
        }
        GameActionDef::StartDialogue { .. } => {
            // Handled by dispatch_custom_actions_system — skip here.
            // 由 dispatch_custom_actions_system 处理 — 此处跳过。
        }
        GameActionDef::Custom {
            action_type,
            params,
        } => {
            let local_facts_snapshot = view_root
                .local_state()
                .iter()
                .map(|(key, value)| (key.clone(), value.clone()))
                .collect();
            let local_facts = view_root
                .local_state_mut_for_owner()
                .as_facts_mut_for_owner();
            let mut ctx = ViewActionExecCtx {
                local_facts,
                global_facts,
                audio,
                asset_server,
                audio_cache,
                enum_registry,
                config: souprune_config,
                fact_history,
                frame_number,
                rule_id,
            };
            if !extensions.handle(action_type, params, &mut ctx) {
                debug!(
                    "FRE Bridge: Unhandled custom action '{}' with params {:?}",
                    action_type, params
                );
                custom_action_writer.write(FreCustomActionEvent {
                    action_type: action_type.clone(),
                    params: params.clone(),
                    local_facts: local_facts_snapshot,
                    local_facts_target: true,
                });
            }
        }
        GameActionDef::Log { message } => {
            info!("FRE Bridge: Log: {}", message);
        }
    }
}

fn execute_view_state_action(
    action: &GameActionDef,
    view_root: &mut ViewRoot,
    global_facts: &LayeredFactDatabase,
    enum_registry: &EnumRegistry,
    fact_history: &mut crate::core::trace::FactChangeHistory,
    frame_number: u64,
    rule_id: &str,
) -> Option<()> {
    match action {
        GameActionDef::SetLocalFact(key, value) => {
            let combined = CombinedFactReader::new(view_root.local_state(), global_facts);
            let fact_value = evaluate_local_fact_value(key, value, &combined, enum_registry);
            let old = view_root.local_state().get_by_str(key).cloned();
            fact_history.record(
                format!("local:{key}"),
                old,
                fact_value.clone(),
                rule_id,
                frame_number,
            );
            info!("FRE Bridge: SetLocalFact({}, {:?})", key, fact_value);
            view_root.set_local_value(key.as_str(), fact_value);
            Some(())
        }
        GameActionDef::CloseView => {
            debug!("FRE Bridge: CloseView");
            view_root.request_close();
            Some(())
        }
        GameActionDef::SwitchState(state_name) => {
            debug!("FRE Bridge: SwitchState({})", state_name);
            view_root.switch_state(state_name.clone());
            Some(())
        }
        _ => None,
    }
}

pub fn handle_switch_state_system(
    mut active_view_query: Query<&mut ViewRoot, With<ActiveView>>,
    mut next_state: ResMut<NextState<SequenceSubState>>,
) {
    for mut view_root in active_view_query.iter_mut() {
        if let Some(FactValue::String(state_name)) = view_root
            .local_state()
            .get_by_str(fre_facts::VIEW_SWITCH_STATE)
        {
            let state_name = state_name.clone();
            info!("FRE Bridge: Switching to state '{}'", state_name);
            next_state.set(SequenceSubState::new(&state_name));
            view_root.remove_local_value(fre_facts::VIEW_SWITCH_STATE);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_fact_rule_event::{EnumRegistry, LayeredFactDatabase, LocalFactValue};

    #[test]
    fn set_local_fact_updates_active_view_through_controlled_write() {
        let mut view_root = ViewRoot::new("tests/menu.view.ron".to_string());
        view_root.set_local_value("selection", FactValue::Int(0));
        let global_facts = LayeredFactDatabase::new();
        let enum_registry = EnumRegistry::default();
        let mut fact_history = crate::core::trace::FactChangeHistory::default();

        execute_view_state_action(
            &GameActionDef::SetLocalFact(
                "selection".to_string(),
                LocalFactValue::Expr("$selection + 1".to_string()),
            ),
            &mut view_root,
            &global_facts,
            &enum_registry,
            &mut fact_history,
            1,
            "test_rule",
        )
        .expect("SetLocalFact should be a view-state action");

        assert_eq!(view_root.local_state().get_int("selection"), Some(1));
    }
}
