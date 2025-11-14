use crate::core::animation::components::{
    SpriteAnimationClip, SpriteAnimationCurrentFrame, SpriteAnimationTimer,
};
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

    // Add components for an entity that has a SpriteAnimationClip but no SpriteAnimationCurrentFrame
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

pub(crate) fn animate_sprite_system(
    time: Res<Time>,
    mut query: Query<(
        &mut SpriteAnimationTimer,
        &mut SpriteAnimationCurrentFrame,
        &SpriteAnimationClip,
    )>,
) {
    for (mut timer, mut current_frame, clip) in query.iter_mut() {
        if timer.tick(time.delta().as_secs_f32()) && clip.len() > 1 {
            current_frame.value = (current_frame.value + 1) % clip.len();
            if !clip.is_looping() && current_frame.value == clip.len() - 1 {
                current_frame.value = clip.len() - 1;
            }
        }
    }
}
fn apply_sprite_animation(
    sprite: &mut Sprite,
    clip: &mut SpriteAnimationClip,
    current_frame: &SpriteAnimationCurrentFrame,
) {
    clip.frame = current_frame.value;
    *sprite = clip.get_current_sprite().clone();
}

pub(crate) fn update_sprite_animation_system(
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
    mut commands: Commands,
    mut sprite_params: crate::core::sprite::params::SpriteParams,
    mut query: Query<
        (
            Entity,
            &mut Sprite,
            &mut SpriteAnimationClip,
            &SpriteAnimationCurrentFrame,
        ),
        Added<SpriteAnimationCurrentFrame>,
    >,
) {
    for (entity, mut sprite, mut clip, current_frame) in query.iter_mut() {
        apply_sprite_animation(&mut sprite, &mut clip, current_frame);

        // Get frame duration from configuration
        let frame_duration = sprite_params
            .create_sprite_context()
            .get_animation_frame_duration(clip.clip_name());
        commands
            .entity(entity)
            .insert(SpriteAnimationTimer::new(frame_duration));
    }
}
