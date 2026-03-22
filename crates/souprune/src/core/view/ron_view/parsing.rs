use super::super::layout::FloatOrExpr;
use crate::core::mode::SequenceSubState;
use std::collections::HashMap;

mod repeat_preprocess;
mod text_templates;

// Re-export PlayerDataView so callers can import parsing helpers from one place.
pub use super::player_data::PlayerDataView;

// Re-export evaluation helpers alongside the parsing utilities.
#[allow(unused_imports)]
pub use super::evaluation::{
    evaluate_condition, evaluate_dynamic_color, evaluate_fact_expression, evaluate_float_expr,
    evaluate_float_expr_with_current, evaluate_float_expr_with_repeat,
    evaluate_transition_condition_unified, evaluate_visible_when,
    preprocess_fact_expressions_with_repeat, resolve_val_bool, resolve_val_f32,
};
pub use repeat_preprocess::preprocess_sprite_def_for_repeat;
pub use text_templates::{resolve_data_path, resolve_text_content};

/// Context for repeat variable substitution in UI elements.
/// Used when spawning repeated UI elements like HP bars.
#[derive(Clone, Debug, Default)]
pub struct RepeatContext {
    /// Current iteration index.
    pub index: usize,
    /// Variable bindings: name -> value.
    pub variables: HashMap<String, String>,
}

impl RepeatContext {
    pub fn new(index: usize) -> Self {
        Self {
            index,
            variables: HashMap::new(),
        }
    }

    pub fn with_item(mut self, name: &str, value: String) -> Self {
        self.variables.insert(name.to_string(), value);
        self
    }

    pub fn get_index(&self) -> usize {
        self.index
    }

    pub fn get_variable(&self, name: &str) -> Option<&str> {
        self.variables.get(name).map(|s| s.as_str())
    }
}

/// Analyze whether an expression depends on time (@time).
pub fn expression_depends_on_time(expr: &FloatOrExpr) -> bool {
    match expr {
        crate::core::sequencer::chapter_schema::Value::Static(_) => false,
        crate::core::sequencer::chapter_schema::Value::Expr(expr_str) => expr_str.contains("@time"),
    }
}

/// Check if a Vec3Tuple (translation or scale) contains time-dependent expressions.
pub fn vec3_tuple_depends_on_time(tuple: &super::super::layout::serde_types::Vec3Tuple) -> bool {
    expression_depends_on_time(&tuple.0)
        || expression_depends_on_time(&tuple.1)
        || expression_depends_on_time(&tuple.2)
}

pub fn parse_sequence_state(state_str: &str) -> Option<SequenceSubState> {
    match state_str {
        "" | "None" => None,
        name => Some(SequenceSubState::new(name)),
    }
}
