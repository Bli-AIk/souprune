//! Repeat-driven desired View tree computation tests.
//!
//! repeat 驱动的目标 View 树计算测试。

use super::*;

#[test]
fn desired_state_repeat_respects_limit_for_local_string_list() {
    let mut root = node("Item", StyleDef::default(), Vec::new());
    root.repeat = Some(RepeatDef {
        source: "names".to_string(),
        limit: Some(2),
        index_var: None,
        item_var: None,
    });
    let db = LayeredFactDatabase::new();
    let mut local = LocalState::new();
    local.set(
        "names",
        FactValue::StringList(vec![
            "one".to_string(),
            "two".to_string(),
            "three".to_string(),
        ]),
    );

    let desired = compute_desired_state(
        &asset(root),
        Vec2::new(640.0, 480.0),
        &db,
        &local,
        "",
        None,
        None,
    );

    assert_eq!(desired.roots.len(), 2);
    assert_eq!(desired.roots[0].key.full_name, "Item_0");
    assert_eq!(desired.roots[1].key.full_name, "Item_1");
}

#[test]
fn desired_state_repeat_binds_int_item_var_for_transform() {
    let mut root = node("Value", StyleDef::default(), Vec::new());
    root.repeat = Some(RepeatDef {
        source: "values".to_string(),
        limit: None,
        index_var: None,
        item_var: Some("value".to_string()),
    });
    root.transform = Some(SerializableTransform {
        translation: Some((
            Value::Expr("@value".to_string()),
            Value::Static(0.0),
            Value::Static(0.0),
        )),
        rotation: None,
        scale: None,
    });
    let db = LayeredFactDatabase::new();
    let mut local = LocalState::new();
    local.set("values", FactValue::IntList(vec![4, 8]));

    let desired = compute_desired_state(
        &asset(root),
        Vec2::new(640.0, 480.0),
        &db,
        &local,
        "",
        None,
        None,
    );

    let xs: Vec<f32> = desired
        .roots
        .iter()
        .map(|root| root.transform.translation.x)
        .collect();
    assert_eq!(xs, vec![4.0, 8.0]);
}

#[test]
fn desired_state_repeat_count_updates_sibling_layout_rect() {
    let mut repeated = node(
        "Item",
        StyleDef {
            width: Some(SerializableVal::Px(50.0)),
            height: Some(SerializableVal::Px(20.0)),
            ..Default::default()
        },
        Vec::new(),
    );
    repeated.repeat = Some(RepeatDef {
        source: "items".to_string(),
        limit: None,
        index_var: None,
        item_var: None,
    });
    let sibling = node(
        "Tail",
        StyleDef {
            width: Some(SerializableVal::Px(50.0)),
            height: Some(SerializableVal::Px(20.0)),
            ..Default::default()
        },
        Vec::new(),
    );
    let root = node(
        "Root",
        StyleDef {
            width: Some(SerializableVal::Px(300.0)),
            height: Some(SerializableVal::Px(100.0)),
            flex_direction: Some(UiFlexDirection::Row),
            justify_content: Some(SerializableJustifyContent::Center),
            ..Default::default()
        },
        vec![repeated, sibling],
    );
    let db = LayeredFactDatabase::new();
    let mut local = LocalState::new();
    local.set("items", FactValue::StringList(vec!["one".to_string()]));
    let one_item = compute_desired_state(
        &asset(root.clone()),
        Vec2::new(300.0, 100.0),
        &db,
        &local,
        "",
        None,
        None,
    );

    local.set(
        "items",
        FactValue::StringList(vec![
            "one".to_string(),
            "two".to_string(),
            "three".to_string(),
        ]),
    );
    let three_items = compute_desired_state(
        &asset(root),
        Vec2::new(300.0, 100.0),
        &db,
        &local,
        "",
        None,
        None,
    );

    let one_tail = one_item.roots[0].children.last().expect("tail child");
    let three_tail = three_items.roots[0].children.last().expect("tail child");
    assert_eq!(one_tail.name, "Tail");
    assert_eq!(three_tail.name, "Tail");
    assert_eq!(one_tail.layout_rect.expect("tail rect").x, 150.0);
    assert_eq!(three_tail.layout_rect.expect("tail rect").x, 200.0);
}

