//! Tests for desired View tree computation.
//!
//! 目标 View 树计算测试。

use super::*;
use crate::core::sequencer::chapter_schema::Value;
use crate::core::view::layout::{
    CoordinateSystem, RepeatDef, SerializableJustifyContent, SerializableTransform,
    SerializableVal, SpriteDef, StyleDef, UiFlexDirection, ViewBoxLogicDef, ViewCameraTargetDef,
    ViewSpaceDef, ViewWorld3dPlaneDef,
};
use crate::core::visual::Visual;
use bevy::prelude::Vec3;
use bevy_fact_rule_event::{FactValue, LayeredFactDatabase};

fn asset(root: ViewNodeDef) -> ViewLayoutAsset {
    ViewLayoutAsset {
        roots: vec![root],
        requires: Vec::new(),
        facts: None,
        space: None,
        coordinate_system: CoordinateSystem::Standard,
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

#[test]
fn desired_state_keeps_taffy_layout_offset() {
    let child = node(
        "Child",
        StyleDef {
            width: Some(SerializableVal::Px(100.0)),
            height: Some(SerializableVal::Px(40.0)),
            ..Default::default()
        },
        Vec::new(),
    );
    let root = node(
        "Root",
        StyleDef {
            width: Some(SerializableVal::Px(640.0)),
            height: Some(SerializableVal::Px(480.0)),
            flex_direction: Some(UiFlexDirection::Row),
            justify_content: Some(SerializableJustifyContent::Center),
            ..Default::default()
        },
        vec![child],
    );
    let db = LayeredFactDatabase::new();
    let local = LocalState::new();

    let desired = compute_desired_state(
        &asset(root),
        Vec2::new(640.0, 480.0),
        &db,
        &local,
        "",
        None,
        None,
    );

    assert_eq!(desired.roots[0].children[0].transform.translation.x, 270.0);
    let rect = desired.roots[0].children[0]
        .layout_rect
        .expect("layout rect should be stored");
    assert_eq!(rect.x, 270.0);
    assert_eq!(rect.y, 0.0);
    assert_eq!(rect.width, 100.0);
    assert_eq!(rect.height, 40.0);
}

#[test]
fn desired_state_maps_spatial_layout_offset_to_plane_units() {
    let child = node(
        "Child",
        StyleDef {
            width: Some(SerializableVal::Px(100.0)),
            height: Some(SerializableVal::Px(40.0)),
            ..Default::default()
        },
        Vec::new(),
    );
    let root = node(
        "Root",
        StyleDef {
            width: Some(SerializableVal::Px(640.0)),
            height: Some(SerializableVal::Px(480.0)),
            flex_direction: Some(UiFlexDirection::Row),
            justify_content: Some(SerializableJustifyContent::Center),
            ..Default::default()
        },
        vec![child],
    );
    let mut layout = asset(root);
    layout.space = Some(ViewSpaceDef::World3dPlane(Box::new(ViewWorld3dPlaneDef {
        transform: SerializableTransform::default(),
        rotation_degrees: None,
        plane_size: (6.4, 4.8),
        pixels_per_unit: 100.0,
        camera: ViewCameraTargetDef::Main,
        anchor: Default::default(),
        orientation: Default::default(),
        depth: Default::default(),
        input: Default::default(),
    })));
    let db = LayeredFactDatabase::new();
    let local = LocalState::new();

    let desired = compute_desired_state(
        &layout,
        Vec2::new(640.0, 480.0),
        &db,
        &local,
        "",
        None,
        None,
    );

    assert_eq!(
        desired.roots[0].children[0].transform.translation,
        Vec3::new(2.7, 0.0, 0.0)
    );
}

#[test]
fn desired_state_skips_display_none_nodes() {
    let hidden = node(
        "Hidden",
        StyleDef {
            width: Some(SerializableVal::Px(100.0)),
            height: Some(SerializableVal::Px(40.0)),
            display: Some(SerializableDisplay::None),
            ..Default::default()
        },
        Vec::new(),
    );
    let root = node(
        "Root",
        StyleDef {
            width: Some(SerializableVal::Px(640.0)),
            height: Some(SerializableVal::Px(480.0)),
            ..Default::default()
        },
        vec![hidden],
    );
    let db = LayeredFactDatabase::new();
    let local = LocalState::new();

    let desired = compute_desired_state(
        &asset(root),
        Vec2::new(640.0, 480.0),
        &db,
        &local,
        "",
        None,
        None,
    );

    assert!(desired.roots[0].children.is_empty());
}

#[test]
fn desired_state_combines_layout_and_explicit_transform() {
    let mut child = node(
        "Child",
        StyleDef {
            width: Some(SerializableVal::Px(100.0)),
            height: Some(SerializableVal::Px(40.0)),
            ..Default::default()
        },
        Vec::new(),
    );
    child.transform = Some(SerializableTransform {
        translation: Some((Value::Static(5.0), Value::Static(-6.0), Value::Static(7.0))),
        rotation: None,
        scale: None,
    });
    let root = node(
        "Root",
        StyleDef {
            width: Some(SerializableVal::Px(640.0)),
            height: Some(SerializableVal::Px(480.0)),
            flex_direction: Some(UiFlexDirection::Row),
            justify_content: Some(SerializableJustifyContent::Center),
            ..Default::default()
        },
        vec![child],
    );
    let db = LayeredFactDatabase::new();
    let local = LocalState::new();

    let desired = compute_desired_state(
        &asset(root),
        Vec2::new(640.0, 480.0),
        &db,
        &local,
        "",
        None,
        None,
    );

    assert_eq!(
        desired.roots[0].children[0].transform.translation,
        Vec3::new(275.0, -6.0, 7.0)
    );
}

#[test]
fn desired_state_treats_taffy_child_slots_as_parent_local() {
    let mut leaf = node(
        "Leaf",
        StyleDef {
            width: Some(SerializableVal::Px(40.0)),
            height: Some(SerializableVal::Px(20.0)),
            ..Default::default()
        },
        Vec::new(),
    );
    leaf.view_box = Some(ViewBoxLogicDef {
        width: 40.0,
        height: 20.0,
        border_width: 0.0,
        offset: (
            Value::Static(20.0),
            Value::Static(-10.0),
            Value::Static(0.0),
        ),
        fill_shader: None,
        structure_file: None,
        fill_color: None,
    });

    let mut row = node(
        "Row",
        StyleDef {
            width: Some(SerializableVal::Px(160.0)),
            height: Some(SerializableVal::Px(80.0)),
            padding: Some(crate::core::view::layout::SerializableRect {
                left: SerializableVal::Px(10.0),
                right: SerializableVal::Px(0.0),
                top: SerializableVal::Px(20.0),
                bottom: SerializableVal::Px(0.0),
            }),
            ..Default::default()
        },
        vec![leaf],
    );
    row.view_box = Some(ViewBoxLogicDef {
        width: 160.0,
        height: 80.0,
        border_width: 0.0,
        offset: (
            Value::Static(80.0),
            Value::Static(-40.0),
            Value::Static(0.0),
        ),
        fill_shader: None,
        structure_file: None,
        fill_color: None,
    });

    let root = node(
        "Root",
        StyleDef {
            width: Some(SerializableVal::Px(240.0)),
            height: Some(SerializableVal::Px(120.0)),
            padding: Some(crate::core::view::layout::SerializableRect {
                left: SerializableVal::Px(30.0),
                right: SerializableVal::Px(0.0),
                top: SerializableVal::Px(40.0),
                bottom: SerializableVal::Px(0.0),
            }),
            ..Default::default()
        },
        vec![row],
    );

    let db = LayeredFactDatabase::new();
    let local = LocalState::new();

    let desired = compute_desired_state(
        &asset(root),
        Vec2::new(960.0, 540.0),
        &db,
        &local,
        "",
        None,
        None,
    );

    let row = &desired.roots[0].children[0];
    let leaf = &row.children[0];

    assert_eq!(row.transform.translation, Vec3::new(110.0, -80.0, 0.0));
    assert_eq!(leaf.transform.translation, Vec3::new(-50.0, 10.0, 0.0));
}

#[test]
fn desired_state_keeps_manual_sprite_child_transform_relative_to_view_box_center() {
    let mut cursor = node("Cursor", StyleDef::default(), Vec::new());
    cursor.sprite = Some(SpriteDef {
        visual: Visual("common/view/heartsmall".to_string()),
        initial_state: None,
        color: None,
        flip_x: false,
        flip_y: false,
        transform: Some(SerializableTransform {
            translation: Some((
                Value::Static(-76.0),
                Value::Static(24.0),
                Value::Static(6.0),
            )),
            rotation: None,
            scale: None,
        }),
        pivot: None,
        frame_duration: None,
        visible_when: None,
        material: None,
    });

    let mut parent = node("ParentBox", StyleDef::default(), vec![cursor]);
    parent.view_box = Some(ViewBoxLogicDef {
        width: 167.0,
        height: 175.0,
        border_width: 0.0,
        offset: (Value::Static(0.0), Value::Static(0.0), Value::Static(0.0)),
        fill_shader: None,
        structure_file: None,
        fill_color: None,
    });
    let db = LayeredFactDatabase::new();
    let local = LocalState::new();

    let desired = compute_desired_state(
        &asset(parent),
        Vec2::new(640.0, 480.0),
        &db,
        &local,
        "",
        None,
        None,
    );

    assert_eq!(
        desired.roots[0].children[0].transform.translation,
        Vec3::new(-76.0, 24.0, 6.0)
    );
}

#[test]
fn desired_state_places_observer_demo_visuals_inside_surface() {
    let asset: ViewLayoutAsset = ron::from_str(include_str!(
        "../../../../../examples/assets/view/layout_observer_demo.view.ron"
    ))
    .expect("observer example asset should parse");
    let db = LayeredFactDatabase::new();
    let local = LocalState::new();

    let desired =
        compute_desired_state(&asset, Vec2::new(960.0, 540.0), &db, &local, "", None, None);

    let surface = desired
        .roots
        .iter()
        .find(|element| element.name == "ObserverSurface")
        .expect("surface element");
    let surface_bounds = visual_bounds(surface, Vec3::ZERO).expect("surface bounds");

    for name in [
        "ObserverHeader",
        "ObserverBody",
        "ObserverBadge",
        "HeaderLeft",
        "ObserverLineA",
    ] {
        let bounds = find_descendant_bounds(surface, name, Vec3::ZERO).expect("descendant bounds");
        assert!(
            contains_bounds(surface_bounds, bounds),
            "{name} bounds {:?} should be inside surface {:?}",
            bounds,
            surface_bounds
        );
    }
}

mod repeat;

#[derive(Debug, Clone, Copy)]
struct Bounds {
    left: f32,
    right: f32,
    top: f32,
    bottom: f32,
}

fn visual_bounds(element: &DesiredElement, parent_translation: Vec3) -> Option<Bounds> {
    let center = parent_translation + element.transform.translation;
    let rect = element.layout_rect?;
    Some(Bounds {
        left: center.x - rect.width * 0.5,
        right: center.x + rect.width * 0.5,
        top: center.y + rect.height * 0.5,
        bottom: center.y - rect.height * 0.5,
    })
}

fn contains_bounds(container: Bounds, child: Bounds) -> bool {
    child.left >= container.left
        && child.right <= container.right
        && child.top <= container.top
        && child.bottom >= container.bottom
}

fn find_descendant_bounds(
    root: &DesiredElement,
    name: &str,
    parent_translation: Vec3,
) -> Option<Bounds> {
    let translation = parent_translation + root.transform.translation;
    root.children.iter().find_map(|child| {
        if child.name == name {
            visual_bounds(child, translation)
        } else {
            find_descendant_bounds(child, name, translation)
        }
    })
}
