//! View `.view_layout.ron` layout tests.
//!
//! `.view_layout.ron` 布局测试。

#[path = "test_support.rs"]
mod test_support;

use evalexpr::{ContextWithMutableVariables, HashMapContext, Value, eval_with_context};
use std::collections::{HashMap, HashSet};

use souprune::{
    IndexBoundDef, ReactivePositionDef, SerializableVec3, TransitionActionDef, ViewLayoutAsset,
    ViewNodeDef,
};

const VIEW_DIRS: &[&str] = &["overworld/view", "battle/view"];
const VIEW_SUFFIX: &str = ".view_layout.ron";

fn view_files() -> Vec<String> {
    let mut files = Vec::new();
    for dir in VIEW_DIRS {
        files.extend(test_support::list_project_files_with_suffix(
            dir,
            VIEW_SUFFIX,
        ));
    }
    assert!(
        !files.is_empty(),
        "No .view_layout.ron files found under projects/example_mod"
    );
    files
}

fn load_view_layouts() -> HashMap<String, ViewLayoutAsset> {
    view_files()
        .into_iter()
        .map(|relative| {
            let asset: ViewLayoutAsset = test_support::parse_project_ron(&relative);
            (relative, asset)
        })
        .collect()
}

/// Ensure every `.view_layout.ron` file loads.
///
/// 确保所有 `.view_layout.ron` 文件均可加载。
#[test]
fn view_layouts_deserialize() {
    for (relative, layout) in load_view_layouts() {
        assert!(
            layout.version > 0,
            "{} should define a schema version",
            relative
        );
        assert!(
            !layout.roots.is_empty(),
            "{} should provide root nodes",
            relative
        );
    }
}

fn collect_node_names(nodes: &[ViewNodeDef], acc: &mut HashSet<String>) {
    for node in nodes {
        acc.insert(node.name.clone());
        collect_node_names(&node.children, acc);
    }
}

fn flatten_nodes<'a>(node: &'a ViewNodeDef, nodes: &mut Vec<&'a ViewNodeDef>) {
    nodes.push(node);
    for child in &node.children {
        flatten_nodes(child, nodes);
    }
}

fn all_nodes(layout: &ViewLayoutAsset) -> Vec<&ViewNodeDef> {
    let mut nodes = Vec::new();
    for node in &layout.roots {
        flatten_nodes(node, &mut nodes);
    }
    nodes
}

fn is_valid_action(name: &str) -> bool {
    matches!(
        name,
        "Up" | "Down" | "Left" | "Right" | "Confirm" | "Cancel" | "Menu"
    )
}

const OVERWORLD_STATES: &[&str] = &["Normal", "Backpack", "Cutscene"];

/// Verify navigation, transitions, and triggers reference existing layers/states.
///
/// 验证导航、跳转与触发引用的层或状态均有效。
#[test]
fn view_layout_reference_integrity() {
    for (relative, layout) in load_view_layouts() {
        let mut defined_layers = HashSet::new();
        collect_node_names(&layout.roots, &mut defined_layers);
        if let Some(nav) = &layout.navigation {
            for layer in nav.keys() {
                defined_layers.insert(layer.clone());
            }
        }
        if let Some(transitions) = &layout.transitions {
            for layer in transitions.keys() {
                defined_layers.insert(layer.clone());
            }
        }

        if let Some(nav) = &layout.navigation {
            for (layer, rule) in nav {
                assert!(
                    defined_layers.contains(layer),
                    "navigation layer {layer} in {relative} is undefined"
                );
                for action in rule.mappings.keys() {
                    assert!(
                        is_valid_action(action),
                        "navigation uses unknown action {action} in {relative}"
                    );
                }
                if let Some(bound) = &rule.max_index {
                    if let IndexBoundDef::Static(value) = bound {
                        assert!(
                            *value <= 64,
                            "max_index for {layer} in {relative} is unexpectedly large"
                        );
                    }
                }
            }
        }

        if let Some(transitions) = &layout.transitions {
            for (layer, def) in transitions {
                assert!(
                    defined_layers.contains(layer),
                    "transitions reference undefined layer {layer} in {relative}"
                );
                if let Some(on_confirm) = &def.on_confirm {
                    for rule in on_confirm {
                        if let TransitionActionDef::GotoLayer(target) = &rule.action {
                            assert!(
                                defined_layers.contains(target),
                                "GotoLayer target {target} in {relative} does not exist"
                            );
                        }
                    }
                }
                if let Some(action) = &def.on_cancel {
                    if let TransitionActionDef::GotoLayer(target) = action {
                        assert!(
                            defined_layers.contains(target),
                            "Cancel action target {target} in {relative} does not exist"
                        );
                    }
                }
            }
        }

        if let Some(triggers) = &layout.global_triggers {
            for rules in triggers.values() {
                for rule in rules {
                    assert!(
                        OVERWORLD_STATES.contains(&rule.target_state.as_str()),
                        "unknown overworld state {} referenced in {relative}",
                        rule.target_state
                    );
                    if let Some(states) = &rule.allowed_states {
                        for state in states {
                            assert!(
                                OVERWORLD_STATES.contains(&state.as_str()),
                                "unknown allowed state {state} referenced in {relative}"
                            );
                        }
                    }
                }
            }
        }
    }
}

