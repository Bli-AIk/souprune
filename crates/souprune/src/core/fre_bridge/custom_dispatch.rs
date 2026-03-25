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
use bevy_fact_rule_event::{EnumRegistry, FactEvent, LayeredFactDatabase};

use crate::core::game_action::{
    GameActionDef, GameActionHandlerRegistry, GameRule, GameRuleRegistry,
};

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
