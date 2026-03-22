//! Applies FRE rule results to the currently active View and related UI-side state.
//!
//! 把 FRE 规则计算结果应用到当前激活的 View 以及相关的界面侧状态上。
//!
//! This file is the UI-facing executor in the FRE bridge. It listens for fact
//! events, finds rules that target the active view, evaluates them against
//! local and global facts, and then performs actions such as playing sounds,
//! mutating local view facts, switching state, starting dialogue, or queuing
//! follow-up outputs.
//!
//! 这个文件是 FRE bridge 面向 UI 的执行层。它监听事实事件，找出作用于当前
//! 活跃 View 的规则，在局部与全局事实之上评估它们，然后执行播放音效、
//! 修改 View 局部事实、切换状态、启动对话、继续排队输出事件等动作。

use super::{evaluate_conditions, evaluate_local_fact_value, item_actions};
use bevy::prelude::*;
use bevy_fact_rule_event::{
    CombinedFactReader, EnumRegistry, FactEvent, FactValue, LayeredFactDatabase, PendingFactEvents,
};

use crate::config::SoupruneConfig;
use crate::core::audio;
use crate::core::fre_facts;
use crate::core::game_action::{GameActionDef, GameRule, GameRuleRegistry};
use crate::core::item::ItemRegistry;
use crate::core::mode::SequenceSubState;
use crate::core::view::components::{ActiveView, ViewRoot};

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

fn process_event_view_actions(
    event: &FactEvent,
    rule_registry: &GameRuleRegistry,
    active_view_query: &mut Query<&mut ViewRoot, With<ActiveView>>,
    audio: &bevy_kira_audio::Audio,
    asset_server: &AssetServer,
    global_facts: &mut LayeredFactDatabase,
    pending_events: &mut PendingFactEvents,
    trigger_history: &mut Option<ResMut<crate::extra::debug::RuleTriggerHistory>>,
    time: &Time,
    enum_registry: &EnumRegistry,
    item_registry: &ItemRegistry,
    souprune_config: &SoupruneConfig,
) {
    let rule_groups = rule_registry.get_matching_rules_grouped(event);
    log_event_rule_matches(event, &rule_groups);

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

            if let Some(history) = trigger_history {
                history.record_trigger(&rule.id, time.elapsed_secs_f64());
            }

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
    audio: Res<bevy_kira_audio::Audio>,
    asset_server: Res<AssetServer>,
    mut global_facts: ResMut<LayeredFactDatabase>,
    mut pending_events: ResMut<PendingFactEvents>,
    mut trigger_history: Option<ResMut<crate::extra::debug::RuleTriggerHistory>>,
    time: Res<Time>,
    enum_registry: Res<EnumRegistry>,
    item_registry: Res<ItemRegistry>,
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

fn execute_action(
    action: &GameActionDef,
    local_facts: &mut bevy_fact_rule_event::FactDatabase,
    global_facts: &mut LayeredFactDatabase,
    audio: &bevy_kira_audio::Audio,
    asset_server: &AssetServer,
    enum_registry: &EnumRegistry,
    item_registry: &ItemRegistry,
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
            item_actions::execute_use_item(
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
            item_actions::execute_check_item(
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
            item_actions::execute_drop_item(
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

pub fn handle_switch_state_system(
    mut active_view_query: Query<&mut ViewRoot, With<ActiveView>>,
    mut next_state: ResMut<NextState<SequenceSubState>>,
) {
    for mut view_root in active_view_query.iter_mut() {
        if let Some(FactValue::String(state_name)) = view_root
            .local_facts
            .get_by_str(fre_facts::VIEW_SWITCH_STATE)
        {
            let state_name = state_name.clone();
            info!("FRE Bridge: Switching to state '{}'", state_name);
            next_state.set(SequenceSubState::new(&state_name));
            view_root.local_facts.remove(fre_facts::VIEW_SWITCH_STATE);
        }
    }
}
