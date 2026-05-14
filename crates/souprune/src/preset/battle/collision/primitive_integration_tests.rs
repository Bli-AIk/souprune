//! Battle-box primitive integration tests.
//!
//! 战斗框 primitive 集成测试。

use super::*;
use crate::core::collision::{CollisionRegionStore, ConstraintHandle, RegionHandle};
use bevy::prelude::Update;

fn app_with_player_constraint_system() -> App {
    let mut app = App::new();
    app.init_resource::<CollisionRegionStore>().add_systems(
        Update,
        (
            cleanup_retired_battle_box_regions_system,
            sync_battle_box_regions_system,
            constrain_player_to_battle_box_system,
        )
            .chain(),
    );
    app
}

fn spawn_box(app: &mut App, id: &str, boundary: CollisionBoundary) -> Entity {
    app.world_mut()
        .spawn((
            BattleBox,
            BattleBoxId(id.to_string()),
            BattleBoxState::default(),
            BattleBoxVisualStyle::default(),
            AlightMotionBattleBoxBounds {
                width: boundary.half_size.x * 2.0,
                height: boundary.half_size.y * 2.0,
                center_offset: Vec2::ZERO,
            },
            Transform::from_translation(boundary.center.extend(0.0)),
            GlobalTransform::from_translation(boundary.center.extend(0.0)),
        ))
        .id()
}

#[test]
fn battle_box_runtime_registers_region_and_player_constraint_handles() {
    let mut app = app_with_player_constraint_system();
    let box_entity = spawn_box(
        &mut app,
        "main",
        CollisionBoundary {
            center: Vec2::ZERO,
            half_size: Vec2::new(50.0, 30.0),
        },
    );
    let player_entity = app
        .world_mut()
        .spawn((
            BehaviorParams::new("test"),
            PhysicsCollider::Circle { radius: 8.0 },
            BoundToBattleBox::new("main"),
            Transform::from_xyz(70.0, -50.0, 0.0),
        ))
        .id();

    app.update();

    let region_handle = app
        .world()
        .entity(box_entity)
        .get::<RegionHandle>()
        .copied()
        .expect("battle box should have a region handle");
    let constraint_handle = app
        .world()
        .entity(player_entity)
        .get::<ConstraintHandle>()
        .copied()
        .expect("bound player should have a movement constraint handle");
    let store = app.world().resource::<CollisionRegionStore>();
    assert_eq!(
        store.region(region_handle).unwrap().boundary.half_size,
        Vec2::new(50.0, 30.0)
    );
    assert_eq!(
        store.movement_constraint(constraint_handle).unwrap().region,
        region_handle
    );

    let player_transform = app
        .world()
        .entity(player_entity)
        .get::<Transform>()
        .unwrap();
    assert_eq!(
        player_transform.translation.truncate(),
        Vec2::new(42.0, -22.0)
    );
}

#[test]
fn battle_box_region_handle_survives_boundary_updates() {
    let mut app = app_with_player_constraint_system();
    let box_entity = spawn_box(
        &mut app,
        "main",
        CollisionBoundary {
            center: Vec2::ZERO,
            half_size: Vec2::new(50.0, 30.0),
        },
    );

    app.update();
    let region_handle = *app
        .world()
        .entity(box_entity)
        .get::<RegionHandle>()
        .expect("battle box should have a region handle");

    {
        let mut entity = app.world_mut().entity_mut(box_entity);
        entity.get_mut::<Transform>().unwrap().translation = Vec3::new(20.0, 10.0, 0.0);
        *entity.get_mut::<GlobalTransform>().unwrap() = GlobalTransform::from_xyz(20.0, 10.0, 0.0);
        let mut bounds = entity.get_mut::<AlightMotionBattleBoxBounds>().unwrap();
        bounds.width = 120.0;
        bounds.height = 80.0;
    }

    app.update();

    let entity_handle = *app
        .world()
        .entity(box_entity)
        .get::<RegionHandle>()
        .expect("battle box should keep its region handle");
    let store = app.world().resource::<CollisionRegionStore>();
    let updated = &store.region(entity_handle).unwrap().boundary;
    assert_eq!(entity_handle, region_handle);
    assert_eq!(updated.center, Vec2::new(20.0, 10.0));
    assert_eq!(updated.half_size, Vec2::new(60.0, 40.0));
}

