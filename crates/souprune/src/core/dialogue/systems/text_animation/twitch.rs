//! CYF-style twitch text shake helpers.
//!
//! CYF 风格的文本 Twitch 抖动辅助逻辑。

use bevy::prelude::*;
use bevy_bitmap_text::{GlyphEntity, ShakeEffect, TwitchEffect};

use super::hash_u64;

pub(super) fn apply_twitch_shake(
    entity: Entity,
    elapsed: f32,
    average_frames: u32,
    frame_variation: u32,
    children: &Children,
    glyph_query: &Query<&GlyphEntity>,
    intensity: f32,
    commands: &mut Commands,
) {
    let mut glyph_children = children
        .iter()
        .filter_map(|child| {
            glyph_query
                .get(child)
                .ok()
                .map(|glyph| (child, glyph.char_index))
        })
        .collect::<Vec<_>>();
    glyph_children.sort_by_key(|(_entity, char_index)| *char_index);

    let frame = (elapsed.max(0.0) * 60.0).floor() as u64;
    let target = twitch_target_index(
        frame,
        average_frames,
        frame_variation,
        entity.to_bits(),
        glyph_children.len(),
    );

    for (index, (child, _char_index)) in glyph_children.into_iter().enumerate() {
        let Ok(mut entity_commands) = commands.get_entity(child) else {
            continue;
        };
        entity_commands.remove::<ShakeEffect>();
        if target == Some(index) {
            let offset = twitch_offset(entity.to_bits(), frame, intensity);
            entity_commands.try_insert(TwitchEffect { offset });
        } else {
            entity_commands.remove::<TwitchEffect>();
        }
    }
}

fn twitch_target_index(
    frame: u64,
    average_frames: u32,
    frame_variation: u32,
    seed: u64,
    glyph_count: usize,
) -> Option<usize> {
    if glyph_count == 0 || average_frames == 0 {
        return None;
    }

    let mut tick_start = 0_u64;
    let mut tick = 0_u64;
    loop {
        let delay = next_twitch_delay_frames(average_frames, frame_variation, seed, tick);
        let trigger_frame = tick_start + u64::from(delay);
        if frame == trigger_frame {
            let index_hash = hash_u64(seed ^ tick ^ 0x517c_c1b7_2722_0a95);
            return Some((index_hash % glyph_count as u64) as usize);
        }
        if frame < trigger_frame {
            return None;
        }
        tick_start = trigger_frame;
        tick += 1;
    }
}

fn next_twitch_delay_frames(
    average_frames: u32,
    frame_variation: u32,
    seed: u64,
    tick: u64,
) -> u32 {
    let variation = frame_variation.min(average_frames.saturating_sub(1));
    let min = average_frames - variation;
    let range = variation.saturating_mul(2).saturating_add(1);
    min + (hash_u64(seed ^ tick ^ 0x8d58_2f3a_9e37_79b9) % u64::from(range)) as u32
}

fn twitch_offset(seed: u64, frame: u64, intensity: f32) -> Vec2 {
    let angle = random_unit(hash_u64(seed ^ frame ^ 0xc001_d00d_f00d_f00d)) * std::f32::consts::TAU;
    Vec2::new(angle.sin() * intensity, angle.cos() * intensity)
}

#[inline]
fn random_unit(value: u64) -> f32 {
    (value as f64 / u64::MAX as f64) as f32
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::ecs::system::RunSystemOnce;

    #[test]
    fn target_waits_before_first_one_frame_twitch() {
        let seed = 17;
        let first_delay = next_twitch_delay_frames(48, 16, seed, 0);
        assert_eq!(twitch_target_index(0, 48, 16, seed, 4), None);
        assert!(twitch_target_index(u64::from(first_delay), 48, 16, seed, 4).is_some());
        assert_eq!(
            twitch_target_index(u64::from(first_delay) + 1, 48, 16, seed, 4),
            None
        );
    }

    #[test]
    fn next_delay_stays_inside_configured_frame_window() {
        for tick in 0..64 {
            let delay = next_twitch_delay_frames(48, 16, 99, tick);
            assert!((32..=64).contains(&delay));
        }
    }

    #[test]
    fn marks_one_visible_glyph_with_fixed_offset() {
        let mut world = World::new();

        let parent = world.spawn_empty().id();
        let glyphs = (0..3)
            .map(|char_index| {
                world
                    .spawn((
                        GlyphEntity {
                            char_index,
                            character: 'A',
                        },
                        ShakeEffect { intensity: 9.0 },
                    ))
                    .id()
            })
            .collect::<Vec<_>>();
        for glyph in &glyphs {
            world.entity_mut(*glyph).insert(ChildOf(parent));
        }

        let first_delay = next_twitch_delay_frames(48, 16, parent.to_bits(), 0);
        world
            .run_system_once(
                move |children_query: Query<&Children>,
                      glyph_query: Query<&GlyphEntity>,
                      mut commands: Commands| {
                    let children = children_query.get(parent).unwrap();
                    apply_twitch_shake(
                        parent,
                        first_delay as f32 / 60.0,
                        48,
                        16,
                        children,
                        &glyph_query,
                        2.0,
                        &mut commands,
                    );
                },
            )
            .unwrap();

        let twitch_offsets = glyphs
            .iter()
            .filter_map(|glyph| {
                world
                    .get::<TwitchEffect>(*glyph)
                    .map(|effect| effect.offset)
            })
            .collect::<Vec<_>>();
        assert_eq!(twitch_offsets.len(), 1);
        assert!((twitch_offsets[0].length() - 2.0).abs() < 0.0001);
        assert!(
            glyphs
                .iter()
                .all(|glyph| world.get::<ShakeEffect>(*glyph).is_none())
        );
    }
}
