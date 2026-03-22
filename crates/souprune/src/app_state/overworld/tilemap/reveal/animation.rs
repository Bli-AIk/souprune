//! # animation.rs
//!
//! # animation.rs 文件
//!
//! ## Module Overview
//!
//! ## 模块概述
//!
//! This file drives the time-based half of the overworld tile-reveal effect. It groups pending
//! tiles into ripple directions, advances the beat-synchronized reveal waves, and transitions each
//! tile from a temporary sprite proxy back into the real tilemap with a fade-out tail.
//!
//! 这个文件负责大地图揭露效果里与时间推进相关的一半流程。它会按波纹方向整理待揭露的格子，
//! 用节拍驱动揭露波前推进，并把每个格子从临时精灵代理切换回真正的 tilemap，同时补上淡出尾声。

use super::{
    AnimatingTile, AnimationComplete, FADE_EIGHTH_NOTES, FADE_INITIAL, FADE_TARGET,
    RevealedTileSprite, RippleDirection, ScaleInterpolator, TileFadeState, TileRevealState,
};
use bevy::prelude::*;
use bevy_ecs_tiled::prelude::*;
use bevy_tween::prelude::*;
use std::time::Duration;

/// Collect pending tiles for the current step, organized by direction.
pub(super) fn collect_pending_tiles_system(
    mut reveal_state: ResMut<TileRevealState>,
    tiles_query: Query<
        (Entity, &RevealedTileSprite),
        (Without<AnimatingTile>, Without<AnimationComplete>),
    >,
) {
    if !reveal_state.initialized || reveal_state.all_triggered {
        return;
    }

    if !reveal_state.pending_tiles_by_direction.is_empty() {
        return;
    }

    for (entity, tile) in tiles_query.iter() {
        if tile.manhattan_distance == reveal_state.current_step {
            reveal_state
                .pending_tiles_by_direction
                .entry(tile.direction)
                .or_default()
                .push(entity);
        }
    }
}

/// Update the reveal animation, triggering scale tweens for tiles.
pub(super) fn update_reveal_animation_system(
    mut commands: Commands,
    mut reveal_state: ResMut<TileRevealState>,
    mut beat_events: MessageReader<super::super::beat::BeatEvent>,
    beat_tracker: Res<super::super::beat::BeatTracker>,
) {
    if !reveal_state.initialized || reveal_state.all_triggered {
        beat_events.clear();
        return;
    }

    let mut should_step = false;
    for event in beat_events.read() {
        if matches!(event, super::super::beat::BeatEvent::QuarterNote) {
            should_step = true;
            break;
        }
    }

    if !should_step {
        return;
    }

    const TILES_PER_BEAT: usize = 8;

    for tile_index in 0..TILES_PER_BEAT {
        let all_directions = RippleDirection::all();
        let available_directions: Vec<RippleDirection> = all_directions
            .iter()
            .filter(|d| !reveal_state.used_directions.contains(d))
            .copied()
            .collect();

        let available_directions = if available_directions.is_empty() {
            reveal_state.used_directions.clear();
            all_directions.to_vec()
        } else {
            available_directions
        };

        let directions_with_tiles: Vec<RippleDirection> = available_directions
            .iter()
            .filter(|d| {
                reveal_state
                    .pending_tiles_by_direction
                    .get(d)
                    .is_some_and(|v| !v.is_empty())
            })
            .copied()
            .collect();

        let directions_with_tiles = if directions_with_tiles.is_empty() {
            let any_direction_with_tiles: Vec<RippleDirection> = reveal_state
                .pending_tiles_by_direction
                .iter()
                .filter(|(_, v)| !v.is_empty())
                .map(|(d, _)| *d)
                .collect();

            if any_direction_with_tiles.is_empty() {
                advance_reveal_step(&mut reveal_state);
                return;
            }
            any_direction_with_tiles
        } else {
            directions_with_tiles
        };

        if directions_with_tiles.is_empty() {
            break;
        }

        let pseudo_random_seed = (beat_tracker.counts.quarter as usize)
            .wrapping_mul(73856093)
            .wrapping_add((reveal_state.current_step as usize).wrapping_mul(19349663))
            .wrapping_add(reveal_state.used_directions.len().wrapping_mul(83492791))
            .wrapping_add(tile_index.wrapping_mul(47619417));
        let index = pseudo_random_seed % directions_with_tiles.len();
        let selected_direction = directions_with_tiles[index];

        if !reveal_state.used_directions.contains(&selected_direction) {
            reveal_state.used_directions.push(selected_direction);
        }

        let Some(tiles) = reveal_state
            .pending_tiles_by_direction
            .get_mut(&selected_direction)
        else {
            continue;
        };
        let Some(entity) = tiles.pop() else {
            continue;
        };

        let animation_duration_ms = 500;
        let animation_duration = Duration::from_millis(animation_duration_ms);

        commands.entity(entity).insert(AnimatingTile {
            animation_timer: Timer::from_seconds(
                animation_duration_ms as f32 / 1000.0,
                TimerMode::Once,
            ),
        });
        commands.entity(entity).animation().insert_tween_here(
            animation_duration,
            EaseKind::BackOut,
            entity.into_target().with(ScaleInterpolator {
                start: Vec3::ZERO,
                end: Vec3::ONE,
            }),
        );
    }

    let all_empty = reveal_state
        .pending_tiles_by_direction
        .values()
        .all(|v| v.is_empty());

    if all_empty {
        reveal_state.current_step += 1;
        reveal_state.pending_tiles_by_direction.clear();

        if reveal_state.current_step > reveal_state.max_distance {
            reveal_state.all_triggered = true;
            info!("All tile reveal animations triggered");
        }
    }
}

