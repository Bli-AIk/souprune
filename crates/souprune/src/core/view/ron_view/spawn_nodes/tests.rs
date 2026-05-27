//! Tests for RON View node spawning helpers.
//!
//! RON View 节点生成辅助逻辑测试。

use super::*;
use crate::core::view::layout::ViewLayoutSlot;

#[test]
fn layout_slot_offsets_existing_transform_in_view_coordinates() {
    let slot = ViewLayoutSlot {
        path: "Root/Child".to_string(),
        name: "Child".to_string(),
        x: 210.0,
        y: 120.0,
        width: 100.0,
        height: 40.0,
    };
    let explicit = Transform::from_translation(Vec3::new(5.0, -6.0, 7.0));

    let combined =
        combine_layout_transform(Some(&slot), None, ViewLayoutOrigin::Center, explicit, None);

    assert_eq!(combined.translation, Vec3::new(215.0, -126.0, 7.0));
}

#[test]
fn spatial_layout_slot_uses_plane_units_for_translation() {
    let slot = ViewLayoutSlot {
        path: "Root/Child".to_string(),
        name: "Child".to_string(),
        x: 210.0,
        y: 120.0,
        width: 100.0,
        height: 40.0,
    };
    let plane = ViewWorld3dPlaneDef {
        transform: SerializableTransform::default(),
        rotation_degrees: None,
        plane_size: (6.4, 4.8),
        pixels_per_unit: 100.0,
        camera: ViewCameraTargetDef::Main,
        anchor: Default::default(),
        orientation: Default::default(),
        depth: Default::default(),
        input: Default::default(),
    };
    let explicit = Transform::from_translation(Vec3::new(0.5, -0.25, 7.0));

    let combined = combine_layout_transform(
        Some(&slot),
        None,
        ViewLayoutOrigin::Center,
        explicit,
        Some(&plane),
    );

    assert_eq!(combined.translation, Vec3::new(2.6, -1.45, 7.0));
}

#[derive(Resource)]
struct LayoutMetadataTarget(Entity);

#[derive(Resource)]
struct LayoutMetadataSlots(ViewLayoutSlots);

fn insert_layout_metadata_for_test(
    mut commands: Commands,
    target: Res<LayoutMetadataTarget>,
    slots: Res<LayoutMetadataSlots>,
) {
    let slot = slots.0.get("Root/Child");
    insert_layout_slot_components(&mut commands, target.0, Some(&slots.0), "Root/Child", slot);
}

#[test]
fn layout_slot_metadata_is_inserted_as_runtime_components() {
    let mut slots = ViewLayoutSlots::new();
    slots.push_with_metadata(
        ViewLayoutSlot {
            path: "Root/Child".to_string(),
            name: "Child".to_string(),
            x: 210.0,
            y: 120.0,
            width: 100.0,
            height: 40.0,
        },
        Some(ViewClipRect::new(210.0, 120.0, 100.0, 40.0)),
        Some(ViewScrollState::default()),
    );

    let mut app = App::new();
    let entity = app.world_mut().spawn_empty().id();
    app.insert_resource(LayoutMetadataTarget(entity));
    app.insert_resource(LayoutMetadataSlots(slots));
    app.add_systems(Update, insert_layout_metadata_for_test);

    app.update();

    let entity_ref = app.world().entity(entity);
    assert!(entity_ref.contains::<ViewLayoutRect>());
    assert!(entity_ref.contains::<ViewClipRect>());
    assert!(entity_ref.contains::<ViewScrollState>());
}

#[test]
fn display_none_node_is_not_spawned() {
    let node = ViewNodeDef {
        name: "Hidden".to_string(),
        tags: Vec::new(),
        style: StyleDef {
            display: Some(SerializableDisplay::None),
            ..Default::default()
        },
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
        children: Vec::new(),
        repeat: None,
    };

    assert!(node_display_is_none(&node));
}
