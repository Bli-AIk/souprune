//! Resolves literal, variable, arithmetic, and list expressions used by FRE actions.
//!
//! 解析 FRE 动作里会用到的字面量、变量、算术和列表表达式。
//!
//! Acts as the value-producing side of the FRE expression layer. It turns
//! strings from rule assets into concrete `FactValue`s or integers so actions
//! such as `SetLocalFact` can reuse one consistent evaluator instead of each
//! action parsing mini-expressions on its own.
//!
//! FRE 表达式层里负责产出值的一侧。它把规则资源里的字符串表达式
//! 解析成具体的 `FactValue` 或整数，让 `SetLocalFact` 之类的动作可以复用同一套
//! 求值器，而不是每个动作各写一遍自己的小型解析逻辑。

use super::dynamic::{try_evaluate_array_index, try_evaluate_dynamic_key};
use bevy::prelude::*;
use bevy_fact_rule_event::{EnumRegistry, FactReader, FactValue, FactValueDef, LocalFactValue};

pub(super) fn resolve_value(expr: &str, facts: &dyn FactReader) -> Option<FactValue> {
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

    if let Ok(v) = expr.parse::<i64>() {
        return Some(FactValue::Int(v));
    }

    if expr == "true" {
        return Some(FactValue::Bool(true));
    }
    if expr == "false" {
        return Some(FactValue::Bool(false));
    }

    if let Ok(v) = expr.parse::<f64>() {
        return Some(FactValue::Float(v));
    }

    if expr.len() >= 2
        && ((expr.starts_with('\'') && expr.ends_with('\''))
            || (expr.starts_with('"') && expr.ends_with('"')))
    {
        let inner = &expr[1..expr.len() - 1];
        return Some(FactValue::String(inner.to_string()));
    }

    None
}

pub(super) fn resolve_int(expr: &str, facts: &dyn FactReader) -> Option<i64> {
    let expr = expr.trim();

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

    if let Some(idx) = expr.find(" % ") {
        let left = expr[..idx].trim();
        let right = expr[idx + 3..].trim();
        let left_val = resolve_int(left, facts)?;
        let right_val = resolve_int(right, facts)?;
        if right_val != 0 {
            return Some(left_val % right_val);
        }
        return None;
    }

    if let Some(base) = expr.strip_suffix(".len()")
        && let Some(var_name) = base.strip_prefix('$')
        && let Some(FactValue::StringList(list)) = facts.get_by_str(var_name)
    {
        return Some(list.len() as i64);
    }
    if expr.ends_with(".len()") {
        return None;
    }

    if let Some(var_name) = expr.strip_prefix('$') {
        if let Some(val) = facts.get_int(var_name) {
            return Some(val);
        }
        // Bool → i64 coercion: true=1, false=0 (enables arithmetic on bool facts)
        // Bool → i64 强制转换：true=1, false=0（允许对布尔 fact 进行算术运算）
        if let Some(FactValue::Bool(b)) = facts.get_by_str(var_name) {
            return Some(if *b { 1 } else { 0 });
        }
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
pub(crate) fn evaluate_local_fact_value(
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
pub(super) fn evaluate_simple_expression(expr: &str, facts: &dyn FactReader) -> Option<FactValue> {
    let expr = expr.trim();

    if let Some(result) = try_evaluate_dynamic_key(expr, facts) {
        return Some(result);
    }

    if let Some(result) = try_evaluate_array_index(expr, facts) {
        return Some(result);
    }

    if expr.starts_with('$') && !expr.contains(' ') {
        let var_name = &expr[1..];
        return facts.get_by_str(var_name).cloned();
    }

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

    if let Ok(v) = expr.parse::<i64>() {
        return Some(FactValue::Int(v));
    }

    None
}
