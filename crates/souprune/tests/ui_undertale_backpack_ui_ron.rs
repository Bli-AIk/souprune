//! Backpack UI layout RON tests for the Undertale-style menu.
//!
//! Undertale 风格背包 UI 布局 RON 测试。

#[path = "test_support.rs"]
mod test_support;

use evalexpr::{ContextWithMutableVariables, HashMapContext, Value, eval_with_context};
use std::collections::HashSet;

use souprune::{IndexBoundDef, TransitionActionDef, UIBoxLogicDef, UILayoutAsset};

const LAYERS: &[&str] = &[
    "BackpackMenu",
    "BackpackItem",
    "BackpackItemOptions",
    "BackpackStatus",
];

const OVERWORLD_STATES: &[&str] = &["Normal", "Backpack", "Cutscene"];

/// Confirm the UI layout parses and exposes expected root nodes.
///
/// 确认 UI 布局可以成功解析并提供预期的根节点。
#[test]
fn backpack_ui_layout_deserializes() {
    let layout: UILayoutAsset = test_support::parse_project_ron("ui/undertale_backpack.ui.ron");
    assert_eq!(layout.version, 1);
    assert_eq!(layout.roots.len(), 4);
    assert!(layout.navigation.is_some(), "navigation rules should exist");
}

/// Ensure every referenced UI layer or state belongs to known sets.
///
/// 确保被引用的 UI 层与状态都属于预期集合。
#[test]
fn backpack_ui_reference_integrity() {
    let layout: UILayoutAsset = test_support::parse_project_ron("ui/undertale_backpack.ui.ron");

    let allowed_layers: HashSet<&str> = LAYERS.iter().copied().collect();
    let allowed_actions = ["Up", "Down", "Left", "Right"];

    let navigation = layout
        .navigation
        .as_ref()
        .expect("navigation data required");
    for (layer, rule) in navigation {
        assert!(
            allowed_layers.contains(layer.as_str()),
            "unknown navigation layer {layer}"
        );
        for action in rule.mappings.keys() {
            assert!(
                allowed_actions.contains(&action.as_str()),
                "unexpected navigation action {action}"
            );
        }
        if let Some(IndexBoundDef::Dynamic(expr)) = &rule.max_index {
            assert!(
                expr.contains("inventory"),
                "dynamic max_index should reference inventory state"
            );
        }
    }

    if let Some(transitions) = &layout.transitions {
        for (layer, def) in transitions {
            assert!(
                allowed_layers.contains(layer.as_str()),
                "unknown transition layer {layer}"
            );
            if let Some(on_confirm) = &def.on_confirm {
                for rule in on_confirm {
                    if let TransitionActionDef::GotoLayer(target) = &rule.action {
                        assert!(
                            allowed_layers.contains(target.as_str()),
                            "transition targets unknown layer {target}"
                        );
                    }
                }
            }
            if let Some(action) = &def.on_cancel {
                if let TransitionActionDef::GotoLayer(target) = action {
                    assert!(
                        allowed_layers.contains(target.as_str()),
                        "cancel action targets unknown layer {target}"
                    );
                }
            }
        }
    }

    if let Some(triggers) = &layout.global_triggers {
        for (_name, rules) in triggers {
            for rule in rules {
                assert!(
                    OVERWORLD_STATES.contains(&rule.target_state.as_str()),
                    "unknown overworld state {}",
                    rule.target_state
                );
                if let Some(states) = &rule.allowed_states {
                    for state in states {
                        assert!(
                            OVERWORLD_STATES.contains(&state.as_str()),
                            "unknown allowed state {state}"
                        );
                    }
                }
            }
        }
    }
}

/// Evaluate dynamic expressions (conditional offsets & inventory bounds) to preflight runtime logic.
///
/// 评估动态表达式（条件偏移与背包限制）以提前验证运行时逻辑。
#[test]
fn backpack_ui_logic_expressions_rehearse() {
    let layout: UILayoutAsset = test_support::parse_project_ron("ui/undertale_backpack.ui.ron");

    let info_box = layout
        .roots
        .iter()
        .find(|node| node.name == "InfoBox")
        .expect("InfoBox node should exist");
    let Some(UIBoxLogicDef { offset, .. }) = &info_box.ui_shape_logic else {
        panic!("InfoBox should define ui_shape_logic");
    };
    let y_expr = offset
        .y
        .as_expr()
        .expect("InfoBox offset.y should be dynamic");

    let above_camera = evaluate_anchor_expr(y_expr, 50.0, -10.0);
    assert!(
        (above_camera + 68.5).abs() < f32::EPSILON,
        "InfoBox should shift upward when player.y > camera.y"
    );
    let below_camera = evaluate_anchor_expr(y_expr, -20.0, 40.0);
    assert!(
        (below_camera - 66.5).abs() < f32::EPSILON,
        "InfoBox should drop below when player.y <= camera.y"
    );

    let navigation = layout.navigation.as_ref().expect("navigation required");
    let item_rule = navigation
        .get("BackpackItem")
        .expect("BackpackItem navigation should exist");
    let IndexBoundDef::Dynamic(expr) = item_rule
        .max_index
        .as_ref()
        .expect("BackpackItem max_index must exist")
    else {
        panic!("BackpackItem max_index must be dynamic");
    };

    assert_eq!(
        evaluate_inventory_expr(expr, 3, 8),
        3,
        "small inventory should limit index to item count"
    );
    assert_eq!(
        evaluate_inventory_expr(expr, 10, 6),
        6,
        "inventory capacity should clamp when len exceeds limit"
    );
}

fn evaluate_anchor_expr(expr: &str, player_y: f32, camera_y: f32) -> f32 {
    let mut context = HashMapContext::new();
    let _ = context.set_value("player.y".into(), Value::Float(player_y as f64));
    let _ = context.set_value("player.x".into(), Value::Float(0.0));
    let _ = context.set_value("camera.y".into(), Value::Float(camera_y as f64));
    let _ = context.set_value("camera.x".into(), Value::Float(0.0));

    let value: Value = eval_with_context(expr, &context).expect("expression should evaluate");
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
        .unwrap_or_else(|_| panic!("unsupported expression {expr}"))
}
