//! Tests for RON-driven View spawning.
//!
//! RON 驱动 View 生成测试。

use super::*;

fn explicit_screen_layout(space: Option<ViewSpaceDef>) -> ViewLayoutAsset {
    ViewLayoutAsset {
        roots: Vec::new(),
        requires: Vec::new(),
        facts: None,
        space,
        coordinate_system: CoordinateSystem::Standard,
        coordinate_space: Some(CoordinateSpaceDef {
            axis_origin: (
                crate::core::sequencer::chapter_schema::Value::Static(0.0),
                crate::core::sequencer::chapter_schema::Value::Static(0.0),
            ),
            y_axis: YAxisDirectionDef::Down,
            rotation: RotationDirectionDef::CounterClockwise,
            extent: CoordinateExtentDef::Explicit((640.0, 480.0)),
        }),
    }
}

#[test]
fn camera_relative_view_offsets_explicit_coordinate_space_to_camera_viewport() {
    let layout = explicit_screen_layout(None);

    let offset = camera_relative_view_offset(&layout, Some(Vec2::new(320.0, 240.0)));

    assert_eq!(offset, Vec2::new(160.0, -120.0));
}

#[test]
fn explicit_world_2d_space_does_not_offset_explicit_coordinate_space() {
    let layout = explicit_screen_layout(Some(ViewSpaceDef::World2d));

    let offset = camera_relative_view_offset(&layout, Some(Vec2::new(320.0, 240.0)));

    assert_eq!(offset, Vec2::ZERO);
}

#[test]
fn layout_viewport_prefers_explicit_coordinate_space() {
    let layout = explicit_screen_layout(None);

    let viewport = layout_viewport_size(&layout, Some(Vec2::new(320.0, 240.0)));

    assert_eq!(viewport, Vec2::new(640.0, 480.0));
}

#[test]
fn layout_uses_3d_plane_space_detects_world_3d_plane() {
    let mut layout = explicit_screen_layout(None);
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

    assert!(layout_uses_3d_plane_space(&layout));
}

#[test]
fn explicit_world_2d_space_does_not_parent_camera_relative_view() {
    let layout = explicit_screen_layout(Some(ViewSpaceDef::World2d));
    let camera = Entity::from_raw_u32(1).expect("test entity should be valid");

    assert_eq!(camera_relative_parent_for_view(&layout, Some(camera)), None);
}

#[test]
fn camera_2d_relative_space_keeps_camera_parent() {
    let layout = explicit_screen_layout(Some(ViewSpaceDef::Camera2dRelative));
    let camera = Entity::from_raw_u32(1).expect("test entity should be valid");

    assert_eq!(
        camera_relative_parent_for_view(&layout, Some(camera)),
        Some(camera)
    );
}

fn focus_node(policy: Option<ViewFocusPolicyDef>, children: Vec<ViewNodeDef>) -> ViewNodeDef {
    ViewNodeDef {
        name: "FocusNode".to_string(),
        tags: Vec::new(),
        style: StyleDef::default(),
        transform: None,
        focus_policy: policy,
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
fn focus_policy_marks_layout_as_focus_scope_candidate() {
    let mut layout = explicit_screen_layout(None);
    layout.roots = vec![focus_node(Some(ViewFocusPolicyDef::Scope), Vec::new())];

    assert!(layout_requests_focus_scope(&layout));
}

#[test]
fn child_focus_policy_marks_layout_as_focus_scope_candidate() {
    let mut layout = explicit_screen_layout(None);
    layout.roots = vec![focus_node(
        None,
        vec![focus_node(Some(ViewFocusPolicyDef::Focusable), Vec::new())],
    )];

    assert!(layout_requests_focus_scope(&layout));
}
