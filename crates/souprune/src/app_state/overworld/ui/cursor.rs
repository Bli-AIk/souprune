use super::components::{
    BoxCursor, BoxCursorOwner, BoxCursorReady, BoxCursorSprite, OverworldUI, OverworldUIBox,
    UIBoxFiller,
};
use crate::app_state::overworld::OverworldState;
use bevy::ecs::relationship::Relationship;
use bevy::prelude::*;
use std::collections::VecDeque;

fn find_ui_box_filler_entity(
    root: Entity,
    children_query: &Query<&Children>,
    filler_query: &Query<(), With<UIBoxFiller>>,
) -> Option<Entity> {
    let mut queue: VecDeque<Entity> = VecDeque::new();
    if let Ok(children) = children_query.get(root) {
        for child in children.iter() {
            queue.push_back(child);
        }
    }

    while let Some(child) = queue.pop_front() {
        if filler_query.get(child).is_ok() {
            return Some(child);
        }

        if let Ok(children) = children_query.get(child) {
            for grandchild in children.iter() {
                queue.push_back(grandchild);
            }
        }
    }

    None
}

/// Spawn the sprite used to represent the Undertale box cursor once the UI filler exists.
///
/// 当 UI 填充实体就绪后，生成 Undertale 样式的光标贴图。
pub(crate) fn spawn_box_cursor_visual_system(
    mut commands: Commands,
    query: Query<(Entity, &BoxCursor), (With<OverworldUIBox>, Without<BoxCursorReady>)>,
    children_query: Query<&Children>,
    filler_query: Query<(), With<UIBoxFiller>>,
) {
    for (entity, cursor) in query.iter() {
        let Some(filler_entity) = find_ui_box_filler_entity(entity, &children_query, &filler_query)
        else {
            continue;
        };

        let mut cursor_transform = cursor.transform();
        cursor_transform.translation = Vec3::ZERO;

        commands.entity(filler_entity).with_children(|parent| {
            parent.spawn((
                Name::new("BoxCursorSprite"),
                BoxCursorSprite,
                BoxCursorOwner(entity),
                cursor.sprite(),
                cursor_transform,
                Visibility::Hidden,
            ));
        });

        commands.entity(entity).insert(BoxCursorReady);
    }
}

/// Update cursor visibility and translation based on selection focus.
///
/// 根据焦点位置更新光标的可见性与位移。
pub(crate) fn update_box_cursor_state_system(
    overworld_state: Res<State<OverworldState>>,
    ui_query: Query<&OverworldUI>,
    mut box_query: Query<&mut BoxCursor, With<OverworldUIBox>>,
    parent_query: Query<&ChildOf>,
    mut sprite_query: Query<
        (&BoxCursorOwner, &mut Transform, &mut Visibility),
        With<BoxCursorSprite>,
    >,
) {
    for (owner, mut transform, mut visibility) in sprite_query.iter_mut() {
        let Ok(mut cursor) = box_query.get_mut(owner.0) else {
            if *visibility != Visibility::Hidden {
                *visibility = Visibility::Hidden;
            }
            continue;
        };

        let Ok(parent) = parent_query.get(owner.0) else {
            if *visibility != Visibility::Hidden {
                *visibility = Visibility::Hidden;
            }
            continue;
        };

        let Ok(overworld_ui) = ui_query.get(parent.get()) else {
            if *visibility != Visibility::Hidden {
                *visibility = Visibility::Hidden;
            }
            continue;
        };

        let mut should_show = overworld_state.get() == &OverworldState::Backpack;
        should_show &= !cursor.is_hidden();
        should_show &= cursor.visibility().is_visible_for(overworld_ui.layer());

        if should_show {
            if let Some(translation) = cursor.translation_for_index(overworld_ui.index())
                && transform.translation != translation {
                    transform.translation = translation;
                }
            if *visibility != Visibility::Inherited {
                *visibility = Visibility::Inherited;
            }
        } else if *visibility != Visibility::Hidden {
            *visibility = Visibility::Hidden;
        }
    }
}
