//! Applies host-owned entity primitive side effects requested by WASM mods.
//!
//! 提交 WASM 模组请求的宿主拥有实体 primitive 副作用。

use bevy::prelude::*;
use bevy_tween::combinator::tween;
use bevy_tween::interpolate::translation;
use bevy_tween::prelude::*;
use std::collections::HashMap;
use std::time::Duration;

use crate::core::sequencer::tween::ViewBoxSizeInterpolator;
use crate::core::view::ViewBox;
use crate::core::wasm_runtime::PendingHostEffect;

/// Maps opaque WASM handles to concrete Bevy entities owned by the host.
///
/// 将 WASM 不透明句柄映射到宿主拥有的 Bevy 实体。
#[derive(Resource, Default)]
pub(crate) struct HostEntityHandles {
    entities: HashMap<u64, Entity>,
}

impl HostEntityHandles {
    pub(crate) fn entity(&self, handle: u64) -> Option<Entity> {
        self.entities.get(&handle).copied()
    }
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct HostEntityPrimitive;

/// Host entity primitive effects waiting for ECS application.
///
/// 等待提交到 ECS 的宿主实体 primitive 副作用。
#[derive(Resource, Default)]
pub(crate) struct PendingHostEntityEffects {
    effects: Vec<PendingHostEffect>,
}

impl PendingHostEntityEffects {
    pub(crate) fn extend(&mut self, effects: Vec<PendingHostEffect>) {
        self.effects.extend(effects);
    }
}

#[derive(Clone, Copy)]
struct PendingViewBoxState {
    center: Vec2,
    size: Vec2,
    border_width: f32,
    visible: bool,
    tween_bounds: Option<PendingViewBoxTween>,
}

#[derive(Default)]
struct PendingViewBoxUpdate {
    bounds: Option<PendingViewBoxBounds>,
    visible: Option<bool>,
}

#[derive(Clone, Copy)]
struct PendingViewBoxTween {
    center: Vec2,
    size: Vec2,
    duration_secs: f32,
}

enum PendingViewBoxBounds {
    Immediate { center: Vec2, size: Vec2 },
    Tween(PendingViewBoxTween),
}

/// Apply host-owned entity primitive effects after a WASM callback returns.
///
/// WASM 回调返回后提交宿主拥有的实体 primitive 副作用。
pub(super) fn apply_host_entity_effects(
    effects: Vec<PendingHostEffect>,
    commands: &mut Commands,
    handles: &mut HostEntityHandles,
    view_boxes: &mut Query<(&mut Transform, &mut ViewBox, &mut Visibility)>,
) {
    if effects.is_empty() {
        return;
    }

    let mut pending_spawns = HashMap::<u64, PendingViewBoxState>::new();
    let mut pending_updates = HashMap::<u64, PendingViewBoxUpdate>::new();

    for effect in effects {
        match effect {
            PendingHostEffect::SpawnViewBox {
                handle,
                center,
                size,
                border_width,
            } => {
                if handles.entities.contains_key(&handle) {
                    warn!("Ignoring duplicate host entity spawn handle={handle}");
                    continue;
                }

                if pending_spawns
                    .insert(
                        handle,
                        PendingViewBoxState {
                            center,
                            size,
                            border_width,
                            visible: true,
                            tween_bounds: None,
                        },
                    )
                    .is_some()
                {
                    warn!("Replacing duplicate pending ViewBox spawn handle={handle}");
                }
            }
            PendingHostEffect::SetViewBoxBounds {
                handle,
                center,
                size,
            } => {
                if let Some(spawn) = pending_spawns.get_mut(&handle) {
                    spawn.center = center;
                    spawn.size = size;
                    spawn.tween_bounds = None;
                } else {
                    pending_updates.entry(handle).or_default().bounds =
                        Some(PendingViewBoxBounds::Immediate { center, size });
                }
            }
            PendingHostEffect::TweenViewBoxBounds {
                handle,
                center,
                size,
                duration_secs,
            } => {
                let tween = PendingViewBoxTween {
                    center,
                    size,
                    duration_secs,
                };
                if let Some(spawn) = pending_spawns.get_mut(&handle) {
                    spawn.tween_bounds = Some(tween);
                } else {
                    pending_updates.entry(handle).or_default().bounds =
                        Some(PendingViewBoxBounds::Tween(tween));
                }
            }
            PendingHostEffect::SetViewBoxVisible { handle, visible } => {
                if let Some(spawn) = pending_spawns.get_mut(&handle) {
                    spawn.visible = visible;
                } else {
                    pending_updates.entry(handle).or_default().visible = Some(visible);
                }
            }
            PendingHostEffect::RemoveEntity { handle } => {
                if pending_spawns.remove(&handle).is_some() {
                    pending_updates.remove(&handle);
                    continue;
                }

                pending_updates.remove(&handle);
                let Some(entity) = handles.entities.remove(&handle) else {
                    warn!("Ignoring remove for unknown host entity handle={handle}");
                    continue;
                };
                commands.entity(entity).despawn();
            }
        }
    }

    for (handle, update) in pending_updates {
        let Some(entity) = handles.entity(handle) else {
            warn!("Ignoring update for unknown host entity handle={handle}");
            continue;
        };
        let Ok((mut transform, mut view_box, mut visibility)) = view_boxes.get_mut(entity) else {
            warn!("Host entity handle={handle} does not point to a ViewBox primitive");
            continue;
        };

        if let Some(bounds) = update.bounds {
            match bounds {
                PendingViewBoxBounds::Immediate { center, size } => {
                    transform.translation.x = center.x;
                    transform.translation.y = center.y;
                    view_box.set_dimensions(size.x, size.y);
                }
                PendingViewBoxBounds::Tween(tween) => {
                    let start_center = transform.translation.truncate();
                    let start_size = Vec2::new(view_box.width(), view_box.height());
                    spawn_view_box_bounds_tween(
                        commands,
                        entity,
                        start_center,
                        start_size,
                        tween.center,
                        tween.size,
                        tween.duration_secs,
                    );
                }
            }
        }
        if let Some(visible) = update.visible {
            *visibility = visibility_from_bool(visible);
        }
    }

    for (handle, state) in pending_spawns {
        let entity = commands
            .spawn((
                ViewBox::new_full(
                    state.size.x,
                    state.size.y,
                    state.border_width,
                    Vec::new(),
                    None,
                    None,
                    Color::BLACK,
                ),
                Transform::from_translation(state.center.extend(0.0)),
                GlobalTransform::default(),
                visibility_from_bool(state.visible),
                InheritedVisibility::default(),
                ViewVisibility::default(),
                HostEntityPrimitive,
                Name::new(format!("WasmViewBox:{handle}")),
            ))
            .id();
        if let Some(tween) = state.tween_bounds {
            spawn_view_box_bounds_tween(
                commands,
                entity,
                state.center,
                state.size,
                tween.center,
                tween.size,
                tween.duration_secs,
            );
        }
        handles.entities.insert(handle, entity);
    }
}

fn spawn_view_box_bounds_tween(
    commands: &mut Commands,
    entity: Entity,
    start_center: Vec2,
    start_size: Vec2,
    end_center: Vec2,
    end_size: Vec2,
    duration_secs: f32,
) {
    let duration = Duration::from_secs_f32(duration_secs);
    let target_component = entity.into_target();
    commands.spawn_empty().animation().insert(tween(
        duration,
        EaseKind::Linear,
        target_component.clone().with(translation(
            start_center.extend(0.0),
            end_center.extend(0.0),
        )),
    ));
    commands.spawn_empty().animation().insert(tween(
        duration,
        EaseKind::Linear,
        target_component.with(ViewBoxSizeInterpolator {
            start_width: start_size.x,
            start_height: start_size.y,
            end_width: end_size.x,
            end_height: end_size.y,
        }),
    ));
}

fn visibility_from_bool(visible: bool) -> Visibility {
    if visible {
        Visibility::Visible
    } else {
        Visibility::Hidden
    }
}

pub(super) fn flush_host_entity_effects_system(
    mut commands: Commands,
    mut pending: ResMut<PendingHostEntityEffects>,
    mut handles: ResMut<HostEntityHandles>,
    mut view_boxes: Query<(&mut Transform, &mut ViewBox, &mut Visibility)>,
) {
    let effects = std::mem::take(&mut pending.effects);
    apply_host_entity_effects(effects, &mut commands, &mut handles, &mut view_boxes);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn applies_view_box_effects_to_the_same_host_entity_handle() {
        let mut app = App::new();
        app.init_resource::<HostEntityHandles>();
        app.insert_resource(TestEffects(vec![
            PendingHostEffect::SpawnViewBox {
                handle: 7,
                center: Vec2::new(10.0, 20.0),
                size: Vec2::new(120.0, 48.0),
                border_width: 4.0,
            },
            PendingHostEffect::SetViewBoxBounds {
                handle: 7,
                center: Vec2::new(15.0, 25.0),
                size: Vec2::new(144.0, 60.0),
            },
            PendingHostEffect::SetViewBoxVisible {
                handle: 7,
                visible: false,
            },
        ]));
        app.add_systems(Update, apply_test_effects_system);

        app.update();

        let handles = app.world().resource::<HostEntityHandles>();
        let entity = handles.entity(7).expect("handle should map to an entity");
        let entity_ref = app.world().entity(entity);
        let transform = entity_ref.get::<Transform>().expect("transform");
        let view_box = entity_ref.get::<ViewBox>().expect("view box");
        let visibility = entity_ref.get::<Visibility>().expect("visibility");

        assert_eq!(transform.translation, Vec3::new(15.0, 25.0, 0.0));
        assert_eq!(view_box.width(), 144.0);
        assert_eq!(view_box.height(), 60.0);
        assert_eq!(*visibility, Visibility::Hidden);
    }

    #[derive(Resource)]
    struct TestEffects(Vec<PendingHostEffect>);

    fn apply_test_effects_system(
        mut commands: Commands,
        mut handles: ResMut<HostEntityHandles>,
        mut effects: ResMut<TestEffects>,
        mut view_boxes: Query<(&mut Transform, &mut ViewBox, &mut Visibility)>,
    ) {
        apply_host_entity_effects(
            std::mem::take(&mut effects.0),
            &mut commands,
            &mut handles,
            &mut view_boxes,
        );
    }
}
