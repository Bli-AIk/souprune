//! Measurement and spatial example tests for Taffy View layout.
//!
//! Taffy View 布局的测量与空间示例测试。

use super::*;

#[test]
fn fit_view_box_leaf_uses_view_box_measurement() {
    let mut measured = node(
        "measured",
        StyleDef {
            sizing: Some(ViewSizingDef::Fit),
            ..Default::default()
        },
        Vec::new(),
    );
    measured.view_box = Some(crate::core::view::layout::ViewBoxLogicDef {
        width: 180.0,
        height: 64.0,
        border_width: 0.0,
        offset: (
            crate::core::sequencer::chapter_schema::Value::Static(0.0),
            crate::core::sequencer::chapter_schema::Value::Static(0.0),
            crate::core::sequencer::chapter_schema::Value::Static(0.0),
        ),
        fill_shader: None,
        structure_file: None,
        fill_color: None,
    });
    let root = node(
        "root",
        StyleDef {
            width: Some(SerializableVal::Px(640.0)),
            height: Some(SerializableVal::Px(480.0)),
            ..Default::default()
        },
        vec![measured],
    );

    let slots = compute_taffy_layout(&asset(root), Vec2::new(640.0, 480.0)).unwrap();

    let measured = slots.get("0:root/0:measured").expect("measured slot");
    assert_close(measured.width, 180.0);
    assert_close(measured.height, 64.0);
}

#[test]
fn fit_view_box_adds_padding_and_border_to_measured_content() {
    let mut measured = node(
        "measured",
        StyleDef {
            sizing: Some(ViewSizingDef::Fit),
            padding: Some(crate::core::view::layout::SerializableRect {
                left: SerializableVal::Px(10.0),
                right: SerializableVal::Px(10.0),
                top: SerializableVal::Px(3.0),
                bottom: SerializableVal::Px(7.0),
            }),
            border: Some(crate::core::view::layout::SerializableRect {
                left: SerializableVal::Px(1.0),
                right: SerializableVal::Px(2.0),
                top: SerializableVal::Px(4.0),
                bottom: SerializableVal::Px(6.0),
            }),
            ..Default::default()
        },
        Vec::new(),
    );
    measured.view_box = Some(crate::core::view::layout::ViewBoxLogicDef {
        width: 100.0,
        height: 50.0,
        border_width: 0.0,
        offset: (
            crate::core::sequencer::chapter_schema::Value::Static(0.0),
            crate::core::sequencer::chapter_schema::Value::Static(0.0),
            crate::core::sequencer::chapter_schema::Value::Static(0.0),
        ),
        fill_shader: None,
        structure_file: None,
        fill_color: None,
    });
    let root = node(
        "root",
        StyleDef {
            width: Some(SerializableVal::Px(640.0)),
            height: Some(SerializableVal::Px(480.0)),
            ..Default::default()
        },
        vec![measured],
    );

    let slots = compute_taffy_layout(&asset(root), Vec2::new(640.0, 480.0)).unwrap();

    let measured = slots.get("0:root/0:measured").expect("measured slot");
    assert_close(measured.width, 123.0);
    assert_close(measured.height, 70.0);
}

#[test]
fn fit_view_box_with_children_uses_view_box_measurement() {
    let mut measured = node(
        "measured",
        StyleDef {
            sizing: Some(ViewSizingDef::Fit),
            flex_direction: Some(UiFlexDirection::Row),
            ..Default::default()
        },
        vec![node(
            "child",
            StyleDef {
                width: Some(SerializableVal::Px(25.0)),
                height: Some(SerializableVal::Px(12.0)),
                ..Default::default()
            },
            Vec::new(),
        )],
    );
    measured.view_box = Some(crate::core::view::layout::ViewBoxLogicDef {
        width: 180.0,
        height: 64.0,
        border_width: 0.0,
        offset: (
            crate::core::sequencer::chapter_schema::Value::Static(0.0),
            crate::core::sequencer::chapter_schema::Value::Static(0.0),
            crate::core::sequencer::chapter_schema::Value::Static(0.0),
        ),
        fill_shader: None,
        structure_file: None,
        fill_color: None,
    });
    let root = node(
        "root",
        StyleDef {
            width: Some(SerializableVal::Px(640.0)),
            height: Some(SerializableVal::Px(480.0)),
            ..Default::default()
        },
        vec![measured],
    );

    let slots = compute_taffy_layout(&asset(root), Vec2::new(640.0, 480.0)).unwrap();

    let measured = slots.get("0:root/0:measured").expect("measured slot");
    assert_close(measured.width, 180.0);
    assert_close(measured.height, 64.0);
}

