//! FRE condition evaluation and expression resolution helpers.
//!
//! This submodule contains all condition evaluation, value resolution,
//! and expression evaluation logic used by the FRE bridge.
//!
//! FRE 条件评估和表达式解析辅助函数。
//!
//! 此子模块包含 FRE 桥接使用的所有条件评估、值解析和表达式评估逻辑。

mod conditions;
mod dynamic;
mod expressions;

use bevy::prelude::*;
use bevy_fact_rule_event::EnumRegistry;

pub(crate) use conditions::evaluate_conditions;
pub use conditions::evaluate_single_condition;
pub(crate) use expressions::evaluate_local_fact_value;

/// Souprune's implementation of condition evaluation.
/// This evaluator is registered with the FRE system to evaluate rule conditions.
///
/// Souprune 的条件评估实现。
/// 此评估器注册到 FRE 系统以评估规则条件。
pub(super) struct SoupruneConditionEvaluator;

impl bevy_fact_rule_event::ConditionEvaluatorTrait for SoupruneConditionEvaluator {
    fn evaluate(
        &self,
        conditions: &[String],
        facts: &dyn bevy_fact_rule_event::FactReader,
        enums: &EnumRegistry,
    ) -> bool {
        evaluate_conditions(conditions, facts, enums)
    }
}

