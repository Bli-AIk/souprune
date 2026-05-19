//! Tests for WASM host state and host-api conversions.
//!
//! 覆盖 WASM 宿主状态与 host-api 转换逻辑的测试。

use bevy::prelude::*;
use bevy_fact_rule_event::FactValue;
use std::sync::{Arc, Mutex};

use super::context::CallContext;
use super::conversions::{fre_to_wit_fact, wit_to_fre_fact};
use super::souprune::plugin::host_api::Host;
use super::souprune::plugin::host_api::{
    ColliderShape as WitColliderShape, FactValue as WitFact, Rgba as WitRgba,
    SpriteEntityDesc as WitSpriteEntityDesc, Vec2 as WitVec2,
};
use super::{HostState, PendingHostEffect, PendingSoundEffect};
use crate::core::collision::{CollisionRegionStore, PhysicsCollider, TriggerCollider};

#[test]
fn host_states_share_collision_region_registry() {
    let shared = Arc::new(Mutex::new(CollisionRegionStore::default()));
    let mut first = HostState::new_for_mod(Arc::clone(&shared));
    let mut second = HostState::new_for_mod(Arc::clone(&shared));

    let region =
        first.create_collision_region(WitVec2 { x: 0.0, y: 0.0 }, WitVec2 { x: 20.0, y: 20.0 });
    let constraint = second
        .create_movement_constraint(region, WitColliderShape::Circle(5.0))
        .unwrap();

    let constrained = second
        .constrain_movement(constraint, WitVec2 { x: 30.0, y: 0.0 })
        .unwrap();

    assert_eq!(constrained.x, 15.0);
    assert_eq!(constrained.y, 0.0);

    first.set_collision_region_bounds(
        region,
        WitVec2 { x: 0.0, y: 0.0 },
        WitVec2 { x: 10.0, y: 10.0 },
    );
    let constrained_after_update = second
        .constrain_movement(constraint, WitVec2 { x: 30.0, y: 0.0 })
        .unwrap();

    assert_eq!(constrained_after_update.x, 5.0);
    assert_eq!(constrained_after_update.y, 0.0);
}

#[test]
fn call_context_takes_fact_event_and_host_entity_effects_together() {
    let mut ctx = CallContext::default();
    ctx.pending_fact_mutations
        .push(("cursor:index".to_string(), FactValue::Int(1)));
    ctx.pending_global_fact_mutations
        .push(("player:hp".to_string(), FactValue::Int(12)));
    ctx.pending_events.push("view.cursor.moved".to_string());
    ctx.pending_sounds.push(PendingSoundEffect::FullPath(
        "assets/audios/snd_heal_c.wav".to_string(),
    ));
    ctx.pending_host_effects
        .push(PendingHostEffect::SpawnViewBox {
            handle: 1,
            center: Vec2::new(10.0, 20.0),
            size: Vec2::new(120.0, 48.0),
            border_width: 4.0,
        });

    let pending = ctx.take_pending_side_effects();

    assert_eq!(pending.fact_mutations.len(), 1);
    assert_eq!(pending.global_fact_mutations.len(), 1);
    assert_eq!(pending.events, vec!["view.cursor.moved".to_string()]);
    assert_eq!(
        pending.sounds,
        vec![PendingSoundEffect::FullPath(
            "assets/audios/snd_heal_c.wav".to_string()
        )]
    );
    assert_eq!(pending.host_effects.len(), 1);
    assert!(ctx.pending_fact_mutations.is_empty());
    assert!(ctx.pending_global_fact_mutations.is_empty());
    assert!(ctx.pending_events.is_empty());
    assert!(ctx.pending_sounds.is_empty());
    assert!(ctx.pending_host_effects.is_empty());
}

#[test]
fn wit_fact_conversion_preserves_list_fact_types() {
    match fre_to_wit_fact(&FactValue::StringList(vec!["a".into(), "b".into()])) {
        WitFact::TextList(list) => assert_eq!(list, vec!["a".to_string(), "b".to_string()]),
        _ => panic!("expected TextList"),
    }
    match fre_to_wit_fact(&FactValue::IntList(vec![1, 2])) {
        WitFact::IntList(list) => assert_eq!(list, vec![1, 2]),
        _ => panic!("expected IntList"),
    }
    match fre_to_wit_fact(&FactValue::FloatList(vec![1.5, 2.5])) {
        WitFact::FloatList(list) => assert_eq!(list, vec![1.5, 2.5]),
        _ => panic!("expected FloatList"),
    }
    match fre_to_wit_fact(&FactValue::BoolList(vec![true, false])) {
        WitFact::BoolList(list) => assert_eq!(list, vec![true, false]),
        _ => panic!("expected BoolList"),
    }

    assert_eq!(
        wit_to_fre_fact(WitFact::TextList(vec!["a".into(), "b".into()])),
        FactValue::StringList(vec!["a".into(), "b".into()])
    );
    assert_eq!(
        wit_to_fre_fact(WitFact::IntList(vec![1, 2])),
        FactValue::IntList(vec![1, 2])
    );
    assert_eq!(
        wit_to_fre_fact(WitFact::FloatList(vec![1.5, 2.5])),
        FactValue::FloatList(vec![1.5, 2.5])
    );
    assert_eq!(
        wit_to_fre_fact(WitFact::BoolList(vec![true, false])),
        FactValue::BoolList(vec![true, false])
    );
}

