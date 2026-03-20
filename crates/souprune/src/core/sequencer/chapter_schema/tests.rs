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
fn test_tween_view_element_chapter() {
    let ron = r#"TweenViewElement(
        selector: LocalName("BattleBox"),
        target: BoxSize(to: ("@current", 130.0)),
        duration: 0.5,
        easing: QuadInOut,
        wait_for_completion: true,
    )"#;
    let result: Result<Chapter, _> = ron::from_str(ron);
    match &result {
        Ok(v) => println!("TweenViewElement OK: {:?}", v),
        Err(e) => println!("TweenViewElement ERR: {}", e),
    }
    assert!(
        result.is_ok(),
        "Failed to parse TweenViewElement: {:?}",
        result.err()
    );
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
