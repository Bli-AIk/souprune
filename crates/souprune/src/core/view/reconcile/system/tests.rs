//! Tests for View reconciliation systems.
//!
//! View 对账系统测试。

use super::*;
use crate::core::camera::MainGameCamera;
use crate::core::view::components::ActiveView;
use crate::core::view::layout::CoordinateSystem;
use crate::core::view::ron_view::HotReloadableViewRoot;
use bevy_fact_rule_event::FactValue;

fn empty_layout_asset() -> ViewLayoutAsset {
    ViewLayoutAsset {
        roots: Vec::new(),
        requires: Vec::new(),
        facts: None,
        space: None,
        coordinate_system: CoordinateSystem::Standard,
        coordinate_space: None,
    }
}

#[test]
fn changed_active_view_root_local_state_marks_its_layout_asset() {
    let mut app = App::new();
    app.init_resource::<PendingReconciliations>()
        .insert_resource(Assets::<ViewLayoutAsset>::default())
        .add_systems(Update, detect_view_root_local_state_changes_system);

    let handle = app
        .world_mut()
        .resource_mut::<Assets<ViewLayoutAsset>>()
        .add(empty_layout_asset());
    let asset_id = handle.id();
    let root_entity = app
        .world_mut()
        .spawn((
            ViewRoot::new("tests/menu.view.ron".to_string()),
            HotReloadableViewRoot {
                layout_path: "tests/menu.view.ron".to_string(),
                layout_handle: handle,
                pre_spawn_events: Vec::new(),
                pre_spawn_fre_handles: Vec::new(),
            },
            ReconciliationEnabled,
            ActiveView,
        ))
        .id();

    app.update();
    app.world_mut()
        .resource_mut::<PendingReconciliations>()
        .clear();

    app.world_mut()
        .entity_mut(root_entity)
        .get_mut::<ViewRoot>()
        .expect("root should have ViewRoot")
        .override_local_value_for_debug("selection", FactValue::Int(1));

    app.update();

    let pending = app.world().resource::<PendingReconciliations>();
    assert!(pending.asset_ids.contains(&asset_id));
}

#[test]
fn changed_view_root_local_state_marks_layout_asset_without_active_view() {
    let mut app = App::new();
    app.init_resource::<PendingReconciliations>()
        .insert_resource(Assets::<ViewLayoutAsset>::default())
        .add_systems(Update, detect_view_root_local_state_changes_system);

    let handle = app
        .world_mut()
        .resource_mut::<Assets<ViewLayoutAsset>>()
        .add(empty_layout_asset());
    let asset_id = handle.id();
    let root_entity = app
        .world_mut()
        .spawn((
            ViewRoot::new("tests/local-only.view.ron".to_string()),
            HotReloadableViewRoot {
                layout_path: "tests/local-only.view.ron".to_string(),
                layout_handle: handle,
                pre_spawn_events: Vec::new(),
                pre_spawn_fre_handles: Vec::new(),
            },
            ReconciliationEnabled,
        ))
        .id();

    app.update();
    app.world_mut()
        .resource_mut::<PendingReconciliations>()
        .clear();

    app.world_mut()
        .entity_mut(root_entity)
        .get_mut::<ViewRoot>()
        .expect("root should have ViewRoot")
        .override_local_value_for_debug("items", FactValue::StringList(vec!["A".into()]));

    app.update();

    let pending = app.world().resource::<PendingReconciliations>();
    assert!(pending.asset_ids.contains(&asset_id));
}

#[test]
fn changed_global_facts_mark_layout_asset_without_active_view() {
    let mut app = App::new();
    app.init_resource::<PendingReconciliations>()
        .insert_resource(Assets::<ViewLayoutAsset>::default())
        .insert_resource(LayeredFactDatabase::new())
        .add_systems(Update, detect_fact_changes_system);

    let handle = app
        .world_mut()
        .resource_mut::<Assets<ViewLayoutAsset>>()
        .add(empty_layout_asset());
    let asset_id = handle.id();
    app.world_mut().spawn((
        HotReloadableViewRoot {
            layout_path: "tests/fact-only.view.ron".to_string(),
            layout_handle: handle,
            pre_spawn_events: Vec::new(),
            pre_spawn_fre_handles: Vec::new(),
        },
        ReconciliationEnabled,
    ));

    app.update();
    app.world_mut()
        .resource_mut::<PendingReconciliations>()
        .clear();

    app.world_mut()
        .resource_mut::<LayeredFactDatabase>()
        .set_global("label", FactValue::String("changed".to_string()));

    app.update();

    let pending = app.world().resource::<PendingReconciliations>();
    assert!(pending.asset_ids.contains(&asset_id));
}

#[test]
fn changed_active_main_2d_camera_projection_forces_all_reconciliation() {
    let mut app = App::new();
    app.init_resource::<PendingReconciliations>()
        .add_systems(Update, detect_main_camera_view_changes_system);

    let camera_entity = app
        .world_mut()
        .spawn((
            Camera2d,
            Camera::default(),
            Projection::Orthographic(OrthographicProjection::default_2d()),
            MainGameCamera,
        ))
        .id();

    app.update();
    app.world_mut()
        .resource_mut::<PendingReconciliations>()
        .clear();

    let mut camera = app.world_mut().entity_mut(camera_entity);
    let mut projection = camera
        .get_mut::<Projection>()
        .expect("camera should have Projection");
    let Projection::Orthographic(orthographic) = &mut *projection else {
        panic!("projection should be orthographic");
    };
    orthographic.scale = 2.0;

    app.update();

    assert!(app.world().resource::<PendingReconciliations>().force_all);
}
