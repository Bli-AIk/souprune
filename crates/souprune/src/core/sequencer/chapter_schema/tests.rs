//! Contains parsing-focused regression tests for the sequence chapter schema.
//!
//! 放置面向解析回归的序列章节 schema 测试。
//!
//! These tests exercise representative RON snippets for values, tween targets,
//! and chapter variants so schema refactors do not silently break existing asset
//! syntax. This file is not runtime logic; it protects the serialized language
//! that designers and editors rely on.
//!
//! 这些测试会用有代表性的 RON 片段去覆盖值类型、tween 目标和章节变体，避免
//! schema 重构时悄悄破坏现有资产语法。这个文件不是运行时代码，而是在保护
//! 设计者与编辑器都依赖的那套序列化语言。

use super::*;

#[test]
fn test_val_f32_parse() {
    let ron_static = "130.0";
    let ron_expr = r#""@current""#;

    let result_static: Result<Value<f32>, _> = ron::from_str(ron_static);
    let result_expr: Result<Value<f32>, _> = ron::from_str(ron_expr);

    println!("Static parse result: {:?}", result_static);
    println!("Expr parse result: {:?}", result_expr);

    assert!(
        result_static.is_ok(),
        "Failed to parse static: {:?}",
        result_static.err()
    );
    assert!(
        result_expr.is_ok(),
        "Failed to parse expr: {:?}",
        result_expr.err()
    );
}

#[test]
fn test_vec2_tuple_parse() {
    let ron = r#"("@current", 130.0)"#;
    let result: Result<Vec2Tuple, _> = ron::from_str(ron);
    println!("Vec2Tuple parse result: {:?}", result);
    assert!(
        result.is_ok(),
        "Failed to parse Vec2Tuple: {:?}",
        result.err()
    );
}

#[test]
fn test_set_view_element_with_duration() {
    let ron = r#"#![enable(implicit_some)]
    SetViewElement(
        selector: LocalName("BattleBox"),
        target: BoxSize(to: ("@current", 130.0)),
        duration: 0.5,
        easing: QuadInOut,
        wait_for_completion: true,
    )"#;
    let result: Result<Chapter, _> = ron::from_str(ron);
    match &result {
        Ok(v) => println!("SetViewElement (animated) OK: {:?}", v),
        Err(e) => println!("SetViewElement (animated) ERR: {}", e),
    }
    assert!(
        result.is_ok(),
        "Failed to parse SetViewElement (animated): {:?}",
        result.err()
    );
}

#[test]
fn test_set_view_element_instant() {
    // No duration/easing → instant set
    let ron = r#"SetViewElement(
        selector: LocalName("BattleBox"),
        target: BoxSize(to: (175, "@current")),
    )"#;
    let result: Result<Chapter, _> = ron::from_str(ron);
    match &result {
        Ok(v) => println!("SetViewElement (instant) OK: {:?}", v),
        Err(e) => println!("SetViewElement (instant) ERR: {}", e),
    }
    assert!(
        result.is_ok(),
        "Failed to parse SetViewElement (instant): {:?}",
        result.err()
    );
}

#[test]
fn test_set_view_element_anchor() {
    let ron = r#"SetViewElement(
        selector: LocalName("BattleBox"),
        target: Anchor(0.0, -1.0),
    )"#;
    let result: Result<Chapter, _> = ron::from_str(ron);
    match &result {
        Ok(v) => println!("SetViewElement (anchor) OK: {:?}", v),
        Err(e) => println!("SetViewElement (anchor) ERR: {}", e),
    }
    assert!(
        result.is_ok(),
        "Failed to parse SetViewElement (anchor): {:?}",
        result.err()
    );
    if let Ok(Chapter::SetViewElement {
        target, duration, ..
    }) = result
    {
        assert!(matches!(target, TweenTarget::Anchor(x, y) if x == 0.0 && y == -1.0));
        assert!(duration.is_none());
    }
}

#[test]
fn test_tween_target_box_size() {
    let ron = r#"BoxSize(to: ("@current", 130.0))"#;
    let result: Result<TweenTarget, _> = ron::from_str(ron);
    match &result {
        Ok(v) => println!("TweenTarget BoxSize OK: {:?}", v),
        Err(e) => println!("TweenTarget BoxSize ERR: {}", e),
    }
    assert!(
        result.is_ok(),
        "Failed to parse TweenTarget::BoxSize: {:?}",
        result.err()
    );
}

#[test]
fn test_tween_target_box_size_with_from() {
    let ron = r#"BoxSize(from: Some((100.0, 100.0)), to: (566.0, "@current"))"#;
    let result: Result<TweenTarget, _> = ron::from_str(ron);
    match &result {
        Ok(v) => println!("TweenTarget BoxSize with from OK: {:?}", v),
        Err(e) => println!("TweenTarget BoxSize with from ERR: {}", e),
    }
    assert!(
        result.is_ok(),
        "Failed to parse TweenTarget::BoxSize with from: {:?}",
        result.err()
    );
}

#[test]
fn test_split_battle_box_chapter_with_out_cubic_easing() {
    let ron = r#"SplitBattleBox(
        source: "main",
        result: ("left_anim", "right_anim"),
        axis: Vertical,
        gap: 25.0,
        duration: 0.8,
        easing: OutCubic,
    )"#;
    let result: Result<Chapter, _> = ron::from_str(ron);
    match &result {
        Ok(v) => println!("SplitBattleBox easing OK: {:?}", v),
        Err(e) => println!("SplitBattleBox easing ERR: {}", e),
    }
    assert!(
        result.is_ok(),
        "Failed to parse SplitBattleBox with OutCubic easing: {:?}",
        result.err()
    );
}

#[test]
fn test_merge_battle_boxes_chapter_with_out_cubic_easing() {
    let ron = r#"MergeBattleBoxes(
        sources: ("left_anim", "right_anim"),
        result: "main",
        gap_policy: Expands,
        duration: 0.5,
        easing: OutCubic,
    )"#;
    let result: Result<Chapter, _> = ron::from_str(ron);
    match &result {
        Ok(v) => println!("MergeBattleBoxes easing OK: {:?}", v),
        Err(e) => println!("MergeBattleBoxes easing ERR: {}", e),
    }
    assert!(
        result.is_ok(),
        "Failed to parse MergeBattleBoxes with OutCubic easing: {:?}",
        result.err()
    );
}

#[test]
fn test_tween_target_anchor() {
    let ron = r#"Anchor(0.0, -1.0)"#;
    let result: Result<TweenTarget, _> = ron::from_str(ron);
    assert!(
        result.is_ok(),
        "Failed to parse TweenTarget::Anchor: {:?}",
        result.err()
    );
    match result.unwrap() {
        TweenTarget::Anchor(x, y) => {
            assert!((x - 0.0).abs() < f32::EPSILON);
            assert!((y - (-1.0)).abs() < f32::EPSILON);
        }
        _ => panic!("Expected Anchor variant"),
    }
}

#[test]
fn test_tween_target_box_size_no_anchor_field() {
    let ron = r#"BoxSize(to: (175.0, 130.0))"#;
    let result: Result<TweenTarget, _> = ron::from_str(ron);
    assert!(result.is_ok());
    match result.unwrap() {
        TweenTarget::BoxSize { from, to } => {
            assert!(from.is_none());
            assert!(matches!(to.0, Value::Static(v) if (v - 175.0).abs() < f32::EPSILON));
            assert!(matches!(to.1, Value::Static(v) if (v - 130.0).abs() < f32::EPSILON));
        }
        _ => panic!("Expected BoxSize variant"),
    }
}
