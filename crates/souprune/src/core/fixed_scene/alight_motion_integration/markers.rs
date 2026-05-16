//! # markers.rs
//!
//! # markers.rs 文件
//!
//! ## Module Overview
//!
//! ## 模块概述
//!
//! Classifies newly spawned Alight Motion entities for the battle layer. It listens to
//! imported layer-spawn notifications, tags entities as bullets, collision boundaries, or hidden layers,
//! and then propagates those markers through the imported hierarchy when needed.
//!
//! 负责给战斗层里新生成的 Alight Motion 实体打分类标记。它会监听导入图层的生成
//! 通知，把实体标成弹幕、碰撞边界或隐藏层，并在需要时把这些标记沿导入层级继续向下传播。

use super::{
    AlightMotionBattlePatterns, AlightMotionBoundaryMarker, AlightMotionBulletMarker,
    AlightMotionEntity, AlightMotionHiddenMarker,
};
use bevy::prelude::*;
use bevy_alight_motion::prelude::{AmEntitySpawned, AmForceHidden};

/// Observer function that handles AmEntitySpawned events.
pub(super) fn on_am_entity_spawned(
    trigger: On<AmEntitySpawned>,
    mut commands: Commands,
    patterns: Option<Res<AlightMotionBattlePatterns>>,
) {
    let event = trigger.event();
    let layer_name = &event.layer_name;

    trace!(
        "[AM Battle] Entity spawned: '{}' (type={:?})",
        layer_name, event.element_type
    );

    commands.entity(event.entity).insert(AlightMotionEntity);

    if let Some(patterns) = patterns {
        if let Some(ref regex) = patterns.bullet_regex
            && regex.is_match(layer_name)
        {
            commands
                .entity(event.entity)
                .insert(AlightMotionBulletMarker);
            trace!(
                "  → Matched bullet pattern, added AlightMotionBulletMarker to '{}'",
                layer_name
            );
        }

        if let Some(ref regex) = patterns.boundary_regex
            && regex.is_match(layer_name)
        {
            commands
                .entity(event.entity)
                .insert(AlightMotionBoundaryMarker);
            trace!(
                "  → Matched boundary pattern, added AlightMotionBoundaryMarker to '{}'",
                layer_name
            );
        }

        if let Some(ref regex) = patterns.hidden_regex
            && regex.is_match(layer_name)
        {
            commands.entity(event.entity).insert((
                AlightMotionHiddenMarker,
                AmForceHidden,
                Visibility::Hidden,
            ));
        }
    }
}

/// System to propagate AM markers from parent groups to children.
pub(super) fn propagate_am_markers_system(
    mut commands: Commands,
    am_entities: Query<
        (
            Entity,
            Option<&AlightMotionBulletMarker>,
            Option<&AlightMotionBoundaryMarker>,
            Option<&AlightMotionHiddenMarker>,
        ),
        With<AlightMotionEntity>,
    >,
    parent_query: Query<&ChildOf>,
) {
    for (entity, bullet_marker, boundary_marker, hidden_marker) in am_entities.iter() {
        let has_bullet = bullet_marker.is_some();
        let has_boundary = boundary_marker.is_some();
        let has_hidden = hidden_marker.is_some();

        if has_bullet && has_boundary && has_hidden {
            continue;
        }

        let mut current = entity;
        let mut inherited_bullet = false;
        let mut inherited_boundary = false;
        let mut inherited_hidden = false;

        while let Ok(child_of) = parent_query.get(current) {
            let parent = child_of.parent();

            let Ok((_, parent_bullet, parent_boundary, parent_hidden)) = am_entities.get(parent)
            else {
                current = parent;
                continue;
            };

            if !has_bullet && parent_bullet.is_some() {
                inherited_bullet = true;
            }
            if !has_boundary && parent_boundary.is_some() {
                inherited_boundary = true;
            }
            if !has_hidden && parent_hidden.is_some() {
                inherited_hidden = true;
            }

            if (has_bullet || inherited_bullet)
                && (has_boundary || inherited_boundary)
                && (has_hidden || inherited_hidden)
            {
                break;
            }

            current = parent;
        }

        if inherited_bullet {
            commands.entity(entity).insert(AlightMotionBulletMarker);
            info!(
                "[AM Battle] Inherited AlightMotionBulletMarker to entity {:?}",
                entity
            );
        }
        if inherited_boundary {
            commands.entity(entity).insert(AlightMotionBoundaryMarker);
            info!(
                "[AM Battle] Inherited AlightMotionBoundaryMarker to entity {:?}",
                entity
            );
        }
        if inherited_hidden {
            commands.entity(entity).insert((
                AlightMotionHiddenMarker,
                AmForceHidden,
                Visibility::Hidden,
            ));
            info!(
                "[AM Battle] Inherited AlightMotionHiddenMarker + AmForceHidden to entity {:?}",
                entity
            );
        }
    }
}

/// System to apply visibility hidden to entities with AlightMotionHiddenMarker.
pub(super) fn apply_am_hidden_visibility(
    mut hidden_entities: Query<(Entity, &Name, &mut Visibility), With<AlightMotionHiddenMarker>>,
) {
    for (entity, name, mut visibility) in hidden_entities.iter_mut() {
        if *visibility != Visibility::Hidden {
            *visibility = Visibility::Hidden;
            info!(
                "[AM Battle] Applied Hidden visibility to entity {:?} '{}'",
                entity, name
            );
        }
    }
}
