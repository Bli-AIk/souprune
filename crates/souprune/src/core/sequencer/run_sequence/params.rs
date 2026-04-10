//! Evaluates and injects `RunSequence` parameters into a view-local fact scope.
//!
//! 对 `RunSequence` 参数求值，并把结果注入到 View 局部事实作用域中。
//!
//! Gives nested sequences a predictable way to receive call-site data. It
//! converts `FactValueMatch` arguments into concrete `FactValue`s and stores
//! them under a reserved local-fact prefix so the loaded sequence can read
//! them without colliding with existing view facts.
//!
//! 给嵌套序列提供一套稳定的入参机制。它把 `FactValueMatch`
//! 参数求值成具体 `FactValue`，并用保留的局部 fact 前缀写入当前 View，
//! 这样被加载的子序列就能读取这些参数，同时避免和现有 view facts 冲突。

use bevy::prelude::*;
use bevy_fact_rule_event::{FactValue, LayeredFactDatabase};
use std::collections::HashMap;

use crate::core::view::ViewRoot;

use super::super::chapter_schema::FactValueMatch;

/// Prefix for injected sequence parameters in local_facts.
/// Parameters are stored as `_param_{name}` to avoid conflicts.
///
/// 注入到 local_facts 的序列参数前缀。
/// 参数存储为 `_param_{name}` 以避免冲突。
const PARAM_PREFIX: &str = "_param_";

/// Inject sequence parameters into a ViewRoot's local_facts.
///
/// For `Expr("$key")` lookups, checks the ViewRoot's own local_facts first
/// (where parent RunSequence params are stored) before falling back to
/// the LayeredFactDatabase.
pub(super) fn inject_sequence_params(
    view_root: &mut ViewRoot,
    params: &HashMap<String, FactValueMatch>,
    layered_db: &LayeredFactDatabase,
) {
    for (key, value) in params {
        let prefixed_key = format!("{}{}", PARAM_PREFIX, key);
        let Some(fact_value) = resolve_fact_value(value, layered_db, &view_root.local_facts)
        else {
            warn!("RunSequence: Failed to evaluate param '{}'", key);
            continue;
        };
        view_root.local_facts.set(prefixed_key, fact_value.clone());
        info!("RunSequence: Injected param '{}' = {:?}", key, fact_value);
    }
}

fn resolve_fact_value(
    value: &FactValueMatch,
    layered_db: &LayeredFactDatabase,
    local_facts: &bevy_fact_rule_event::FactDatabase,
) -> Option<FactValue> {
    match value {
        FactValueMatch::Bool(value) => Some(FactValue::Bool(*value)),
        FactValueMatch::Int(value) => Some(FactValue::Int(*value)),
        FactValueMatch::Float(value) => Some(FactValue::Float(*value)),
        FactValueMatch::String(value) => Some(FactValue::String(value.clone())),
        FactValueMatch::Expr(expr) => {
            if let Some(fact_key) = expr.strip_prefix('$') {
                // Check ViewRoot local_facts first (parent RunSequence params),
                // then fall back to layered DB.
                local_facts
                    .get_by_str(fact_key)
                    .or_else(|| layered_db.get_by_str(fact_key))
                    .cloned()
            } else {
                bevy_fact_rule_event::expr::evaluate_expr_to_fact(expr, layered_db)
            }
        }
    }
}
