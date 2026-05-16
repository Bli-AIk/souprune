//! # bridge.rs
//!
//! Bridge between Sequencer and FRE systems.
//!
//! Sequencer 和 FRE 系统之间的桥接。
//!
//! This module provides systems that emit FRE events when specific
//! sequencer events occur (Chapter completion, selection confirmation, etc.).
//!
//! 本模块提供系统，在特定 Sequencer 事件发生时发出 FRE 事件
//! （Chapter 完成、选择确认等）。
//!
//! NOTE: Menu navigation is expected to be owned by project FRE rules.
//! The old hardcoded navigation system has been removed.
//!
//! 注意：菜单导航应由项目 FRE 规则拥有。
//! 旧的硬编码导航系统已被移除。

use bevy::prelude::*;
use bevy_fact_rule_event::{FactEvent, LayeredFactDatabase};

/// Event emitted when a Chapter completes.
/// This is an internal Bevy event used to bridge Sequencer → FRE.
///
/// Chapter 完成时发出的事件。
/// 这是用于桥接 Sequencer → FRE 的内部 Bevy 事件。
#[derive(Message, Debug, Clone)]
pub struct ChapterCompletedEvent {
    /// Type of chapter that completed (e.g., "DanmakuPerformance", "AwaitInteraction")
    pub chapter_type: String,
    /// Optional result data from the chapter
    pub result: Option<String>,
}

impl ChapterCompletedEvent {
    /// Create a new chapter completed event.
    pub fn new(chapter_type: impl Into<String>) -> Self {
        Self {
            chapter_type: chapter_type.into(),
            result: None,
        }
    }

    /// Create with result data.
    pub fn with_result(chapter_type: impl Into<String>, result: impl Into<String>) -> Self {
        Self {
            chapter_type: chapter_type.into(),
            result: Some(result.into()),
        }
    }
}

/// Run condition: Check if there are chapter completed events to process.
/// 运行条件：检查是否有章节完成事件需要处理。
pub fn has_chapter_completed_events(events: MessageReader<ChapterCompletedEvent>) -> bool {
    !events.is_empty()
}

/// System to emit FRE events when chapters complete.
/// Converts internal ChapterCompletedEvent to FactEvent for FRE processing.
///
/// Chapter 完成时发出 FRE 事件的系统。
/// 将内部 ChapterCompletedEvent 转换为 FactEvent 供 FRE 处理。
pub fn emit_chapter_completed_events_system(
    mut chapter_events: MessageReader<ChapterCompletedEvent>,
    mut fact_event_writer: MessageWriter<FactEvent>,
    mut layered_db: ResMut<LayeredFactDatabase>,
) {
    for event in chapter_events.read() {
        // Emit generic chapter completed event
        let mut fact_event =
            FactEvent::new("chapter_completed").with_data("chapter_type", &event.chapter_type);

        if let Some(result) = &event.result {
            fact_event = fact_event.with_data("result", result);
        }

        fact_event_writer.write(fact_event);

        // Emit specific events based on chapter type
        match event.chapter_type.as_str() {
            "DanmakuPerformance" => {
                fact_event_writer.write(FactEvent::new("danmaku_finished"));
            }
            "AlightMotionPerformance" => {
                fact_event_writer.write(FactEvent::new("alight_motion_performance_finished"));
            }
            "Wait" => {
                fact_event_writer.write(FactEvent::new("wait_finished"));
            }
            _ => {}
        }

        // Update phase fact if needed
        if event.chapter_type == "DanmakuPerformance"
            || event.chapter_type == "AlightMotionPerformance"
        {
            layered_db.set("phase", "player_turn");
            fact_event_writer.write(FactEvent::new("phase_changed"));
        }

        info!(
            "FixedScene FRE Bridge: Emitted events for chapter completion: {}",
            event.chapter_type
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::fre_bridge::evaluate_local_fact_value;
    use crate::core::game_action::{GameActionDef, GameFreAsset};
    use crate::core::view::ViewRoot;
    use bevy_fact_rule_event::{
        CombinedFactReader, EnumRegistry, FactValue, RuleEventDef, RuleScopeDef,
    };

    #[test]
    fn menu_state_can_be_projected_from_rule_dynamic_actor_data() {
        let fre = GameFreAsset {
            scope: RuleScopeDef::Local,
            enums: Default::default(),
            facts: Default::default(),
            rules: vec![bevy_fact_rule_event::RuleDef {
                id: "open_actor_action_menu".to_string(),
                event: RuleEventDef::Event("input:confirm".to_string()),
                conditions: Vec::new(),
                actions: vec![
                    GameActionDef::SetLocalFact(
                        "current_enemy_id".to_string(),
                        bevy_fact_rule_event::LocalFactValue::Expr(
                            "$enemy_ids[$enemy_selection]".to_string(),
                        ),
                    ),
                    GameActionDef::SetLocalFact(
                        "act_count".to_string(),
                        bevy_fact_rule_event::LocalFactValue::Expr(
                            "$${current_enemy_id}.act_count".to_string(),
                        ),
                    ),
                    GameActionDef::SetLocalFact(
                        "action_labels".to_string(),
                        bevy_fact_rule_event::LocalFactValue::Expr(
                            "$${current_enemy_id}.action_labels".to_string(),
                        ),
                    ),
                ],
                modifications: Vec::new(),
                outputs: Vec::new(),
                enabled: true,
                priority: 0,
                consume_event: true,
            }],
        };
        let act_rule = fre
            .rules
            .iter()
            .find(|rule| {
                rule.actions.iter().any(|action| {
                    matches!(action, GameActionDef::SetLocalFact(key, _) if key == "act_count")
                })
            })
            .expect("projected menu rule should set action count");

        let mut enum_registry = EnumRegistry::default();
        enum_registry.register_from_asset(&fre);
        let mut global_facts = LayeredFactDatabase::new();
        global_facts.set("mad_dummy.act_count", FactValue::Int(2));
        global_facts.set(
            "mad_dummy.action_labels",
            FactValue::StringList(vec!["Check".into(), "Talk".into()]),
        );
        global_facts.set(
            "mad_dummy.action_sequences",
            FactValue::StringList(vec![
                "check.sequence.ron".into(),
                "talk.sequence.ron".into(),
            ]),
        );
        global_facts.set(
            "mad_dummy.action_params",
            FactValue::StringList(vec!["check".into(), "talk".into()]),
        );

        let mut view_root = ViewRoot::new("battle/view/undertale.view.ron".to_string());
        view_root.set_local_value("interactable", true);
        view_root.set_local_value("depth", FactValue::Int(1));
        view_root.set_local_value("menu_context", FactValue::Int(1));
        view_root.set_local_value("enemy_selection", FactValue::Int(0));
        view_root.set_local_value(
            "enemy_ids",
            FactValue::StringList(vec!["mad_dummy".to_string()]),
        );

        for action in &act_rule.actions {
            if let GameActionDef::SetLocalFact(key, value) = action {
                let combined = CombinedFactReader::new(view_root.local_state(), &global_facts);
                let fact_value = evaluate_local_fact_value(key, value, &combined, &enum_registry);
                view_root.set_local_value(key.as_str(), fact_value);
            }
        }

        assert_eq!(
            view_root.local_state().get_string("current_enemy_id"),
            Some("mad_dummy")
        );
        assert_eq!(view_root.local_state().get_int("act_count"), Some(2));
        assert_eq!(
            view_root.local_state().get_string_list("action_labels"),
            Some(&["Check".to_string(), "Talk".to_string()][..])
        );
    }
}