fn evaluate_offset_expr(expr: &str) -> f32 {
    let mut context = HashMapContext::new();
    let _ = context.set_value("player.y".into(), Value::Float(16.0));
    let _ = context.set_value("player.x".into(), Value::Float(-8.0));
    let _ = context.set_value("camera.y".into(), Value::Float(4.0));
    let _ = context.set_value("camera.x".into(), Value::Float(2.0));
    let value: Value =
        eval_with_context(expr, &context).expect("dynamic anchor expression should evaluate");
    match value {
        Value::Float(v) => v as f32,
        Value::Int(v) => v as f32,
        other => panic!("expression returned non-numeric value: {other:?}"),
    }
}

fn evaluate_inventory_expr(expr: &str, inventory_len: usize, capacity: usize) -> usize {
    let expr = expr.trim();
    if expr == "inventory.len()" {
        return inventory_len;
    }
    if expr == "inventory_capacity" {
        return capacity;
    }
    if let Some(inner) = expr
        .strip_prefix("min(")
        .and_then(|rest| rest.strip_suffix(')'))
    {
        let parts: Vec<&str> = inner.split(',').map(|s| s.trim()).collect();
        if parts.len() == 2 {
            let a = evaluate_inventory_expr(parts[0], inventory_len, capacity);
            let b = evaluate_inventory_expr(parts[1], inventory_len, capacity);
            return a.min(b);
        }
    }
    if let Some(inner) = expr
        .strip_prefix("max(")
        .and_then(|rest| rest.strip_suffix(')'))
    {
        let parts: Vec<&str> = inner.split(',').map(|s| s.trim()).collect();
        if parts.len() == 2 {
            let a = evaluate_inventory_expr(parts[0], inventory_len, capacity);
            let b = evaluate_inventory_expr(parts[1], inventory_len, capacity);
            return a.max(b);
        }
    }
    expr.parse()
        .unwrap_or_else(|_| panic!("unsupported index expression {expr}"))
}

fn check_vec_expr(vec: &SerializableVec3) {
    for expr in [vec.0.as_expr(), vec.1.as_expr(), vec.2.as_expr()] {
        if let Some(expr) = expr {
            let value = evaluate_offset_expr(expr);
            assert!(
                value.is_finite(),
                "expression {expr} should evaluate to a finite number"
            );
        }
    }
}

/// Evaluate dynamic expressions (anchors or navigation bounds) to ensure they are valid.
///
/// 评估动态表达式（锚点或导航范围）以确保其有效。
#[test]
fn view_layout_dynamic_expressions_evaluate() {
    for (relative, layout) in load_view_layouts() {
        for node in all_nodes(&layout) {
            if let Some(indicator) = &node.reactive_indicator {
                if let Some(pos) = &indicator.default_translation {
                    match pos {
                        ReactivePositionDef::Static(vec) => check_vec_expr(vec),
                        ReactivePositionDef::Linear { origin, step } => {
                            check_vec_expr(origin);
                            check_vec_expr(step);
                        }
                        ReactivePositionDef::Custom { positions } => {
                            for pos in positions {
                                check_vec_expr(pos);
                            }
                        }
                    }
                }
            }
            if let Some(logic) = &node.ui_shape_logic {
                check_vec_expr(&logic.offset);
            }
        }

        if let Some(nav) = &layout.navigation {
            for (layer, rule) in nav {
                if let Some(IndexBoundDef::Dynamic(expr)) = &rule.min_index {
                    let value = evaluate_inventory_expr(expr, 3, 8);
                    assert!(
                        value <= 8,
                        "min_index expression {expr} for {layer} in {relative} returned unexpected value"
                    );
                }
                if let Some(IndexBoundDef::Dynamic(expr)) = &rule.max_index {
                    let value = evaluate_inventory_expr(expr, 5, 6);
                    assert!(
                        value <= 6,
                        "max_index expression {expr} for {layer} in {relative} returned unexpected value"
                    );
                }
            }
        }
    }
}
