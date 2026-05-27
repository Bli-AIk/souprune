//! Tests for View layout observer selection.
//!
//! View 布局观察器选择测试。

use super::*;
use crate::core::view::layout::{
    SerializableAlignItems, SerializableAlignSelf, SerializableDisplay, SerializableJustifyContent,
    SerializablePositionType, UiFlexDirection, ViewLayoutEdges, ViewLayoutGap,
    ViewLayoutLengthDebug, ViewLayoutSizingDebug,
};

fn sample_selection(entity: Entity, depth: usize, area: f32) -> ViewLayoutObserverSelection {
    let width = area.sqrt();
    ViewLayoutObserverSelection {
        entity,
        root_entity: Entity::from_bits(99),
        root_layout_path: "view/demo.view.ron".to_string(),
        root_namespace: "view_demo".to_string(),
        element_name: "demo::Element".to_string(),
        element_path: format!("0:Root/{depth}:Node"),
        depth,
        area,
        rect: ViewLayoutRect {
            x: 12.0,
            y: 24.0,
            width,
            height: width,
        },
        element_transform: GlobalTransform::IDENTITY,
        origin: ViewLayoutObserverOrigin::Center,
        is_hidden: false,
        clip_rect: None,
        scroll_state: None,
        debug: Some(ViewLayoutDebugMetadata {
            path: format!("0:Root/{depth}:Node"),
            name: "Node".to_string(),
            depth,
            parent_path: Some("0:Root".to_string()),
            display: SerializableDisplay::Flex,
            position_type: SerializablePositionType::Relative,
            flex_direction: UiFlexDirection::Row,
            justify_content: Some(SerializableJustifyContent::Center),
            align_items: Some(SerializableAlignItems::Center),
            align_self: Some(SerializableAlignSelf::Auto),
            margin: ViewLayoutEdges::new(1.0, 2.0, 3.0, 4.0),
            padding: ViewLayoutEdges::new(5.0, 6.0, 7.0, 8.0),
            border: ViewLayoutEdges::new(9.0, 10.0, 11.0, 12.0),
            gap: ViewLayoutGap::new(13.0, 14.0),
            overflow: None,
            sizing: ViewLayoutSizingDebug {
                width: ViewLayoutLengthDebug::Px(10.0),
                height: ViewLayoutLengthDebug::Px(10.0),
                flex_grow: 1.0,
                flex_shrink: 1.0,
                flex_basis: ViewLayoutLengthDebug::Px(0.0),
            },
        }),
        spatial_plane: None,
        spatial_hit: None,
    }
}

#[test]
fn choose_best_selection_prefers_deeper_candidate() {
    let shallow = sample_selection(Entity::from_bits(1), 1, 400.0);
    let deep = sample_selection(Entity::from_bits(2), 4, 400.0);

    let chosen = choose_best_selection([shallow, deep]).expect("selection");

    assert_eq!(chosen.entity, Entity::from_bits(2));
}

#[test]
fn selected_selection_for_mode_uses_locked_selection_when_locked() {
    let hover = sample_selection(Entity::from_bits(1), 1, 400.0);
    let locked = sample_selection(Entity::from_bits(2), 2, 300.0);

    let selected =
        selected_selection_for_mode(ViewLayoutObserverMode::Locked, Some(hover), Some(locked))
            .expect("selection");

    assert_eq!(selected.entity, Entity::from_bits(2));
}

#[test]
fn centered_origin_cursor_maps_to_local_layout_box() {
    let point = local_point_to_layout_point(
        Vec2::ZERO,
        ViewLayoutRect {
            x: 12.0,
            y: 24.0,
            width: 200.0,
            height: 100.0,
        },
        ViewLayoutObserverOrigin::Center,
        1.0,
    );

    assert_eq!(point, Vec2::new(100.0, 50.0));
}

#[test]
fn top_left_origin_cursor_uses_container_local_top_left() {
    let point = local_point_to_layout_point(
        Vec2::new(40.0, -35.0),
        ViewLayoutRect {
            x: 12.0,
            y: 24.0,
            width: 200.0,
            height: 100.0,
        },
        ViewLayoutObserverOrigin::TopLeft,
        1.0,
    );

    assert_eq!(point, Vec2::new(40.0, 35.0));
}

#[test]
fn local_layout_rect_comparison_is_relative_to_element_rect() {
    let base = ViewLayoutRect {
        x: 100.0,
        y: 40.0,
        width: 200.0,
        height: 100.0,
    };
    let clipped = ViewLayoutRect {
        x: 120.0,
        y: 50.0,
        width: 60.0,
        height: 30.0,
    };

    assert!(point_in_local_layout_rect(
        Vec2::new(40.0, 20.0),
        clipped,
        base
    ));
    assert!(!point_in_local_layout_rect(
        Vec2::new(10.0, 20.0),
        clipped,
        base
    ));
}

#[test]
fn spatial_cursor_point_scales_plane_units_to_layout_pixels() {
    let point = local_point_to_layout_point(
        Vec2::new(0.25, -0.1),
        ViewLayoutRect {
            x: 0.0,
            y: 0.0,
            width: 100.0,
            height: 80.0,
        },
        ViewLayoutObserverOrigin::Center,
        100.0,
    );

    assert_eq!(point, Vec2::new(75.0, 50.0));
}

#[test]
fn observer_origin_uses_top_left_for_pure_containers_only() {
    let container = ViewContainer;

    assert_eq!(
        observer_origin(Some(&container)),
        ViewLayoutObserverOrigin::TopLeft
    );
    assert_eq!(observer_origin(None), ViewLayoutObserverOrigin::Center);
}
