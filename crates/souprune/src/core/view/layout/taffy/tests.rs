//! Tests for pure Taffy View layout solving.
//!
//! 纯 Taffy View 布局求解测试。

use super::*;
use crate::core::view::layout::{
    SerializableAlignItems, SerializableAlignSelf, SerializableDisplay, SerializableJustifyContent,
    SerializablePositionType, SerializableTransform, SerializableVal, StyleDef, StyleGap,
    UiFlexDirection, ViewFocusPolicyDef, ViewNodeDef, ViewOverflowAxisDef, ViewOverflowDef,
    ViewScrollState, ViewSizeAxisDef, ViewSizingDef, ViewSpaceDef, ViewSpatialAnchorDef,
    ViewSpatialInputDef, ViewSpatialOrientationDef,
};

fn asset(root: ViewNodeDef) -> ViewLayoutAsset {
    ViewLayoutAsset {
        roots: vec![root],
        requires: Vec::new(),
        facts: None,
        space: None,
        coordinate_system: Default::default(),
        coordinate_space: None,
    }
}

fn node(name: &str, style: StyleDef, children: Vec<ViewNodeDef>) -> ViewNodeDef {
    ViewNodeDef {
        name: name.to_string(),
        tags: Vec::new(),
        style,
        transform: None,
        focus_policy: None,
        visible_when: None,
        background_color: None,
        border_color: None,
        image: None,
        sprite: None,
        state_sprite: None,
        texts: Vec::new(),
        view_box: None,
        children,
        repeat: None,
    }
}

fn find_node<'a>(nodes: &'a [ViewNodeDef], name: &str) -> Option<&'a ViewNodeDef> {
    nodes.iter().find_map(|node| {
        if node.name == name {
            Some(node)
        } else {
            find_node(&node.children, name)
        }
    })
}

fn assert_close(actual: f32, expected: f32) {
    assert!(
        (actual - expected).abs() < 0.01,
        "expected {actual} to be close to {expected}"
    );
}

#[test]
fn row_flex_centers_children_with_gap() {
    let child_style = StyleDef {
        width: Some(SerializableVal::Px(100.0)),
        height: Some(SerializableVal::Px(40.0)),
        ..Default::default()
    };
    let root = node(
        "root",
        StyleDef {
            width: Some(SerializableVal::Px(640.0)),
            height: Some(SerializableVal::Px(480.0)),
            flex_direction: Some(UiFlexDirection::Row),
            justify_content: Some(crate::core::view::layout::SerializableJustifyContent::Center),
            gap: Some(StyleGap {
                row: SerializableVal::Px(0.0),
                column: SerializableVal::Px(20.0),
            }),
            ..Default::default()
        },
        vec![
            node("first", child_style.clone(), Vec::new()),
            node("second", child_style, Vec::new()),
        ],
    );

    let slots = compute_taffy_layout(&asset(root), Vec2::new(640.0, 480.0)).unwrap();

    assert_close(slots.get("0:root/0:first").expect("first slot").x, 210.0);
    assert_close(slots.get("0:root/1:second").expect("second slot").x, 330.0);
}

#[test]
fn flex_layout_exposes_debug_metadata() {
    let child_style = StyleDef {
        width: Some(SerializableVal::Px(48.0)),
        height: Some(SerializableVal::Px(24.0)),
        align_self: Some(SerializableAlignSelf::Center),
        ..Default::default()
    };
    let root = node(
        "root",
        StyleDef {
            width: Some(SerializableVal::Px(320.0)),
            height: Some(SerializableVal::Px(160.0)),
            flex_direction: Some(UiFlexDirection::Row),
            justify_content: Some(SerializableJustifyContent::SpaceBetween),
            align_items: Some(SerializableAlignItems::Center),
            padding: Some(SerializableRect {
                left: SerializableVal::Px(12.0),
                right: SerializableVal::Px(14.0),
                top: SerializableVal::Px(16.0),
                bottom: SerializableVal::Px(18.0),
            }),
            border: Some(SerializableRect {
                left: SerializableVal::Px(1.0),
                right: SerializableVal::Px(2.0),
                top: SerializableVal::Px(3.0),
                bottom: SerializableVal::Px(4.0),
            }),
            gap: Some(StyleGap {
                row: SerializableVal::Px(6.0),
                column: SerializableVal::Px(8.0),
            }),
            ..Default::default()
        },
        vec![node("child", child_style, Vec::new())],
    );

    let slots = compute_taffy_layout(&asset(root), Vec2::new(320.0, 160.0)).unwrap();
    let root_debug = slots.debug_metadata("0:root").expect("root metadata");
    let child_debug = slots
        .debug_metadata("0:root/0:child")
        .expect("child metadata");

    assert_eq!(root_debug.display, SerializableDisplay::Flex);
    assert_eq!(root_debug.position_type, SerializablePositionType::Relative);
    assert_eq!(root_debug.flex_direction, UiFlexDirection::Row);
    assert_eq!(
        root_debug.justify_content,
        Some(SerializableJustifyContent::SpaceBetween)
    );
    assert_eq!(root_debug.align_items, Some(SerializableAlignItems::Center));
    assert_close(root_debug.padding.left, 12.0);
    assert_close(root_debug.padding.bottom, 18.0);
    assert_close(root_debug.border.top, 3.0);
    assert_close(root_debug.gap.column, 8.0);
    assert_eq!(root_debug.sizing.width, ViewLayoutLengthDebug::Px(320.0));
    assert_eq!(root_debug.sizing.height, ViewLayoutLengthDebug::Px(160.0));
    assert_eq!(child_debug.depth, 1);
    assert_eq!(child_debug.parent_path.as_deref(), Some("0:root"));
    assert_eq!(child_debug.align_self, Some(SerializableAlignSelf::Center));
}

