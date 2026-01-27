//! # systems.rs
//!
//! # systems.rs 文件
//!
//! ## Module Overview
//!
//! ## 模块概述
//!
//! Implements systems for updating sprite animations, including frame advancement based on timers and synchronizing the visual `Sprite` component with the current animation frame.
//!
//! 实现用于更新精灵动画的系统，包括基于计时器的帧推进以及将可视化的 `Sprite` 组件与当前动画帧同步。

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
    //
    // 如果实体没有 Sprite 但有 SpriteAnimationClip，则移除 SpriteAnimationClip
    for entity in query_without_sprite.iter() {
        commands.entity(entity).remove::<SpriteAnimationClip>();
    }

    // Add components for an entity that has a SpriteAnimationClip but no SpriteAnimationCurrentFrame
    //
    // 为有 SpriteAnimationClip 但没有 SpriteAnimationCurrentFrame 的实体添加组件
    for entity in query_with_clip.iter() {
        commands
            .entity(entity)
            .insert(SpriteAnimationCurrentFrame::default());
    }

    // Remove component for entity without SpriteAnimationClip but with SpriteAnimationCurrentFrame
    //
    // 为没有 SpriteAnimationClip 但有 SpriteAnimationCurrentFrame 的实体移除组件
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
    // Preserve the existing sprite color before applying animation frame
    let preserved_color = sprite.color;
    *sprite = clip.get_current_sprite().clone();
    // Restore the preserved color (for tinted bullets like orange/blue soul bullets)
    sprite.color = preserved_color;
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
        //
        // 从配置获取帧持续时间
        let frame_duration = sprite_params
            .create_sprite_context()
            .get_animation_frame_duration(clip.clip_name());
        commands
            .entity(entity)
            .insert(SpriteAnimationTimer::new(frame_duration));
    }
}