#[test]
fn fit_text_leaf_uses_conservative_text_measurement() {
    let mut measured = node(
        "text",
        StyleDef {
            sizing: Some(ViewSizingDef::Fit),
            ..Default::default()
        },
        Vec::new(),
    );
    measured.texts.push(crate::core::view::layout::TextDef {
        id: "label".to_string(),
        content: Some("AB\nC".to_string()),
        font: "DTM-Mono".to_string(),
        align: None,
        anchor: None,
        world_scale: (
            crate::core::sequencer::chapter_schema::Value::Static(1.0),
            crate::core::sequencer::chapter_schema::Value::Static(1.0),
        ),
        color: (
            crate::core::sequencer::chapter_schema::Value::Static(1.0),
            crate::core::sequencer::chapter_schema::Value::Static(1.0),
            crate::core::sequencer::chapter_schema::Value::Static(1.0),
            crate::core::sequencer::chapter_schema::Value::Static(1.0),
        ),
        transform: SerializableTransform::default(),
        line_height: Some(1.0),
        char_spacing: Some(0.0),
        word_spacing: Some(0.0),
        text_style: None,
        conditional_style: None,
        visible_when: None,
    });
    let root = node(
        "root",
        StyleDef {
            width: Some(SerializableVal::Px(640.0)),
            height: Some(SerializableVal::Px(480.0)),
            ..Default::default()
        },
        vec![measured],
    );

    let slots = compute_taffy_layout(&asset(root), Vec2::new(640.0, 480.0)).unwrap();

    let measured = slots.get("0:root/0:text").expect("text slot");
    assert_close(measured.width, 1.0);
    assert_close(measured.height, 2.0);
}

#[test]
fn fit_text_measurement_preserves_negative_spacing() {
    let mut measured = node(
        "text",
        StyleDef {
            sizing: Some(ViewSizingDef::Fit),
            ..Default::default()
        },
        Vec::new(),
    );
    measured.texts.push(crate::core::view::layout::TextDef {
        id: "label".to_string(),
        content: Some("A B".to_string()),
        font: "DTM-Mono".to_string(),
        align: None,
        anchor: None,
        world_scale: (
            crate::core::sequencer::chapter_schema::Value::Static(128.0),
            crate::core::sequencer::chapter_schema::Value::Static(128.0),
        ),
        color: (
            crate::core::sequencer::chapter_schema::Value::Static(1.0),
            crate::core::sequencer::chapter_schema::Value::Static(1.0),
            crate::core::sequencer::chapter_schema::Value::Static(1.0),
            crate::core::sequencer::chapter_schema::Value::Static(1.0),
        ),
        transform: SerializableTransform::default(),
        line_height: Some(1.0),
        char_spacing: Some(-4.0),
        word_spacing: Some(-8.0),
        text_style: None,
        conditional_style: None,
        visible_when: None,
    });
    let root = node(
        "root",
        StyleDef {
            width: Some(SerializableVal::Px(640.0)),
            height: Some(SerializableVal::Px(480.0)),
            ..Default::default()
        },
        vec![measured],
    );

    let slots = compute_taffy_layout(&asset(root), Vec2::new(640.0, 480.0)).unwrap();

    let measured = slots.get("0:root/0:text").expect("text slot");
    assert_close(measured.width, 172.0);
}

