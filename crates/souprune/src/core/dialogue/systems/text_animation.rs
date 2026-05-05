//! Text animation systems — bridges TextBlock entities to dialogue channels,
//! applies shake and wave effects to visible glyphs.
//!
//! 文本动画系统 — 桥接 TextBlock 实体到对话通道，对可见字形应用抖动和波浪效果。

use bevy::prelude::*;
use bevy_bitmap_text::{
    GlyphBaseOffset, GlyphEntity, GlyphReveal, ShakeEffect, TextBlock, TwitchEffect,
};
use bevy_ecs_typewriter::Typewriter;
use bevy_fact_rule_event::LayeredFactDatabase;
use souprune_schema::dialogue::TextShakeModeDef;

mod twitch;

use crate::core::dialogue::components::TextBlockDialogueChannel;
use crate::core::dialogue::text_animation_config::TextAnimationConfig;
use crate::core::view::components::text::{ViewTextAnimationStyle, ViewTextTemplate};
use crate::core::view::sdf_view_shape::parse_text_preserving_whitespace;

use super::lifecycle::DialogueControllerEntity;

/// System set for text animation systems (runs after TypewriterSystemSet).
///
/// 文本动画系统集（在 TypewriterSystemSet 之后运行）。
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct TextAnimationSystemSet;

/// System set for dialogue text-block synchronization.
///
/// 对话文本块同步系统集。
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct TextBlockSyncSystemSet;

/// Links [`TextBlock`](bevy_bitmap_text::TextBlock) entities to their dialogue channels.
///
/// 将 TextBlock 实体链接到它们的对话通道。
///
/// Parses `ViewTextTemplate` strings (e.g. `{{dialogue:battle_narration:text}}`),
/// extracts the channel name, and inserts [`TextBlockDialogueChannel`].
///
/// 解析 `ViewTextTemplate` 字符串（如 `{{dialogue:battle_narration:text}}`），
/// 提取通道名并插入 `TextBlockDialogueChannel`。
pub fn link_textblock_dialogue_channel_system(
    mut commands: Commands,
    text_query: Query<(Entity, &ViewTextTemplate), Changed<ViewTextTemplate>>,
) {
    for (entity, template) in text_query.iter() {
        if let Some(channel_name) = extract_dialogue_channel(&template.0) {
            commands
                .entity(entity)
                .insert(TextBlockDialogueChannel(channel_name));
        }
    }
}

/// Mirrors typewriter source text and reveal progress into linked text blocks.
///
/// 将打字机源文本与揭示进度镜像到已链接的文本块。
pub fn sync_typewriter_reveal_to_textblocks_system(
    mut commands: Commands,
    typewriter_query: Query<
        (&crate::core::dialogue::DialogueChannel, &Typewriter),
        With<DialogueControllerEntity>,
    >,
    mut text_query: Query<(
        Entity,
        &TextBlockDialogueChannel,
        &mut TextBlock,
        Option<&mut GlyphReveal>,
    )>,
) {
    for (entity, text_channel, mut text_block, reveal) in text_query.iter_mut() {
        let Some((_channel, typewriter)) = typewriter_query
            .iter()
            .find(|(channel, _typewriter)| channel.name == text_channel.0)
        else {
            if reveal.is_some() {
                commands.entity(entity).remove::<GlyphReveal>();
            }
            continue;
        };

        if text_block.full_text() != typewriter.source_text {
            *text_block = parse_text_preserving_whitespace(&typewriter.source_text);
        }

        match reveal {
            Some(mut reveal) => {
                if reveal.visible_count != typewriter.current_char_index {
                    reveal.visible_count = typewriter.current_char_index;
                }
            }
            None => {
                commands.entity(entity).insert(GlyphReveal {
                    visible_count: typewriter.current_char_index,
                });
            }
        }
    }
}

fn extract_dialogue_channel(template: &str) -> Option<String> {
    let rest = template.trim();
    let inner = rest.strip_prefix("{{")?.strip_suffix("}}")?.trim();
    let inner = inner.strip_prefix('$').unwrap_or(inner);
    let after_dialogue = inner.strip_prefix("dialogue:")?;
    let channel = after_dialogue.split(':').next()?;
    if channel.is_empty() {
        None
    } else {
        Some(channel.to_string())
    }
}