#[test]
fn desired_state_fact_text_length_updates_fit_sibling_layout_rect() {
    let mut label = node(
        "Label",
        StyleDef {
            sizing: Some(crate::core::view::layout::ViewSizingDef::Fit),
            ..Default::default()
        },
        Vec::new(),
    );
    label.texts.push(crate::core::view::layout::TextDef {
        id: "label_text".to_string(),
        content: Some("{$label}".to_string()),
        font: "default".to_string(),
        align: None,
        anchor: None,
        world_scale: (Value::Static(1.0), Value::Static(1.0)),
        color: (
            Value::Static(1.0),
            Value::Static(1.0),
            Value::Static(1.0),
            Value::Static(1.0),
        ),
        transform: SerializableTransform::default(),
        line_height: None,
        char_spacing: None,
        word_spacing: None,
        text_style: None,
        conditional_style: None,
        visible_when: None,
    });
    let marker = node(
        "Marker",
        StyleDef {
            width: Some(SerializableVal::Px(20.0)),
            height: Some(SerializableVal::Px(20.0)),
            ..Default::default()
        },
        Vec::new(),
    );
    let root = node(
        "Root",
        StyleDef {
            width: Some(SerializableVal::Px(300.0)),
            height: Some(SerializableVal::Px(100.0)),
            flex_direction: Some(UiFlexDirection::Row),
            ..Default::default()
        },
        vec![label, marker],
    );
    let db = LayeredFactDatabase::new();
    let mut local = LocalState::new();
    local.set("label", FactValue::String("A".to_string()));
    let short = compute_desired_state(
        &asset(root.clone()),
        Vec2::new(300.0, 100.0),
        &db,
        &local,
        "",
        None,
        None,
    );

    local.set("label", FactValue::String("ABCDEFG".to_string()));
    let long = compute_desired_state(
        &asset(root),
        Vec2::new(300.0, 100.0),
        &db,
        &local,
        "",
        None,
        None,
    );

    let short_marker = short.roots[0].children.last().expect("marker child");
    let long_marker = long.roots[0].children.last().expect("marker child");
    assert!(
        long_marker.layout_rect.expect("long marker rect").x
            > short_marker.layout_rect.expect("short marker rect").x
    );
}

#[test]
fn desired_state_repeat_item_text_length_updates_fit_sibling_layout_rect() {
    let mut label = node(
        "Label",
        StyleDef {
            sizing: Some(crate::core::view::layout::ViewSizingDef::Fit),
            ..Default::default()
        },
        Vec::new(),
    );
    label.repeat = Some(RepeatDef {
        source: "items".to_string(),
        limit: Some(1),
        index_var: None,
        item_var: None,
    });
    label.texts.push(crate::core::view::layout::TextDef {
        id: "label_text".to_string(),
        content: Some("@item".to_string()),
        font: "default".to_string(),
        align: None,
        anchor: None,
        world_scale: (Value::Static(1.0), Value::Static(1.0)),
        color: (
            Value::Static(1.0),
            Value::Static(1.0),
            Value::Static(1.0),
            Value::Static(1.0),
        ),
        transform: SerializableTransform::default(),
        line_height: None,
        char_spacing: None,
        word_spacing: None,
        text_style: None,
        conditional_style: None,
        visible_when: None,
    });
    let marker = node(
        "Marker",
        StyleDef {
            width: Some(SerializableVal::Px(20.0)),
            height: Some(SerializableVal::Px(20.0)),
            ..Default::default()
        },
        Vec::new(),
    );
    let root = node(
        "Root",
        StyleDef {
            width: Some(SerializableVal::Px(300.0)),
            height: Some(SerializableVal::Px(100.0)),
            flex_direction: Some(UiFlexDirection::Row),
            ..Default::default()
        },
        vec![label, marker],
    );
    let db = LayeredFactDatabase::new();
    let mut local = LocalState::new();
    local.set("items", FactValue::StringList(vec!["A".to_string()]));
    let short = compute_desired_state(
        &asset(root.clone()),
        Vec2::new(300.0, 100.0),
        &db,
        &local,
        "",
        None,
        None,
    );

    local.set(
        "items",
        FactValue::StringList(vec!["LONG_LABEL".to_string()]),
    );
    let long = compute_desired_state(
        &asset(root),
        Vec2::new(300.0, 100.0),
        &db,
        &local,
        "",
        None,
        None,
    );

    let short_marker = short.roots[0].children.last().expect("marker child");
    let long_marker = long.roots[0].children.last().expect("marker child");
    assert!(
        long_marker.layout_rect.expect("long marker rect").x
            > short_marker.layout_rect.expect("short marker rect").x
    );
}

#[test]
fn desired_state_repeat_default_item_var_resolves_text_content() {
    let mut label = node("Label", StyleDef::default(), Vec::new());
    label.repeat = Some(RepeatDef {
        source: "items".to_string(),
        limit: Some(1),
        index_var: None,
        item_var: None,
    });
    label.texts.push(crate::core::view::layout::TextDef {
        id: "label_text".to_string(),
        content: Some("@item".to_string()),
        font: "default".to_string(),
        align: None,
        anchor: None,
        world_scale: (Value::Static(1.0), Value::Static(1.0)),
        color: (
            Value::Static(1.0),
            Value::Static(1.0),
            Value::Static(1.0),
            Value::Static(1.0),
        ),
        transform: SerializableTransform::default(),
        line_height: None,
        char_spacing: None,
        word_spacing: None,
        text_style: None,
        conditional_style: None,
        visible_when: None,
    });
    let db = LayeredFactDatabase::new();
    let mut local = LocalState::new();
    local.set("items", FactValue::StringList(vec!["Alpha".to_string()]));

    let desired = compute_desired_state(
        &asset(label),
        Vec2::new(300.0, 100.0),
        &db,
        &local,
        "",
        None,
        None,
    );

    assert_eq!(desired.roots[0].texts[0].content, "Alpha");
}