#[test]
fn fit_text_measurement_applies_spacing_after_every_glyph_and_keeps_trailing_line() {
    let mut measured = node(
        "text",
        StyleDef {
            sizing: Some(ViewSizingDef::Fit),
            ..Default::default()
        },
        Vec::new(),
    );
    measured.texts.push(crate::core::view::layout::TextDef {
        id: "label".to_string(),
        content: Some("AB\n".to_string()),
        font: "DTM-Mono".to_string(),
        align: None,
        anchor: None,
        world_scale: (
            crate::core::sequencer::chapter_schema::Value::Static(128.0),
            crate::core::sequencer::chapter_schema::Value::Static(128.0),
        ),
        color: (
            crate::core::sequencer::chapter_schema::Value::Static(1.0),
            crate::core::sequencer::chapter_schema::Value::Static(1.0),
            crate::core::sequencer::chapter_schema::Value::Static(1.0),
            crate::core::sequencer::chapter_schema::Value::Static(1.0),
        ),
        transform: SerializableTransform::default(),
        line_height: Some(1.0),
        char_spacing: Some(16.0),
        word_spacing: Some(0.0),
        text_style: None,
        conditional_style: None,
        visible_when: None,
    });
    let root = node(
        "root",
        StyleDef {
            width: Some(SerializableVal::Px(640.0)),
            height: Some(SerializableVal::Px(480.0)),
            ..Default::default()
        },
        vec![measured],
    );

    let slots = compute_taffy_layout(&asset(root), Vec2::new(640.0, 480.0)).unwrap();

    let measured = slots.get("0:root/0:text").expect("text slot");
    assert_close(measured.width, 160.0);
    assert_close(measured.height, 256.0);
}

#[test]
fn fixed_sizing_sets_dimensions_without_width_height_fields() {
    let root = node(
        "root",
        StyleDef {
            sizing: Some(ViewSizingDef::Fixed {
                width: SerializableVal::Px(320.0),
                height: SerializableVal::Px(120.0),
            }),
            ..Default::default()
        },
        Vec::new(),
    );

    let slots = compute_taffy_layout(&asset(root), Vec2::new(640.0, 480.0)).unwrap();

    let root = slots.get("0:root").expect("root slot");
    assert_close(root.width, 320.0);
    assert_close(root.height, 120.0);
}

#[test]
fn fill_sizing_takes_remaining_main_axis_space() {
    let fixed = node(
        "fixed",
        StyleDef {
            sizing: Some(ViewSizingDef::Fixed {
                width: SerializableVal::Px(100.0),
                height: SerializableVal::Px(40.0),
            }),
            ..Default::default()
        },
        Vec::new(),
    );
    let fill = node(
        "fill",
        StyleDef {
            sizing: Some(ViewSizingDef::Axes {
                width: ViewSizeAxisDef::Fill,
                height: ViewSizeAxisDef::Fixed(SerializableVal::Px(40.0)),
            }),
            ..Default::default()
        },
        Vec::new(),
    );
    let root = node(
        "root",
        StyleDef {
            width: Some(SerializableVal::Px(640.0)),
            height: Some(SerializableVal::Px(480.0)),
            flex_direction: Some(UiFlexDirection::Row),
            gap: Some(StyleGap {
                row: SerializableVal::Px(0.0),
                column: SerializableVal::Px(20.0),
            }),
            ..Default::default()
        },
        vec![fixed, fill],
    );

    let slots = compute_taffy_layout(&asset(root), Vec2::new(640.0, 480.0)).unwrap();

    let fill = slots.get("0:root/1:fill").expect("fill slot");
    assert_close(fill.width, 520.0);
}

#[test]
fn spatial_plane_view_asset_parses_and_solves() {
    let layout: ViewLayoutAsset = ron::from_str(include_str!(
        "../../../../../../examples/assets/view/spatial_plane.view.ron"
    ))
    .expect("spatial plane example should parse");

    let Some(ViewSpaceDef::World3dPlane(plane)) = &layout.space else {
        panic!("spatial plane example should declare World3dPlane space");
    };
    assert!(plane.rotation_degrees.is_some());
    assert!(matches!(
        plane.anchor,
        ViewSpatialAnchorDef::Named(ref name) if name == "SpatialAnchor"
    ));
    assert!(matches!(
        plane.orientation,
        ViewSpatialOrientationDef::Fixed
    ));
    assert!(matches!(plane.input, ViewSpatialInputDef::PlaneRay));

    let slots = compute_taffy_layout(&layout, Vec2::new(360.0, 220.0))
        .expect("spatial plane example should solve layout");

    assert!(slots.get("0:SpatialPanel").is_some());
    assert!(slots.get("0:SpatialPanel/0:SpatialRow").is_some());
    assert!(
        slots
            .get("0:SpatialPanel/0:SpatialRow/0:SpatialRowItemA")
            .is_some()
    );
    assert!(
        slots
            .get("0:SpatialPanel/1:SpatialAbsoluteMarker")
            .is_some()
    );
}