/// Applies the active shake preset to visible glyphs in a text block.
///
/// 对文本块中的可见字形应用当前启用的抖动预设。
///
/// Continuous mode marks every visible glyph; random-single mode marks at most one glyph
/// during each successful interval pulse.
///
/// 连续模式会标记每个可见字形；随机单字符模式只会在成功触发的间隔脉冲中
/// 标记至多一个字形。
pub fn typewriter_shake_system(
    time: Res<Time>,
    config: Res<TextAnimationConfig>,
    facts: Res<LayeredFactDatabase>,
    text_block_query: Query<
        (
            Entity,
            Option<&TextBlockDialogueChannel>,
            Option<&ViewTextAnimationStyle>,
            &Children,
        ),
        Or<(With<TextBlockDialogueChannel>, With<ViewTextAnimationStyle>)>,
    >,
    glyph_query: Query<&GlyphEntity>,
    mut commands: Commands,
) {
    for (entity, channel, text_style, children) in text_block_query.iter() {
        let preset = text_style
            .map(|style| config.resolve_preset(Some(&style.0)))
            .or_else(|| channel.map(|channel| config.resolve_channel_preset(&facts, &channel.0)))
            .flatten();
        let Some(preset) = preset else {
            remove_shake_effects(children, &glyph_query, &mut commands);
            continue;
        };

        let Some(shake_def) = &preset.shake else {
            remove_shake_effects(children, &glyph_query, &mut commands);
            continue;
        };
        if shake_def.intensity <= 0.0 {
            remove_shake_effects(children, &glyph_query, &mut commands);
            continue;
        }

        match shake_def.mode {
            TextShakeModeDef::Continuous => {
                apply_continuous_shake(children, &glyph_query, shake_def.intensity, &mut commands);
            }
            TextShakeModeDef::RandomSingle {
                interval_seconds,
                chance,
                duration_seconds,
            } => {
                apply_random_single_shake(
                    entity,
                    time.elapsed_secs(),
                    interval_seconds,
                    chance,
                    duration_seconds,
                    children,
                    &glyph_query,
                    shake_def.intensity,
                    &mut commands,
                );
            }
            TextShakeModeDef::Twitch {
                average_frames,
                frame_variation,
            } => {
                twitch::apply_twitch_shake(
                    entity,
                    time.elapsed_secs(),
                    average_frames,
                    frame_variation,
                    children,
                    &glyph_query,
                    shake_def.intensity,
                    &mut commands,
                );
            }
        }
    }
}

fn apply_continuous_shake(
    children: &Children,
    glyph_query: &Query<&GlyphEntity>,
    intensity: f32,
    commands: &mut Commands,
) {
    for child in children.iter() {
        if glyph_query.get(child).is_ok()
            && let Ok(mut entity_commands) = commands.get_entity(child)
        {
            entity_commands.remove::<TwitchEffect>();
            entity_commands.try_insert(ShakeEffect { intensity });
        }
    }
}

fn apply_random_single_shake(
    entity: Entity,
    elapsed: f32,
    interval_seconds: f32,
    chance: f32,
    duration_seconds: f32,
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

    let Some(target_index) = random_single_target_index(
        elapsed,
        interval_seconds,
        chance,
        duration_seconds,
        entity.to_bits(),
        glyph_children.len(),
    ) else {
        remove_shake_effects(children, glyph_query, commands);
        return;
    };

    for (index, (child, _char_index)) in glyph_children.into_iter().enumerate() {
        let Ok(mut entity_commands) = commands.get_entity(child) else {
            continue;
        };
        entity_commands.remove::<TwitchEffect>();
        if index == target_index {
            entity_commands.try_insert(ShakeEffect { intensity });
        } else {
            entity_commands.remove::<ShakeEffect>();
        }
    }
}

fn random_single_target_index(
    elapsed: f32,
    interval_seconds: f32,
    chance: f32,
    duration_seconds: f32,
    seed: u64,
    glyph_count: usize,
) -> Option<usize> {
    if glyph_count == 0 || interval_seconds <= 0.0 || chance <= 0.0 || duration_seconds <= 0.0 {
        return None;
    }

    let elapsed = elapsed.max(0.0);
    let interval_position = elapsed % interval_seconds;
    if interval_position > duration_seconds.min(interval_seconds) {
        return None;
    }

    let tick = (elapsed / interval_seconds).floor() as u64;
    let roll = random_unit(hash_u64(seed ^ tick ^ 0xa5a5_5a5a_c3c3_3c3c));
    if roll >= chance.min(1.0) {
        return None;
    }

    let index_hash = hash_u64(seed ^ tick ^ 0x517c_c1b7_2722_0a95);
    Some((index_hash % glyph_count as u64) as usize)
}

#[inline]
fn random_unit(value: u64) -> f32 {
    (value as f64 / u64::MAX as f64) as f32
}

#[inline]
fn hash_u64(x: u64) -> u64 {
    let mut z = x.wrapping_add(0x9e37_79b9_7f4a_7c15);
    z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    z ^ (z >> 31)
}

fn remove_shake_effects(
    children: &Children,
    glyph_query: &Query<&GlyphEntity>,
    commands: &mut Commands,
) {
    for child in children.iter() {
        if glyph_query.get(child).is_ok()
            && let Ok(mut entity_commands) = commands.get_entity(child)
        {
            entity_commands.remove::<ShakeEffect>();
            entity_commands.remove::<TwitchEffect>();
        }
    }
}

