use super::{
    AlightMotionBattleBoxMarker, AlightMotionBattlePatterns, AlightMotionBulletMarker,
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

        if let Some(ref regex) = patterns.battle_box_regex
            && regex.is_match(layer_name)
        {
            commands
                .entity(event.entity)
                .insert(AlightMotionBattleBoxMarker);
            trace!(
                "  → Matched battle_box pattern, added AlightMotionBattleBoxMarker to '{}'",
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
            Option<&AlightMotionBattleBoxMarker>,
            Option<&AlightMotionHiddenMarker>,
        ),
        With<AlightMotionEntity>,
    >,
    parent_query: Query<&ChildOf>,
) {
    for (entity, bullet_marker, battle_box_marker, hidden_marker) in am_entities.iter() {
        let has_bullet = bullet_marker.is_some();
        let has_battle_box = battle_box_marker.is_some();
        let has_hidden = hidden_marker.is_some();

        if has_bullet && has_battle_box && has_hidden {
            continue;
        }

        let mut current = entity;
        let mut inherited_bullet = false;
        let mut inherited_battle_box = false;
        let mut inherited_hidden = false;

        while let Ok(child_of) = parent_query.get(current) {
            let parent = child_of.parent();

            let Ok((_, parent_bullet, parent_battle_box, parent_hidden)) = am_entities.get(parent)
            else {
                current = parent;
                continue;
            };

            if !has_bullet && parent_bullet.is_some() {
                inherited_bullet = true;
            }
            if !has_battle_box && parent_battle_box.is_some() {
                inherited_battle_box = true;
            }
            if !has_hidden && parent_hidden.is_some() {
                inherited_hidden = true;
            }

            if (has_bullet || inherited_bullet)
                && (has_battle_box || inherited_battle_box)
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
        if inherited_battle_box {
            commands.entity(entity).insert(AlightMotionBattleBoxMarker);
            info!(
                "[AM Battle] Inherited AlightMotionBattleBoxMarker to entity {:?}",
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
