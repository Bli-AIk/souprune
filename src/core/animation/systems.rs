use crate::core::animation::{SpriteAnimationClip, SpriteAnimationCurrentFrame};
use bevy::prelude::*;
use bevy::sprite::Sprite;

pub(crate) fn sync_sprite_animation_system(
    mut commands: Commands,
    query_without_sprite: Query<Entity, (Without<Sprite>, With<SpriteAnimationClip>)>,
    query_with_clip: Query<
        Entity,
        (
            With<SpriteAnimationClip>,
            Without<SpriteAnimationCurrentFrame>,
        ),
    >,
    query_without_clip: Query<
        Entity,
        (
            Without<SpriteAnimationClip>,
            With<SpriteAnimationCurrentFrame>,
        ),
    >,
) {
    // If the entity has no Sprite but a SpriteAnimationClip, remove the SpriteAnimationClip
    for entity in query_without_sprite.iter() {
        commands.entity(entity).remove::<SpriteAnimationClip>();
    }

    // Add a component for an entity that has a SpriteAnimationClip but no SpriteAnimationCurrentFrame
    for entity in query_with_clip.iter() {
        commands
            .entity(entity)
            .insert(SpriteAnimationCurrentFrame::default());
    }

    // Remove component for entity without SpriteAnimationClip but with SpriteAnimationCurrentFrame
    for entity in query_without_clip.iter() {
        commands
            .entity(entity)
            .remove::<SpriteAnimationCurrentFrame>();
    }
}
fn apply_sprite_animation(
    sprite: &mut Sprite,
    clip: &mut SpriteAnimationClip,
    current_frame: &SpriteAnimationCurrentFrame,
) {
    *sprite = clip.get_current_sprite().clone();
    clip.frame = current_frame.value;
    println!("{}", clip.frame);
}

pub(crate) fn update_sprite_animation_system(
    time: Res<Time>,
    mut query: Query<(
        &mut Sprite,
        &mut SpriteAnimationClip,
        &SpriteAnimationCurrentFrame,
    )>,
) {
    for (mut sprite, mut clip, current_frame) in query.iter_mut() {
        if clip.frame != current_frame.value {
            apply_sprite_animation(&mut sprite, &mut clip, current_frame);
        }
    }
}

pub(crate) fn setup_sprite_animation_clip_system(
    mut query: Query<
        (
            &mut Sprite,
            &mut SpriteAnimationClip,
            &SpriteAnimationCurrentFrame,
        ),
        Added<SpriteAnimationCurrentFrame>,
    >,
) {
    for (mut sprite, mut clip, current_frame) in query.iter_mut() {
        apply_sprite_animation(&mut sprite, &mut clip, current_frame);
    }
}