/// Check if animating tiles have completed their animation.
pub(super) fn check_animation_complete_system(
    mut commands: Commands,
    time: Res<Time>,
    mut animating_tiles: Query<(Entity, &mut AnimatingTile), Without<AnimationComplete>>,
) {
    for (entity, mut animating) in animating_tiles.iter_mut() {
        animating.animation_timer.tick(time.delta());
        if animating.animation_timer.is_finished() {
            commands.entity(entity).insert(AnimationComplete);
        }
    }
}

/// Cleanup completed sprite animations: hide sprite, show original tile, add fade state.
pub(super) fn cleanup_completed_sprites_system(
    mut commands: Commands,
    completed_sprites: Query<(Entity, &RevealedTileSprite), With<AnimationComplete>>,
    mut tiles_query: Query<
        (Entity, &TilePos, &mut TileVisible),
        (With<TiledTile>, Without<TileFadeState>),
    >,
) {
    for (sprite_entity, reveal_sprite) in completed_sprites.iter() {
        for (tile_entity, tile_pos, mut tile_visible) in tiles_query.iter_mut() {
            if tile_pos.x == reveal_sprite.tile_pos.0 && tile_pos.y == reveal_sprite.tile_pos.1 {
                tile_visible.0 = true;

                let hash =
                    (tile_pos.x.wrapping_mul(73856093)) ^ (tile_pos.y.wrapping_mul(19349663));
                let random_offset = hash % 8;

                commands.entity(tile_entity).insert(TileFadeState {
                    fade: FADE_INITIAL,
                    random_offset,
                    eighth_notes_elapsed: 0,
                });
            }
        }

        commands.entity(sprite_entity).despawn();
    }
}

/// Update tile fade based on eighth notes.
pub(super) fn update_tile_fade_system(
    mut beat_events: MessageReader<super::super::beat::BeatEvent>,
    mut tiles_query: Query<(&mut TileColor, &mut TileFadeState), With<TiledTile>>,
) {
    let mut eighth_note_count = 0u32;
    for event in beat_events.read() {
        if matches!(event, super::super::beat::BeatEvent::EighthNote) {
            eighth_note_count += 1;
        }
    }

    if eighth_note_count == 0 {
        return;
    }

    let step_size = (FADE_INITIAL - FADE_TARGET) / FADE_EIGHTH_NOTES as f32;

    for (mut tile_color, mut fade_state) in tiles_query.iter_mut() {
        if fade_state.fade <= FADE_TARGET {
            continue;
        }

        fade_state.eighth_notes_elapsed += eighth_note_count;
        if fade_state.eighth_notes_elapsed <= fade_state.random_offset {
            continue;
        }

        let effective_steps = fade_state.eighth_notes_elapsed - fade_state.random_offset;
        let target_fade = (FADE_INITIAL - step_size * effective_steps as f32).max(FADE_TARGET);

        fade_state.fade = target_fade;
        tile_color.0 = Color::srgba(target_fade, 1.0, 1.0, 1.0);
    }
}

/// Advance to the next reveal step and mark as finished when all distances are covered.
fn advance_reveal_step(reveal_state: &mut TileRevealState) {
    reveal_state.current_step += 1;
    reveal_state.pending_tiles_by_direction.clear();
    if reveal_state.current_step > reveal_state.max_distance {
        reveal_state.all_triggered = true;
        info!("All tile reveal animations triggered");
    }
}
