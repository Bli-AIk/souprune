//! Evaluates boolean rule conditions against the current FRE fact view.
//!
//! 在当前 FRE 事实视图之上评估规则条件表达式。
//!
//! Acts as the condition-checking half of the FRE expression layer. It
//! parses comparison operators, resolves operands through the shared expression
//! helpers, and produces the boolean answers that decide whether a rule may
//! execute.
//!
//! FRE 表达式层里负责条件判断的一半。它解析比较运算符，借助共享的
//! 表达式解析辅助逻辑求出左右两侧的值，并最终给出某条规则是否允许执行的布尔结果。

use super::expressions::{resolve_number, resolve_value};
use bevy::prelude::*;
use bevy_fact_rule_event::{EnumRegistry, FactReader, FactValue};

/// Evaluate condition expressions against a FactReader.
/// Returns true if all conditions pass.
///
/// 根据 FactReader 评估条件表达式。
/// 如果所有条件都通过则返回 true。
pub(crate) fn evaluate_conditions(
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

    if let Some(idx) = condition.find(" == ") {
        let left = condition[..idx].trim();
        let right = condition[idx + 4..].trim();
        if left.contains(" % ") || left.contains(" + ") || left.contains(" - ") {
            return compare_numbers(left, right, facts, |a, b| a == b);
        }
        return compare_values(left, right, facts, enums, |a, b| a == b);
    }

    if let Some(idx) = condition.find(" != ") {
        let left = condition[..idx].trim();
        let right = condition[idx + 4..].trim();
        if left.contains(" % ") || left.contains(" + ") || left.contains(" - ") {
            return compare_numbers(left, right, facts, |a, b| a != b);
        }
        return compare_values(left, right, facts, enums, |a, b| a != b);
    }

    if let Some(idx) = condition.find(" >= ") {
        let left = condition[..idx].trim();
        let right = condition[idx + 4..].trim();
        return compare_numbers(left, right, facts, |a, b| a >= b);
    }

    if let Some(idx) = condition.find(" <= ") {
        let left = condition[..idx].trim();
        let right = condition[idx + 4..].trim();
        return compare_numbers(left, right, facts, |a, b| a <= b);
    }

    if let Some(idx) = condition.find(" > ") {
        let left = condition[..idx].trim();
        let right = condition[idx + 3..].trim();
        return compare_numbers(left, right, facts, |a, b| a > b);
    }

    if let Some(idx) = condition.find(" < ") {
        let left = condition[..idx].trim();
        let right = condition[idx + 3..].trim();
        return compare_numbers(left, right, facts, |a, b| a < b);
    }

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

fn compare_numbers<F>(left: &str, right: &str, facts: &dyn FactReader, cmp: F) -> bool
where
    F: Fn(f64, f64) -> bool,
{
    let left_val = resolve_number(left, facts);
    let right_val = resolve_number(right, facts);

    match (left_val, right_val) {
        (Some(l), Some(r)) => cmp(l, r),
        _ => false,
    }
}
