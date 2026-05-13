//! Applies pre-spawn FRE events during RON view construction.
//!
//! 在 RON View 构建期间应用预生成 FRE 事件。

use crate::core::game_action::{GameActionDef, GameRule, GameRuleDef};
use crate::core::view::components::ViewRoot;
use bevy::prelude::info;
use bevy_fact_rule_event::{CombinedFactReader, LayeredFactDatabase, RuleScope};

pub(super) fn apply_pre_spawn_events(
    view_root: &mut ViewRoot,
    rule_defs: &[GameRuleDef],
    pre_spawn_events: &[String],
    layered_db: &LayeredFactDatabase,
    enum_registry: &bevy_fact_rule_event::EnumRegistry,
) {
    if pre_spawn_events.is_empty() || rule_defs.is_empty() {
        return;
    }

    for event_id in pre_spawn_events {
        let mut matching_rules = rule_defs
            .iter()
            .enumerate()
            .filter_map(|(index, rule_def)| {
                let rule = rule_def.to_rule_with_index(index, RuleScope::View);
                (rule.enabled && rule.trigger.0 == *event_id).then_some(rule)
            })
            .collect::<Vec<_>>();

        matching_rules.sort_by_key(|rule| std::cmp::Reverse(rule.priority));

        for rule in matching_rules {
            if !pre_spawn_rule_conditions_match(view_root, layered_db, enum_registry, &rule) {
                continue;
            }

            apply_pre_spawn_set_local_fact_actions(
                view_root,
                layered_db,
                enum_registry,
                &rule.actions,
            );

            info!(
                "[ViewRoot] Applied pre-spawn event '{}' via rule '{}'",
                event_id, rule.id
            );

            if rule.consume_event {
                break;
            }
        }
    }
}

fn pre_spawn_rule_conditions_match(
    view_root: &ViewRoot,
    layered_db: &LayeredFactDatabase,
    enum_registry: &bevy_fact_rule_event::EnumRegistry,
    rule: &GameRule,
) -> bool {
    let combined = CombinedFactReader::new(view_root.local_state().as_facts(), layered_db);
    crate::core::fre_bridge::evaluate_conditions(
        &rule.condition_expressions,
        &combined,
        enum_registry,
    )
}

fn apply_pre_spawn_set_local_fact_actions(
    view_root: &mut ViewRoot,
    layered_db: &LayeredFactDatabase,
    enum_registry: &bevy_fact_rule_event::EnumRegistry,
    actions: &[GameActionDef],
) {
    for action in actions {
        let GameActionDef::SetLocalFact(key, value) = action else {
            continue;
        };
        let combined = CombinedFactReader::new(view_root.local_state().as_facts(), layered_db);
        let fact_value = crate::core::fre_bridge::evaluate_local_fact_value(
            key,
            value,
            &combined,
            enum_registry,
        );
        view_root
            .local_state_mut_for_owner()
            .set(key.as_str(), fact_value);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_fact_rule_event::{FactValue, LocalFactValue, RuleEventDef};

    #[test]
    fn pre_spawn_event_applies_set_local_fact_before_initial_spawn() {
        let mut view_root = ViewRoot::new("overworld/backpack.view.ron".into());
        view_root
            .local_state_mut_for_owner()
            .set("info_box_y_offset", 0);

        let mut layered_db = LayeredFactDatabase::new();
        layered_db.set_global("overworld:player_screen_y", FactValue::Float(130.1));

        let rule_defs = vec![GameRuleDef {
            id: "move_info_box_down_when_player_is_low".into(),
            event: RuleEventDef::Event("overworld:screen_facts_updated".into()),
            conditions: vec![
                "$overworld:player_screen_y > 130".into(),
                "$info_box_y_offset != 135".into(),
            ],
            actions: vec![GameActionDef::SetLocalFact(
                "info_box_y_offset".into(),
                LocalFactValue::Int(135),
            )],
            modifications: Vec::new(),
            outputs: Vec::new(),
            enabled: true,
            priority: 100,
            consume_event: true,
        }];

        apply_pre_spawn_events(
            &mut view_root,
            &rule_defs,
            &["overworld:screen_facts_updated".into()],
            &layered_db,
            &bevy_fact_rule_event::EnumRegistry::default(),
        );

        assert_eq!(
            view_root.local_state().get_by_str("info_box_y_offset"),
            Some(&FactValue::Int(135))
        );
    }
}