/// System to register the Souprune condition evaluator with the FRE system.
/// Should run at startup, after FREPlugin is built.
///
/// 将 Souprune 条件评估器注册到 FRE 系统的系统。
/// 应在启动时运行，在 FREPlugin 构建之后。
pub(super) fn register_condition_evaluator_system(mut commands: Commands) {
    commands.insert_resource(bevy_fact_rule_event::ConditionEvaluator::new(
        SoupruneConditionEvaluator,
    ));
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_fact_rule_event::{FactDatabase, FactValue, LayeredFactDatabase};

    fn empty_enums() -> EnumRegistry {
        EnumRegistry::default()
    }

    #[test]
    fn test_evaluate_simple_expression_variable() {
        let mut facts = FactDatabase::new();
        facts.set("selection", FactValue::Int(3));

        let result = expressions::evaluate_simple_expression("$selection", &facts);
        assert_eq!(result, Some(FactValue::Int(3)));
    }

    #[test]
    fn test_evaluate_simple_expression_addition() {
        let mut facts = FactDatabase::new();
        facts.set("selection", FactValue::Int(3));

        let result = expressions::evaluate_simple_expression("$selection + 1", &facts);
        assert_eq!(result, Some(FactValue::Int(4)));
    }

    #[test]
    fn test_evaluate_simple_expression_subtraction() {
        let mut facts = FactDatabase::new();
        facts.set("selection", FactValue::Int(3));

        let result = expressions::evaluate_simple_expression("$selection - 1", &facts);
        assert_eq!(result, Some(FactValue::Int(2)));
    }

    #[test]
    fn test_evaluate_simple_expression_literal() {
        let facts = FactDatabase::new();

        let result = expressions::evaluate_simple_expression("42", &facts);
        assert_eq!(result, Some(FactValue::Int(42)));
    }

    #[test]
    fn test_evaluate_single_condition_equals() {
        let mut facts = FactDatabase::new();
        facts.set("depth", FactValue::Int(0));
        let enums = empty_enums();

        assert!(evaluate_single_condition("$depth == 0", &facts, &enums));
        assert!(!evaluate_single_condition("$depth == 1", &facts, &enums));
    }

    #[test]
    fn test_evaluate_single_condition_greater_than() {
        let mut facts = FactDatabase::new();
        facts.set("selection", FactValue::Int(3));
        let enums = empty_enums();

        assert!(evaluate_single_condition("$selection > 0", &facts, &enums));
        assert!(!evaluate_single_condition("$selection > 3", &facts, &enums));
    }

    #[test]
    fn test_evaluate_single_condition_less_than() {
        let mut facts = FactDatabase::new();
        facts.set("selection", FactValue::Int(3));
        let enums = empty_enums();

        assert!(evaluate_single_condition("$selection < 5", &facts, &enums));
        assert!(!evaluate_single_condition("$selection < 3", &facts, &enums));
    }

    #[test]
    fn test_evaluate_single_condition_float_comparison() {
        let mut facts = FactDatabase::new();
        facts.set("top_down:player_screen_y", FactValue::Float(130.1));
        let enums = empty_enums();

        assert!(evaluate_single_condition(
            "$top_down:player_screen_y > 130.0",
            &facts,
            &enums
        ));
        assert!(!evaluate_single_condition(
            "$top_down:player_screen_y <= 130.0",
            &facts,
            &enums
        ));
    }

    #[test]
    fn test_evaluate_conditions_all_pass() {
        let mut facts = FactDatabase::new();
        facts.set("depth", FactValue::Int(0));
        facts.set("selection", FactValue::Int(1));
        let enums = empty_enums();

        let conditions = vec!["$depth == 0".to_string(), "$selection < 5".to_string()];

        assert!(evaluate_conditions(&conditions, &facts, &enums));
    }

    #[test]
    fn test_evaluate_conditions_one_fails() {
        let mut facts = FactDatabase::new();
        facts.set("depth", FactValue::Int(0));
        facts.set("selection", FactValue::Int(1));
        let enums = empty_enums();

        let conditions = vec!["$depth == 1".to_string(), "$selection < 5".to_string()];

        assert!(!evaluate_conditions(&conditions, &facts, &enums));
    }

    #[test]
    fn test_evaluate_string_list_len() {
        let mut global_facts = LayeredFactDatabase::default();
        global_facts.set_global(
            "test:string_list",
            FactValue::StringList(vec![
                "item1".to_string(),
                "item2".to_string(),
                "item3".to_string(),
            ]),
        );
        let enums = empty_enums();

        let conditions = vec!["$test:string_list.len() == 3".to_string()];
        assert!(evaluate_conditions(&conditions, &global_facts, &enums));

        let conditions2 = vec!["$test:string_list.len() > 2".to_string()];
        assert!(evaluate_conditions(&conditions2, &global_facts, &enums));
    }

    #[test]
    fn test_enum_resolution_in_conditions() {
        let mut facts = FactDatabase::new();
        facts.set("depth", FactValue::Int(0));
        facts.set("menu_context", FactValue::Int(2));

        let mut enums = EnumRegistry::default();
        enums.register(
            "depth",
            &["main".into(), "submenu".into(), "options".into()],
        );
        enums.register(
            "menu_context",
            &["fight".into(), "act".into(), "item".into(), "mercy".into()],
        );

        assert!(evaluate_single_condition(
            "$depth == 'main'",
            &facts,
            &enums
        ));
        assert!(!evaluate_single_condition(
            "$depth == 'submenu'",
            &facts,
            &enums
        ));
        assert!(evaluate_single_condition(
            "$menu_context == 'item'",
            &facts,
            &enums
        ));
        assert!(!evaluate_single_condition(
            "$menu_context == 'fight'",
            &facts,
            &enums
        ));
        assert!(evaluate_single_condition(
            "$depth != 'submenu'",
            &facts,
            &enums
        ));
        assert!(!evaluate_single_condition(
            "$depth == 'nonexistent'",
            &facts,
            &enums
        ));
    }

    #[test]
    fn test_enum_registry_resolve() {
        let mut enums = EnumRegistry::default();
        enums.register(
            "depth",
            &["main".into(), "submenu".into(), "options".into()],
        );

        assert_eq!(enums.resolve("depth", "main"), Some(0));
        assert_eq!(enums.resolve("depth", "submenu"), Some(1));
        assert_eq!(enums.resolve("depth", "options"), Some(2));
        assert_eq!(enums.resolve("depth", "nonexistent"), None);
        assert_eq!(enums.resolve("nonexistent_group", "main"), None);

        assert_eq!(enums.reverse_resolve("depth", 0), Some("main"));
        assert_eq!(enums.reverse_resolve("depth", 1), Some("submenu"));
    }
}