#[test]
fn battle_box_rebind_replaces_movement_constraint() {
    let mut app = app_with_player_constraint_system();
    let _left_entity = spawn_box(
        &mut app,
        "left",
        CollisionBoundary {
            center: Vec2::new(-80.0, 0.0),
            half_size: Vec2::new(40.0, 30.0),
        },
    );
    let right_entity = spawn_box(
        &mut app,
        "right",
        CollisionBoundary {
            center: Vec2::new(80.0, 0.0),
            half_size: Vec2::new(40.0, 30.0),
        },
    );
    let player_entity = app
        .world_mut()
        .spawn((
            BehaviorParams::new("test"),
            PhysicsCollider::Circle { radius: 8.0 },
            BoundToBattleBox::new("left"),
            Transform::from_xyz(-80.0, 0.0, 0.0),
        ))
        .id();

    app.update();
    let old_constraint = *app
        .world()
        .entity(player_entity)
        .get::<ConstraintHandle>()
        .expect("player should start with a movement constraint");

    app.world_mut()
        .entity_mut(player_entity)
        .get_mut::<Transform>()
        .unwrap()
        .translation = Vec3::new(80.0, 0.0, 0.0);
    app.update();

    let new_constraint = *app
        .world()
        .entity(player_entity)
        .get::<ConstraintHandle>()
        .expect("player should keep a movement constraint after rebinding");
    let bound = app
        .world()
        .entity(player_entity)
        .get::<BoundToBattleBox>()
        .unwrap();
    let right_region = *app
        .world()
        .entity(right_entity)
        .get::<RegionHandle>()
        .expect("right battle box should have a region handle");
    let store = app.world().resource::<CollisionRegionStore>();

    assert_eq!(bound.box_id, "right");
    assert_ne!(new_constraint, old_constraint);
    assert!(store.movement_constraint(old_constraint).is_none());
    assert_eq!(
        store.movement_constraint(new_constraint).unwrap().region,
        right_region
    );
}

#[test]
fn retired_battle_box_removes_region_and_constraints() {
    let mut app = app_with_player_constraint_system();
    let box_entity = spawn_box(
        &mut app,
        "main",
        CollisionBoundary {
            center: Vec2::ZERO,
            half_size: Vec2::new(50.0, 30.0),
        },
    );
    let player_entity = app
        .world_mut()
        .spawn((
            BehaviorParams::new("test"),
            PhysicsCollider::Circle { radius: 8.0 },
            BoundToBattleBox::new("main"),
            Transform::from_xyz(0.0, 0.0, 0.0),
        ))
        .id();

    app.update();
    let region_handle = *app
        .world()
        .entity(box_entity)
        .get::<RegionHandle>()
        .expect("battle box should have a region handle");
    let constraint_handle = *app
        .world()
        .entity(player_entity)
        .get::<ConstraintHandle>()
        .expect("player should have a movement constraint");

    app.world_mut().entity_mut(box_entity).remove::<BattleBox>();
    app.update();

    let store = app.world().resource::<CollisionRegionStore>();
    let bound = app
        .world()
        .entity(player_entity)
        .get::<BoundToBattleBox>()
        .unwrap();
    assert!(
        app.world()
            .entity(box_entity)
            .get::<RegionHandle>()
            .is_none()
    );
    assert!(store.region(region_handle).is_none());
    assert!(store.movement_constraint(constraint_handle).is_none());
    assert!(bound.constraint.is_none());
    assert!(
        app.world()
            .entity(player_entity)
            .get::<ConstraintHandle>()
            .is_none()
    );
}
