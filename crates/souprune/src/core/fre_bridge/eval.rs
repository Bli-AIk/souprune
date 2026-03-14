//! FRE condition evaluation and expression resolution helpers.
//!
//! This submodule contains all condition evaluation, value resolution,
//! and expression evaluation logic used by the FRE bridge.
//!
//! FRE 条件评估和表达式解析辅助函数。
//!
//! 此子模块包含 FRE 桥接使用的所有条件评估、值解析和表达式评估逻辑。

use bevy::prelude::*;
use bevy_fact_rule_event::{EnumRegistry, FactReader, FactValue, FactValueDef, LocalFactValue};

/// Evaluate condition expressions against a FactReader.
/// Returns true if all conditions pass.
///
/// 根据 FactReader 评估条件表达式。
/// 如果所有条件都通过则返回 true。
pub(super) fn evaluate_conditions(
    conditions: &[String],
    facts: &dyn FactReader,
    enums: &EnumRegistry,
) -> bool {
    for condition in conditions {
        let result = evaluate_single_condition(condition, facts, enums);
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
    facts: &dyn FactReader,
    enums: &EnumRegistry,
) -> bool {
    let condition = condition.trim();

    // Try to parse comparison operators
    if let Some(idx) = condition.find(" == ") {
        let left = condition[..idx].trim();
        let right = condition[idx + 4..].trim();
        // Use compare_ints if left side contains arithmetic operators (%, +, -)
        // 如果左侧包含算术运算符（%, +, -），使用 compare_ints
        if left.contains(" % ") || left.contains(" + ") || left.contains(" - ") {
            return compare_ints(left, right, facts, |a, b| a == b);
        }
        return compare_values(left, right, facts, enums, |a, b| a == b);
    }

    if let Some(idx) = condition.find(" != ") {
        let left = condition[..idx].trim();
        let right = condition[idx + 4..].trim();
        // Use compare_ints if left side contains arithmetic operators
        if left.contains(" % ") || left.contains(" + ") || left.contains(" - ") {
            return compare_ints(left, right, facts, |a, b| a != b);
        }
        return compare_values(left, right, facts, enums, |a, b| a != b);
    }

    if let Some(idx) = condition.find(" >= ") {
        let left = condition[..idx].trim();
        let right = condition[idx + 4..].trim();
        return compare_ints(left, right, facts, |a, b| a >= b);
    }

    if let Some(idx) = condition.find(" <= ") {
        let left = condition[..idx].trim();
        let right = condition[idx + 4..].trim();
        return compare_ints(left, right, facts, |a, b| a <= b);
    }

    if let Some(idx) = condition.find(" > ") {
        let left = condition[..idx].trim();
        let right = condition[idx + 3..].trim();
        return compare_ints(left, right, facts, |a, b| a > b);
    }

    if let Some(idx) = condition.find(" < ") {
        let left = condition[..idx].trim();
        let right = condition[idx + 3..].trim();
        return compare_ints(left, right, facts, |a, b| a < b);
    }

    // Default: assume it's a boolean variable that should be true
    if let Some(var_name) = condition.strip_prefix('$') {
        return facts.get_bool(var_name).unwrap_or(false);
    }

    warn!("FRE Bridge: Unknown condition format: {}", condition);
    false
}

fn compare_values<F>(
    left: &str,
    right: &str,
    facts: &dyn FactReader,
    enums: &EnumRegistry,
    cmp: F,
) -> bool
where
    F: Fn(&FactValue, &FactValue) -> bool,
{
    let left_val = resolve_value(left, facts);
    let right_val = resolve_value(right, facts);

    match (&left_val, &right_val) {
        (Some(l), Some(r)) => {
            // Enum resolution: Int vs String → try enum lookup
            // Uses the variable name (without $) as the enum group name
            if let (FactValue::Int(_), FactValue::String(variant)) = (l, r)
                && let Some(key) = left.strip_prefix('$')
                && let Some(enum_id) = enums.resolve(key, variant)
            {
                return cmp(l, &FactValue::Int(enum_id));
            }
            if let (FactValue::String(variant), FactValue::Int(_)) = (l, r)
                && let Some(key) = right.strip_prefix('$')
                && let Some(enum_id) = enums.resolve(key, variant)
            {
                return cmp(&FactValue::Int(enum_id), r);
            }
            cmp(l, r)
        }
        _ => false,
    }
}

fn compare_ints<F>(left: &str, right: &str, facts: &dyn FactReader, cmp: F) -> bool
where
    F: Fn(i64, i64) -> bool,
{
    let left_val = resolve_int(left, facts);
    let right_val = resolve_int(right, facts);

    match (left_val, right_val) {
        (Some(l), Some(r)) => cmp(l, r),
        _ => false,
    }
}

fn resolve_value(expr: &str, facts: &dyn FactReader) -> Option<FactValue> {
    // Check for .len() suffix (e.g., $player:inventory.len())
    if let Some(base) = expr.strip_suffix(".len()")
        && let Some(var_name) = base.strip_prefix('$')
        && let Some(FactValue::StringList(list)) = facts.get_by_str(var_name)
    {
        return Some(FactValue::Int(list.len() as i64));
    }
    if expr.ends_with(".len()") {
        return None;
    }

    if let Some(var_name) = expr.strip_prefix('$') {
        return facts.get_by_str(var_name).cloned();
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
    if expr.len() >= 2
        && ((expr.starts_with('\'') && expr.ends_with('\''))
            || (expr.starts_with('"') && expr.ends_with('"')))
    {
        let inner = &expr[1..expr.len() - 1];
        return Some(FactValue::String(inner.to_string()));
    }

    None
}

fn resolve_int(expr: &str, facts: &dyn FactReader) -> Option<i64> {
    let expr = expr.trim();

    // Check for dynamic key pattern: $${prefix}.suffix
    // 检查动态键名模式：$${prefix}.suffix
    if expr.starts_with("$${") && expr.contains('}') {
        if let Some(val) = try_evaluate_dynamic_key(expr, facts) {
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
        let left_val = resolve_int(left, facts)?;
        let right_val = resolve_int(right, facts)?;
        return Some(left_val + right_val);
    }

    if let Some(idx) = expr.find(" - ") {
        let left = expr[..idx].trim();
        let right = expr[idx + 3..].trim();
        let left_val = resolve_int(left, facts)?;
        let right_val = resolve_int(right, facts)?;
        return Some(left_val - right_val);
    }

    // Modulo operation (e.g., $act_selection % 2)
    // 模运算（例如 $act_selection % 2）
    if let Some(idx) = expr.find(" % ") {
        let left = expr[..idx].trim();
        let right = expr[idx + 3..].trim();
        let left_val = resolve_int(left, facts)?;
        let right_val = resolve_int(right, facts)?;
        if right_val != 0 {
            return Some(left_val % right_val);
        }
        return None; // Avoid division by zero
    }

    // Check for .len() suffix (e.g., $player:inventory.len())
    if let Some(base) = expr.strip_suffix(".len()")
        && let Some(var_name) = base.strip_prefix('$')
        && let Some(FactValue::StringList(list)) = facts.get_by_str(var_name)
    {
        return Some(list.len() as i64);
    }
    if expr.ends_with(".len()") {
        return None;
    }

    // Simple variable reference
    if let Some(var_name) = expr.strip_prefix('$') {
        if let Some(val) = facts.get_int(var_name) {
            return Some(val);
        }
        // Check if it's a StringList - return its length
        // This allows conditions like `$enemy_selection < $enemy_names - 1`
        // where $enemy_names is a StringList
        if let Some(val) = facts.get_by_str(var_name)
            && let FactValue::StringList(list) = val
        {
            return Some(list.len() as i64);
        }
        if let Some(val) = facts.get_by_str(var_name)
            && let FactValue::IntList(list) = val
        {
            return Some(list.len() as i64);
        }
        return None;
    }

    expr.parse::<i64>().ok()
}

/// Evaluate a LocalFactValue to a FactValue.
///
/// 将 LocalFactValue 评估为 FactValue。
pub(super) fn evaluate_local_fact_value(
    key: &str,
    value: &LocalFactValue,
    facts: &dyn FactReader,
    enums: &EnumRegistry,
) -> FactValue {
    match value {
        LocalFactValue::Int(v) => FactValue::Int(*v),
        LocalFactValue::Float(v) => FactValue::Float(*v),
        LocalFactValue::Bool(v) => FactValue::Bool(*v),
        LocalFactValue::String(v) => FactValue::String(v.clone()),
        LocalFactValue::Expr(expr) => {
            if let Some(result) = evaluate_simple_expression(expr, facts) {
                result
            } else {
                warn!(
                    "FRE Bridge: Failed to evaluate expression '{}', using 0",
                    expr
                );
                FactValue::Int(0)
            }
        }
        LocalFactValue::Enum(variant) => {
            enums.resolve_fact_value_def(key, &FactValueDef::Enum(variant.clone()))
        }
    }
}

/// Simple expression evaluator for basic arithmetic.
///
/// Supports:
/// - `$name` - fact reference
/// - `$name + 1`, `$name - 1` - simple increment/decrement
/// - `$array[$index]` - array indexing (StringList/IntList)
/// - `$${prefix}.suffix` - dynamic key name (prefix resolved from fact)
///
/// 简单的表达式评估器，用于基本算术。
///
/// 支持：
/// - `$name` - fact 引用
/// - `$name + 1`、`$name - 1` - 简单增减
/// - `$array[$index]` - 数组索引（StringList/IntList）
/// - `$${prefix}.suffix` - 动态键名（prefix 从 fact 解析）
fn evaluate_simple_expression(expr: &str, facts: &dyn FactReader) -> Option<FactValue> {
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
fn try_evaluate_dynamic_key(expr: &str, facts: &dyn FactReader) -> Option<FactValue> {
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

/// Try to evaluate an array indexing expression: `$array[$index]`
///
/// Supports:
/// - StringList indexing → returns String
/// - IntList indexing → returns Int
/// - Index can be a literal number or a $variable reference
///
/// 尝试评估数组索引表达式：`$array[$index]`
fn try_evaluate_array_index(expr: &str, facts: &dyn FactReader) -> Option<FactValue> {
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

// ============================================================================
// Condition Evaluator Implementation
// ============================================================================

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
    use bevy_fact_rule_event::{FactDatabase, LayeredFactDatabase};

    fn empty_enums() -> EnumRegistry {
        EnumRegistry::default()
    }

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

        let conditions = vec![
            "$depth == 1".to_string(), // This fails
            "$selection < 5".to_string(),
        ];

        assert!(!evaluate_conditions(&conditions, &facts, &enums));
    }

    #[test]
    fn test_evaluate_string_list_len() {
        let mut global_facts = LayeredFactDatabase::default();
        global_facts.set_global(
            "player:inventory",
            FactValue::StringList(vec![
                "item1".to_string(),
                "item2".to_string(),
                "item3".to_string(),
            ]),
        );
        let enums = empty_enums();

        // Test $player:inventory.len() resolves to 3
        let conditions = vec!["$player:inventory.len() == 3".to_string()];
        assert!(evaluate_conditions(&conditions, &global_facts, &enums));

        // Test comparison with arithmetic
        let conditions2 = vec!["$player:inventory.len() > 2".to_string()];
        assert!(evaluate_conditions(&conditions2, &global_facts, &enums));
    }

    #[test]
    fn test_enum_resolution_in_conditions() {
        let mut facts = FactDatabase::new();
        facts.set("depth", FactValue::Int(0)); // "main" = 0
        facts.set("menu_context", FactValue::Int(2)); // "item" = 2

        let mut enums = EnumRegistry::default();
        enums.register(
            "depth",
            &["main".into(), "submenu".into(), "options".into()],
        );
        enums.register(
            "menu_context",
            &["fight".into(), "act".into(), "item".into(), "mercy".into()],
        );

        // $depth == 'main' → resolve 'main' to 0, compare Int(0) == Int(0)
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

        // $menu_context == 'item' → resolve 'item' to 2
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

        // != should also work
        assert!(evaluate_single_condition(
            "$depth != 'submenu'",
            &facts,
            &enums
        ));

        // Unknown enum variant falls back to normal string comparison (which fails for Int vs String)
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
