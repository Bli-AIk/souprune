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
pub(super) fn inject_sequence_params(
    view_root: &mut ViewRoot,
    params: &HashMap<String, FactValueMatch>,
    layered_db: &LayeredFactDatabase,
) {
    for (key, value) in params {
        let prefixed_key = format!("{}{}", PARAM_PREFIX, key);
        let Some(fact_value) = resolve_fact_value(value, layered_db) else {
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
) -> Option<FactValue> {
    match value {
        FactValueMatch::Bool(value) => Some(FactValue::Bool(*value)),
        FactValueMatch::Int(value) => Some(FactValue::Int(*value)),
        FactValueMatch::Float(value) => Some(FactValue::Float(*value)),
        FactValueMatch::String(value) => Some(FactValue::String(value.clone())),
        FactValueMatch::Expr(expr) => {
            if let Some(fact_key) = expr.strip_prefix('$') {
                layered_db.get_by_str(fact_key).cloned()
            } else {
                bevy_fact_rule_event::expr::evaluate_expr_to_fact(expr, layered_db)
            }
        }
    }
}