#[test]
fn view_box_host_api_queues_primitives_with_opaque_handles() {
    let shared = Arc::new(Mutex::new(CollisionRegionStore::default()));
    let mut host = HostState::new_for_mod(shared);

    let handle = host.spawn_view_box(
        WitVec2 { x: 10.0, y: 20.0 },
        WitVec2 { x: 120.0, y: 48.0 },
        4.0,
    );
    host.set_view_box_bounds(
        handle,
        WitVec2 { x: 16.0, y: 24.0 },
        WitVec2 { x: 144.0, y: 60.0 },
    );
    host.set_view_box_visible(handle, false);
    host.tween_view_box_bounds(
        handle,
        WitVec2 { x: 32.0, y: 40.0 },
        WitVec2 { x: 180.0, y: 72.0 },
        0.25,
    );
    host.remove_entity(handle);
    let invalid = host.spawn_view_box(
        WitVec2 { x: 0.0, y: 0.0 },
        WitVec2 { x: -1.0, y: 48.0 },
        4.0,
    );

    assert_eq!(handle, 1);
    assert_eq!(invalid, 0);
    assert_eq!(
        host.call_ctx.pending_host_effects,
        vec![
            PendingHostEffect::SpawnViewBox {
                handle,
                center: Vec2::new(10.0, 20.0),
                size: Vec2::new(120.0, 48.0),
                border_width: 4.0,
            },
            PendingHostEffect::SetViewBoxBounds {
                handle,
                center: Vec2::new(16.0, 24.0),
                size: Vec2::new(144.0, 60.0),
            },
            PendingHostEffect::SetViewBoxVisible {
                handle,
                visible: false,
            },
            PendingHostEffect::TweenViewBoxBounds {
                handle,
                center: Vec2::new(32.0, 40.0),
                size: Vec2::new(180.0, 72.0),
                duration_secs: 0.25,
            },
            PendingHostEffect::RemoveEntity { handle },
        ]
    );
}

#[test]
fn sprite_entity_host_api_queues_generic_entity_primitive() {
    let shared = Arc::new(Mutex::new(CollisionRegionStore::default()));
    let mut host = HostState::new_for_mod(shared);

    let handle = host.spawn_sprite_entity(WitSpriteEntityDesc {
        texture: "assets/textures/common/view/heart.png".into(),
        position: WitVec2 { x: 0.0, y: -80.0 },
        z: 10.0,
        color: WitRgba {
            red: 1.0,
            green: 0.0,
            blue: 0.0,
            alpha: 1.0,
        },
        physics_collider: Some(WitColliderShape::Circle(8.0)),
        trigger_collider: Some(WitColliderShape::Rectangle(WitVec2 { x: 2.0, y: 2.0 })),
        behavior_id: Some("soul_red".into()),
        behavior_context: Some("battle".into()),
        bullet_target: true,
        mode_scope: Some("battle".into()),
        name: Some("Player".into()),
    });

    assert_eq!(handle, 1);
    assert_eq!(
        host.call_ctx.pending_host_effects,
        vec![PendingHostEffect::SpawnSpriteEntity {
            handle,
            texture: "assets/textures/common/view/heart.png".into(),
            position: Vec2::new(0.0, -80.0),
            z: 10.0,
            color: Color::srgba(1.0, 0.0, 0.0, 1.0),
            physics_collider: Some(PhysicsCollider::Circle { radius: 8.0 }),
            trigger_collider: Some(TriggerCollider::Box {
                half_size: Vec2::new(2.0, 2.0),
            }),
            behavior_id: Some("soul_red".into()),
            behavior_context: Some("battle".into()),
            bullet_target: true,
            mode_scope: Some("battle".into()),
            name: Some("Player".into()),
        }]
    );
}
