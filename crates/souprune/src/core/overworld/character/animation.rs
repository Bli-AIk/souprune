//! Generic character animation system.
//!
//! 通用角色动画系统。

use crate::core::animation::components::{
    SpriteAnimationClip, SpriteAnimationCurrentFrame, SpriteAnimationTimer,
};
use crate::core::basic_components::Facing;
use crate::core::character_asset::{
    AnimationConfigAsset, CharacterAnimator, state_animation_entry,
};
use crate::core::overworld::character::components::{StateIdle, StateRunning, StateWalking};
use crate::core::sprite::params::SpriteParams;
use bevy::log::error;
use bevy::prelude::*;

/// Generic character animation system that works for any entity with CharacterAnimator.
///
/// 适用于任何具有 CharacterAnimator 的实体的通用角色动画系统。
pub(crate) fn character_animation_system(
    mut sprite_params: SpriteParams,
    anim_configs: Res<Assets<AnimationConfigAsset>>,
    mut query: Query<(
        &Facing,
        &CharacterAnimator,
        &mut Sprite,
        &mut SpriteAnimationClip,
        &mut SpriteAnimationCurrentFrame,
        &mut SpriteAnimationTimer,
        Option<&StateIdle>,
        Option<&StateWalking>,
        Option<&StateRunning>,
    )>,
) {
    for (facing, animator, mut sprite, mut clip, mut frame, mut timer, _idle, walking, running) in
        query.iter_mut()
    {
        let Some(config) = anim_configs.get(&animator.config) else {
            continue;
        };

        let state_name = if running.is_some() {
            "Run"
        } else if walking.is_some() {
            "Walk"
        } else {
            "Idle"
        };

        let Some(state_mapping) = config.states.get(state_name) else {
            continue;
        };

        let entry = state_animation_entry(state_mapping, &facing.value);
        let entry_path = entry.path();

        if !clip.matches_entry(entry_path, entry.flip_x(), entry.flip_y()) {
            let looping = entry.looping_override().unwrap_or(config.default_looping);
            let frame_duration = entry
                .frame_duration_override()
                .unwrap_or(config.default_frame_duration);

            let new_clip = match SpriteAnimationClip::new(
                &mut sprite_params.create_sprite_context(),
                &config.sprite_source,
                entry_path,
                entry.flip_x(),
                entry.flip_y(),
                looping,
                frame_duration,
            ) {
                Ok(clip) => clip,
                Err(e) => {
                    error!(
                        "Failed to change animation to {}: {}. Using fallback.",
                        entry_path, e
                    );
                    SpriteAnimationClip::fallback(
                        &mut sprite_params.create_sprite_context(),
                        entry_path,
                        frame_duration,
                    )
                }
            };

            frame.value = 0;
            timer.reset();
            *clip = new_clip;
            *sprite = clip.get_current_sprite().clone();
        }
    }
}