#[test]
fn absolute_child_uses_parent_inset_and_does_not_participate_in_sibling_flex() {
    let sized_child = StyleDef {
        width: Some(SerializableVal::Px(100.0)),
        height: Some(SerializableVal::Px(40.0)),
        ..Default::default()
    };
    let absolute_child = StyleDef {
        width: Some(SerializableVal::Px(50.0)),
        height: Some(SerializableVal::Px(30.0)),
        left: Some(SerializableVal::Px(25.0)),
        top: Some(SerializableVal::Px(35.0)),
        position_type: Some(SerializablePositionType::Absolute),
        ..Default::default()
    };
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
        vec![
            node("first", sized_child.clone(), Vec::new()),
            node("absolute", absolute_child, Vec::new()),
            node("second", sized_child, Vec::new()),
        ],
    );

    let slots = compute_taffy_layout(&asset(root), Vec2::new(640.0, 480.0)).unwrap();

    let absolute = slots.get("0:root/1:absolute").expect("absolute slot");
    assert_close(absolute.x, 25.0);
    assert_close(absolute.y, 35.0);
    assert_close(slots.get("0:root/0:first").expect("first slot").x, 0.0);
    assert_close(slots.get("0:root/2:second").expect("second slot").x, 120.0);
}

#[test]
fn display_none_node_is_absent_from_slots_and_flex_flow() {
    let visible_child = StyleDef {
        width: Some(SerializableVal::Px(100.0)),
        height: Some(SerializableVal::Px(40.0)),
        ..Default::default()
    };
    let hidden_child = StyleDef {
        width: Some(SerializableVal::Px(100.0)),
        height: Some(SerializableVal::Px(40.0)),
        display: Some(SerializableDisplay::None),
        ..Default::default()
    };
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
        vec![
            node("first", visible_child.clone(), Vec::new()),
            node("hidden", hidden_child, Vec::new()),
            node("second", visible_child, Vec::new()),
        ],
    );

    let slots = compute_taffy_layout(&asset(root), Vec2::new(640.0, 480.0)).unwrap();

    assert!(slots.get("0:root/1:hidden").is_none());
    assert_close(slots.get("0:root/0:first").expect("first slot").x, 0.0);
    assert_close(slots.get("0:root/2:second").expect("second slot").x, 120.0);
}

#[test]
fn overflow_hidden_and_scroll_create_slot_metadata() {
    let root = node(
        "root",
        StyleDef {
            width: Some(SerializableVal::Px(640.0)),
            height: Some(SerializableVal::Px(480.0)),
            flex_direction: Some(UiFlexDirection::Row),
            ..Default::default()
        },
        vec![
            node(
                "hidden",
                StyleDef {
                    width: Some(SerializableVal::Px(160.0)),
                    height: Some(SerializableVal::Px(80.0)),
                    overflow: Some(ViewOverflowDef::Hidden),
                    ..Default::default()
                },
                Vec::new(),
            ),
            node(
                "scroll",
                StyleDef {
                    width: Some(SerializableVal::Px(120.0)),
                    height: Some(SerializableVal::Px(60.0)),
                    overflow: Some(ViewOverflowDef::Axes {
                        horizontal: ViewOverflowAxisDef::Visible,
                        vertical: ViewOverflowAxisDef::Scroll,
                    }),
                    ..Default::default()
                },
                Vec::new(),
            ),
        ],
    );

    let slots = compute_taffy_layout(&asset(root), Vec2::new(640.0, 480.0)).unwrap();

    let hidden = slots.get("0:root/0:hidden").expect("hidden slot");
    let hidden_clip = slots
        .clip_rect("0:root/0:hidden")
        .expect("hidden clip rect");
    assert_close(hidden_clip.x, hidden.x);
    assert_close(hidden_clip.y, hidden.y);
    assert_close(hidden_clip.width, 160.0);
    assert_close(hidden_clip.height, 80.0);
    assert!(slots.scroll_state("0:root/0:hidden").is_none());

    let scroll = slots.get("0:root/1:scroll").expect("scroll slot");
    let scroll_clip = slots
        .clip_rect("0:root/1:scroll")
        .expect("scroll clip rect");
    assert_close(scroll_clip.x, scroll.x);
    assert_close(scroll_clip.y, scroll.y);
    assert_close(scroll_clip.width, 120.0);
    assert_close(scroll_clip.height, 60.0);
    assert_eq!(
        slots.scroll_state("0:root/1:scroll"),
        Some(&ViewScrollState::default())
    );
}

