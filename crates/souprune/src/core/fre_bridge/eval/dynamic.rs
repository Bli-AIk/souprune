//! Resolves dynamic fact-key and array-index expressions used inside FRE values.
//!
//! 解析 FRE 值表达式里使用的动态 fact 键名与数组索引语法。
//!
//! This file handles the cases where a rule cannot point at a static fact key:
//! keys assembled from other facts, and indexed access into list-valued facts.
//! Those features are used heavily by battle/menu logic, so they are isolated
//! here instead of being duplicated across multiple evaluators.
//!
//! 这个文件处理规则无法直接写死 fact 键名时的两种情况：由其他 fact 组装出来的
//! 动态键，以及对列表型 fact 的索引访问。这两类语法在战斗和菜单逻辑里很常见，
//! 所以被单独隔离在这里，而不是在多个求值器里重复实现。

use bevy::prelude::*;
use bevy_fact_rule_event::{FactReader, FactValue};

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
pub(super) fn try_evaluate_dynamic_key(expr: &str, facts: &dyn FactReader) -> Option<FactValue> {
    if !expr.starts_with("$${") || !expr.contains('}') {
        return None;
    }

    let brace_end = expr.find('}')?;
    let prefix_var = &expr[3..brace_end];

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

    let suffix = &expr[brace_end + 1..];

    if let Some(bracket_start) = suffix.find('[')
        && suffix.ends_with(']')
    {
        let array_suffix = &suffix[..bracket_start];
        let index_expr = &suffix[bracket_start + 1..suffix.len() - 1];
        let full_key = format!("{}{}", prefix_value, array_suffix);

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
pub(super) fn try_evaluate_array_index(expr: &str, facts: &dyn FactReader) -> Option<FactValue> {
    if !expr.starts_with('$') || !expr.contains('[') || !expr.ends_with(']') {
        return None;
    }

    let bracket_start = expr.find('[')?;
    let array_name = &expr[1..bracket_start];
    let index_expr = &expr[bracket_start + 1..expr.len() - 1];

    let index: usize = if let Some(index_var_name) = index_expr.strip_prefix('$') {
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
        index_expr.parse::<usize>().ok()?
    };

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
