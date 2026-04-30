//! Dispatches matched custom FRE actions to registered game-side handlers.
//!
//! 分发已经命中的自定义 FRE 动作，把它们交给游戏侧注册的处理器。
//!
//! Covers the branch of the rule pipeline that is not generic enough
//! to be handled as built-in view actions. It looks up matching game rules,
//! re-checks their conditions against the current fact set, and either executes
//! a registered handler or emits a fallback `FreCustomActionEvent`.
//!
//! 负责规则流水线里那部分无法被通用 View 动作直接处理的分支。
//! 它会查找匹配的游戏规则，在当前事实集合上重新确认条件，然后要么调用已注册
//! 的处理器，要么退回成一个 `FreCustomActionEvent` 交给别的系统继续消费。

use super::{FreCustomActionEvent, evaluate_conditions};
use bevy::prelude::*;
use bevy_fact_rule_event::{EnumRegistry, FactEvent, FactValue, LayeredFactDatabase};

use crate::core::fre_facts;
use crate::core::game_action::{
    GameActionDef, GameActionHandlerRegistry, GameRule, GameRuleRegistry,
};

struct StartDialogueFacts<'a> {
    channel: &'a str,
    mortar: &'a str,
    node: &'a str,
    view: Option<&'a str>,
    typewriter: bool,
    focus: bool,
    voice: Option<&'a str>,
}

fn set_start_dialogue_facts(fact_db: &mut LayeredFactDatabase, facts: StartDialogueFacts<'_>) {
    fact_db.set_local(
        fre_facts::DIALOGUE_PENDING_CHANNEL,
        FactValue::String(facts.channel.to_string()),
    );
    fact_db.set_local(
        fre_facts::DIALOGUE_PENDING_MORTAR_PATH,
        FactValue::String(facts.mortar.to_string()),
    );
    fact_db.set_local(
        fre_facts::DIALOGUE_PENDING_MORTAR_NODE,
        FactValue::String(facts.node.to_string()),
    );
    if let Some(view_path) = facts.view {
        fact_db.set_local(
            fre_facts::DIALOGUE_PENDING_VIEW,
            FactValue::String(view_path.to_string()),
        );
    }
    fact_db.set_local(
        fre_facts::DIALOGUE_HAS_TYPEWRITER,
        FactValue::Bool(facts.typewriter),
    );
    fact_db.set_local(
        fre_facts::dialogue_channel_key(facts.channel, "has_typewriter"),
        FactValue::Bool(facts.typewriter),
    );
    fact_db.set_local(fre_facts::DIALOGUE_HAS_FOCUS, FactValue::Bool(facts.focus));
    fact_db.set_local(
        fre_facts::dialogue_channel_key(facts.channel, "has_focus"),
        FactValue::Bool(facts.focus),
    );
    if let Some(voice_path) = facts.voice {
        fact_db.set_local(
            fre_facts::DIALOGUE_VOICE,
            FactValue::String(voice_path.to_string()),
        );
        fact_db.set_local(
            fre_facts::dialogue_channel_key(facts.channel, "voice"),
            FactValue::String(voice_path.to_string()),
        );
    }
    fact_db.set_local(fre_facts::DIALOGUE_PENDING_START, FactValue::Bool(true));
}

fn dispatch_single_custom_action(
    action: &GameActionDef,
    rule: &GameRule,
    fact_db: &mut LayeredFactDatabase,
    handler_registry: &GameActionHandlerRegistry,
    commands: &mut Commands,
    custom_action_writer: &mut MessageWriter<FreCustomActionEvent>,
) {
    match action {
        GameActionDef::StartDialogue {
            channel,
            mortar,
            node,
            view,
            typewriter,
            focus,
            voice,
        } => {
            let channel = channel
                .as_deref()
                .map(fre_facts::normalize_dialogue_channel)
                .unwrap_or(fre_facts::DIALOGUE_DEFAULT_CHANNEL);
            info!(
                "FRE: StartDialogue(channel: {}, mortar: {}, node: {}) from rule '{}'",
                channel, mortar, node, rule.id
            );
            set_start_dialogue_facts(
                fact_db,
                StartDialogueFacts {
                    channel,
                    mortar,
                    node,
                    view: view.as_deref(),
                    typewriter: *typewriter,
                    focus: *focus,
                    voice: voice.as_deref(),
                },
            );
        }
        GameActionDef::Custom {
            action_type,
            params,
        } => {
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
        _ => {}
    }
}

fn dispatch_event_custom_actions(
    event: &FactEvent,
    rule_registry: &GameRuleRegistry,
    fact_db: &mut LayeredFactDatabase,
    handler_registry: &GameActionHandlerRegistry,
    commands: &mut Commands,
    custom_action_writer: &mut MessageWriter<FreCustomActionEvent>,
    enum_registry: &EnumRegistry,
    event_trace: &mut crate::core::trace::EventTraceLog,
    time: &Time,
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

            event_trace.record(
                crate::core::trace::EventPhase::Logic,
                "CustomRuleTriggered",
                format!(
                    "rule '{}' dispatching {} custom actions",
                    rule.id,
                    rule.actions.len()
                ),
                time.elapsed_secs_f64(),
                None,
            );

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

pub fn dispatch_custom_actions_system(
    mut events: MessageReader<FactEvent>,
    rule_registry: Res<GameRuleRegistry>,
    mut fact_db: ResMut<LayeredFactDatabase>,
    handler_registry: Res<GameActionHandlerRegistry>,
    mut commands: Commands,
    mut custom_action_writer: MessageWriter<FreCustomActionEvent>,
    enum_registry: Res<EnumRegistry>,
    mut event_trace: ResMut<crate::core::trace::EventTraceLog>,
    time: Res<Time>,
) {
    for event in events.read() {
        dispatch_event_custom_actions(
            event,
            &rule_registry,
            &mut fact_db,
            &handler_registry,
            &mut commands,
            &mut custom_action_writer,
            &enum_registry,
            &mut event_trace,
            &time,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn start_dialogue_refreshes_channel_scoped_facts() {
        let mut facts = LayeredFactDatabase::new();
        let channel = fre_facts::DIALOGUE_DEFAULT_CHANNEL;

        facts.set_local(
            fre_facts::dialogue_channel_key(channel, "has_focus"),
            FactValue::Bool(false),
        );
        facts.set_local(
            fre_facts::dialogue_channel_key(channel, "has_typewriter"),
            FactValue::Bool(false),
        );
        facts.set_local(
            fre_facts::dialogue_channel_key(channel, "voice"),
            FactValue::String("old.wav".to_string()),
        );

        set_start_dialogue_facts(
            &mut facts,
            StartDialogueFacts {
                channel,
                mortar: "overworld/dialogue.mortar",
                node: "demo_talk",
                view: Some("overworld/view/dialogue.view.ron"),
                typewriter: true,
                focus: true,
                voice: Some("new.wav"),
            },
        );

        assert_eq!(
            facts.get_bool(&fre_facts::dialogue_channel_key(channel, "has_focus")),
            Some(true)
        );
        assert_eq!(
            facts.get_bool(&fre_facts::dialogue_channel_key(channel, "has_typewriter")),
            Some(true)
        );
        assert_eq!(
            facts.get_string(&fre_facts::dialogue_channel_key(channel, "voice")),
            Some("new.wav")
        );
    }
}