/// Applies wave or orbiting distortion to visible glyphs.
///
/// 对可见字形应用波浪或轨道扭曲。
///
/// Two modes:
/// - **Orbiting** (`orbit_angle_per_char_deg` is set): each glyph orbits its base position in a
///   circle, with phase offset proportional to `char_index * angle`.
/// - **Spatial wave** (`orbit_angle_per_char_deg` is None): traditional Y-position-based sine wave
///   distortion across the text area.
///
/// 两种模式：
/// - **轨道**（`orbit_angle_per_char_deg` 已设置）：每个字形围绕基位置做圆周运动，
///   相位偏移与 `char_index * angle` 成正比。
/// - **空间波浪**（`orbit_angle_per_char_deg` 为 None）：传统的基于 Y 坐标的正弦波浪。
pub fn typewriter_wave_system(
    time: Res<Time>,
    config: Res<TextAnimationConfig>,
    facts: Res<LayeredFactDatabase>,
    text_block_query: Query<
        (
            Option<&TextBlockDialogueChannel>,
            Option<&ViewTextAnimationStyle>,
            &Children,
        ),
        Or<(With<TextBlockDialogueChannel>, With<ViewTextAnimationStyle>)>,
    >,
    mut glyph_query: Query<(&GlyphEntity, &GlyphBaseOffset, &mut Transform)>,
) {
    let elapsed = time.elapsed_secs();

    for (channel, text_style, children) in text_block_query.iter() {
        let preset = text_style
            .map(|style| config.resolve_preset(Some(&style.0)))
            .or_else(|| channel.map(|channel| config.resolve_channel_preset(&facts, &channel.0)))
            .flatten();
        let Some(preset) = preset else {
            reset_wave_transforms(children, &mut glyph_query);
            continue;
        };

        let Some(wave_def) = &preset.wave else {
            reset_wave_transforms(children, &mut glyph_query);
            continue;
        };

        if wave_def.orbit_angle_per_char_deg.is_some() {
            apply_orbiting_wave(elapsed, wave_def, children, &mut glyph_query);
        } else {
            apply_spatial_wave(elapsed, wave_def, children, &mut glyph_query);
        }
    }
}

fn reset_wave_transforms(
    children: &Children,
    glyph_query: &mut Query<(&GlyphEntity, &GlyphBaseOffset, &mut Transform)>,
) {
    for child in children.iter() {
        let Ok((_glyph, base_offset, mut transform)) = glyph_query.get_mut(child) else {
            continue;
        };
        transform.translation.x = base_offset.0.x;
        transform.translation.y = base_offset.0.y;
    }
}

/// Orbiting mode — each glyph orbits its base position.
///
/// 轨道模式 — 每个字形围绕基位置旋转。
fn apply_orbiting_wave(
    elapsed: f32,
    wave_def: &souprune_schema::dialogue::TextWaveDef,
    children: &Children,
    glyph_query: &mut Query<(&GlyphEntity, &GlyphBaseOffset, &mut Transform)>,
) {
    let angle_per_char_rad = wave_def
        .orbit_angle_per_char_deg
        .unwrap_or(0.0)
        .to_radians();
    for child in children.iter() {
        let Ok((glyph, base_offset, mut transform)) = glyph_query.get_mut(child) else {
            continue;
        };
        let phase = elapsed * wave_def.frequency + glyph.char_index as f32 * angle_per_char_rad;
        let orb_x = wave_def.amplitude * phase.cos();
        let orb_y = wave_def.amplitude * phase.sin();
        transform.translation.x = base_offset.0.x + orb_x;
        transform.translation.y = base_offset.0.y + orb_y;
    }
}

/// Spatial wave mode — Y-position-based sine distortion.
///
/// 空间波浪模式 — 基于 Y 坐标的正弦扭曲。
fn apply_spatial_wave(
    elapsed: f32,
    wave_def: &souprune_schema::dialogue::TextWaveDef,
    children: &Children,
    glyph_query: &mut Query<(&GlyphEntity, &GlyphBaseOffset, &mut Transform)>,
) {
    let phase_scale = 0.1;
    for child in children.iter() {
        let Ok((_glyph, base_offset, mut transform)) = glyph_query.get_mut(child) else {
            continue;
        };
        let glyph_y = base_offset.0.y;
        let x_off =
            (elapsed * wave_def.frequency + glyph_y * phase_scale).sin() * wave_def.amplitude;
        let y_off = (elapsed * wave_def.frequency * 1.3 + glyph_y * phase_scale).cos()
            * wave_def.amplitude
            * 0.5;
        transform.translation.x = base_offset.0.x + x_off;
        transform.translation.y = base_offset.0.y + y_off;
    }
}

#[cfg(test)]
#[path = "text_animation/tests.rs"]
mod tests;