#[test]
fn explicit_transform_is_not_applied_to_solver_output() {
    let mut child = node(
        "child",
        StyleDef {
            width: Some(SerializableVal::Px(100.0)),
            height: Some(SerializableVal::Px(40.0)),
            ..Default::default()
        },
        Vec::new(),
    );
    child.transform = Some(SerializableTransform {
        translation: Some((
            crate::core::sequencer::chapter_schema::Value::Static(1000.0),
            crate::core::sequencer::chapter_schema::Value::Static(2000.0),
            crate::core::sequencer::chapter_schema::Value::Static(0.0),
        )),
        rotation: None,
        scale: None,
    });
    let root = node(
        "root",
        StyleDef {
            width: Some(SerializableVal::Px(640.0)),
            height: Some(SerializableVal::Px(480.0)),
            ..Default::default()
        },
        vec![child],
    );

    let slots = compute_taffy_layout(&asset(root), Vec2::new(640.0, 480.0)).unwrap();

    let child = slots.get("0:root/0:child").expect("child slot");
    assert_close(child.x, 0.0);
    assert_close(child.y, 0.0);
}

#[test]
fn sibling_index_keeps_duplicate_names_distinct() {
    let child_style = StyleDef {
        width: Some(SerializableVal::Px(100.0)),
        height: Some(SerializableVal::Px(40.0)),
        ..Default::default()
    };
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
        vec![
            node("dup", child_style.clone(), Vec::new()),
            node("dup", child_style, Vec::new()),
        ],
    );

    let slots = compute_taffy_layout(&asset(root), Vec2::new(640.0, 480.0)).unwrap();

    assert_close(slots.get("0:root/0:dup").expect("first duplicate").x, 0.0);
    assert_close(
        slots.get("0:root/1:dup").expect("second duplicate").x,
        120.0,
    );
}

#[test]
fn manual_acceptance_view_asset_parses_and_solves() {
    let mut asset: ViewLayoutAsset = ron::from_str(include_str!(
        "../../../../../examples/assets/view/taffy_minimal.view.ron"
    ))
    .expect("manual acceptance view should parse");
    asset.apply_coordinate_system();

    let slots = compute_taffy_layout(&asset, Vec2::new(640.0, 480.0)).unwrap();

    let centered = slots
        .get("0:TaffyMinimalRoot/0:CenteredElement")
        .expect("centered element slot");
    assert_close(centered.x, 240.0);
    assert_close(centered.width, 160.0);
    let button_row = slots
        .get("0:TaffyMinimalRoot/1:ButtonRow")
        .expect("button row slot");
    assert!(button_row.width > 360.0);
    let fit_probe = slots
        .get("0:TaffyMinimalRoot/4:MeasuredViewBox")
        .expect("fit probe slot");
    assert_close(fit_probe.width, 134.0);
    assert_close(fit_probe.height, 54.0);
    assert!(
        slots
            .get("0:TaffyMinimalRoot/3:HiddenDisplayNoneProbe")
            .is_none()
    );
    let hidden_leaf = slots
        .iter()
        .find(|slot| slot.name == "Stage4HiddenLeafPanel")
        .expect("stage 4 hidden leaf panel slot");
    assert!(slots.clip_rect(&hidden_leaf.path).is_some());
    assert!(slots.scroll_state(&hidden_leaf.path).is_none());
    let scroll_viewport = slots
        .iter()
        .find(|slot| slot.name == "Stage4ScrollViewport")
        .expect("stage 4 scroll viewport slot");
    assert!(slots.clip_rect(&scroll_viewport.path).is_some());
    assert_eq!(
        slots.scroll_state(&scroll_viewport.path),
        Some(&ViewScrollState::default())
    );
    let stage4_region =
        find_node(&asset.roots, "Stage4AcceptanceRegion").expect("stage 4 acceptance region node");
    assert!(matches!(
        stage4_region.focus_policy,
        Some(ViewFocusPolicyDef::Scope)
    ));
}

mod measurement;
