//! Text animation systems — bridges TextBlock entities to dialogue channels,
//! applies shake and wave effects to visible glyphs.
//!
//! 文本动画系统 — 桥接 TextBlock 实体到对话通道，对可见字形应用抖动和波浪效果。

use bevy::prelude::*;
use bevy_bitmap_text::{GlyphBaseOffset, GlyphEntity, ShakeEffect};
use bevy_fact_rule_event::LayeredFactDatabase;

use crate::core::dialogue::components::TextBlockDialogueChannel;
use crate::core::dialogue::text_animation_config::TextAnimationConfig;
use crate::core::fre_facts;
use crate::core::view::components::text::ViewTextTemplate;

/// System set for text animation systems (runs after TypewriterSystemSet).
///
/// 文本动画系统集（在 TypewriterSystemSet 之后运行）。
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct TextAnimationSystemSet;

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

/// For visible glyphs in a text block with an active shake preset, inserts or updates
/// [`ShakeEffect`] on each glyph entity.
///
/// 对于有活跃抖动预设的文本块中的可见字形，在每个字形实体上插入或更新 `ShakeEffect`。
///
/// Applies to ALL visible glyph children unconditionally (not just recently-revealed ones),
/// matching the original Undertale per-frame shake behavior.
///
/// 无条件应用于所有可见子字形（不仅仅是最近揭示的），与 Undertale 原始的逐帧抖动行为一致。
pub fn typewriter_shake_system(
    config: Res<TextAnimationConfig>,
    facts: Res<LayeredFactDatabase>,
    text_block_query: Query<(&TextBlockDialogueChannel, &Children)>,
    glyph_query: Query<&GlyphEntity>,
    mut commands: Commands,
) {
    for (channel, children) in text_block_query.iter() {
        let preset_name = facts
            .get_string(&fre_facts::dialogue_channel_key(
                &channel.0,
                fre_facts::DIALOGUE_TEXT_STYLE,
            ))
            .filter(|s| !s.is_empty());

        let Some(preset) = config.resolve_preset(preset_name) else {
            continue;
        };

        let Some(shake_def) = &preset.shake else {
            continue;
        };
        if shake_def.intensity <= 0.0 {
            continue;
        }

        for child in children.iter() {
            if glyph_query.get(child).is_ok()
                && let Ok(mut entity_commands) = commands.get_entity(child)
            {
                entity_commands.try_insert(ShakeEffect {
                    intensity: shake_def.intensity,
                });
            }
        }
    }
}

/// Applies wave or orbiting distortion to visible glyphs.
///
/// 对可见字形应用波浪或轨道扭曲。
///
/// Two modes:
/// - **Orbiting** (`orbit_angle_per_char_deg` is set): each glyph orbits its base position in a
///   circle, with phase offset proportional to `char_index * angle`. This matches Undertale's
///   `shake == 43` draw effect (Mad Dummy style).
/// - **Spatial wave** (`orbit_angle_per_char_deg` is None): traditional Y-position-based sine wave
///   distortion across the text area.
///
/// 两种模式：
/// - **轨道**（`orbit_angle_per_char_deg` 已设置）：每个字形围绕基位置做圆周运动，
///   相位偏移与 `char_index * angle` 成正比。与 Undertale 的 `shake == 43` 绘制效果一致
///   （Mad Dummy 风格）。
/// - **空间波浪**（`orbit_angle_per_char_deg` 为 None）：传统的基于 Y 坐标的正弦波浪。
pub fn typewriter_wave_system(
    time: Res<Time>,
    config: Res<TextAnimationConfig>,
    facts: Res<LayeredFactDatabase>,
    text_block_query: Query<(&TextBlockDialogueChannel, &Children)>,
    mut glyph_query: Query<(&GlyphEntity, &GlyphBaseOffset, &mut Transform)>,
) {
    let elapsed = time.elapsed_secs();

    for (channel, children) in text_block_query.iter() {
        let preset_name = facts
            .get_string(&fre_facts::dialogue_channel_key(
                &channel.0,
                fre_facts::DIALOGUE_TEXT_STYLE,
            ))
            .filter(|s| !s.is_empty());

        let Some(preset) = config.resolve_preset(preset_name) else {
            continue;
        };

        let Some(wave_def) = &preset.wave else {
            continue;
        };

        if wave_def.orbit_angle_per_char_deg.is_some() {
            apply_orbiting_wave(elapsed, wave_def, children, &mut glyph_query);
        } else {
            apply_spatial_wave(elapsed, wave_def, children, &mut glyph_query);
        }
    }
}

/// Orbiting mode — each glyph orbits its base position (Undertale shake == 43).
///
/// 轨道模式 — 每个字形围绕基位置旋转（Undertale shake == 43）。
fn apply_orbiting_wave(
    elapsed: f32,
    wave_def: &souprune_schema::dialogue::TextWaveDef,
    children: &Children,
    glyph_query: &mut Query<(&GlyphEntity, &GlyphBaseOffset, &mut Transform)>,
) {
    let angle_per_char_rad =
        wave_def.orbit_angle_per_char_deg.unwrap_or(0.0).to_radians();
    for child in children.iter() {
        let Ok((glyph, base_offset, mut transform)) = glyph_query.get_mut(child) else {
            continue;
        };
        let phase =
            elapsed * wave_def.frequency + glyph.char_index as f32 * angle_per_char_rad;
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
        let x_off = (elapsed * wave_def.frequency + glyph_y * phase_scale).sin()
            * wave_def.amplitude;
        let y_off = (elapsed * wave_def.frequency * 1.3 + glyph_y * phase_scale).cos()
            * wave_def.amplitude
            * 0.5;
        transform.translation.x = base_offset.0.x + x_off;
        transform.translation.y = base_offset.0.y + y_off;
    }
}

// ── Shake system (moved from bevy_bitmap_text — UT/DR-specific algorithm) ──

/// Animate glyphs with `ShakeEffect` using hash-based per-frame per-entity pseudo-random jitter.
///
/// 使用基于哈希的逐帧逐实体伪随机抖动，为 `ShakeEffect` 字形制作动画。
///
/// Matches Undertale's `shake = 1` draw behaviour: each character gets a uniformly-distributed
/// random offset in `[-0.5, 0.5) × intensity` on each axis per frame via
/// `random(shake) - shake/2`.
///
/// 匹配 Undertale 的 `shake = 1` 绘制行为：每帧每字符在每个轴上获得
/// `[-0.5, 0.5) × intensity` 的均匀分布随机偏移。
pub fn text_shake_system(
    time: Res<Time>,
    mut query: Query<(Entity, &ShakeEffect, &GlyphBaseOffset, &mut Transform)>,
) {
    let frame = (time.elapsed_secs() * 30.0) as u64;

    for (entity, shake, base, mut transform) in query.iter_mut() {
        let eid = entity.to_bits();
        let hx = hash_u64(eid ^ frame ^ 0xdead_beef_cafe_babe);
        let hy = hash_u64(eid ^ frame ^ 0x8bad_f00d_1ced_c0c0);
        let dx = ((hx as f64 / u64::MAX as f64) as f32 - 0.5) * shake.intensity;
        let dy = ((hy as f64 / u64::MAX as f64) as f32 - 0.5) * shake.intensity;
        transform.translation.x = base.0.x + dx;
        transform.translation.y = base.0.y + dy;
    }
}

/// SplitMix64-style finalizer — avalanches bits for uniform pseudo-random distribution.
///
/// SplitMix64 风格终结器 — 对位进行雪崩混合以获得均匀伪随机分布。
#[inline]
fn hash_u64(x: u64) -> u64 {
    let mut z = x.wrapping_add(0x9e37_79b9_7f4a_7c15);
    z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    z ^ (z >> 31)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_dialogue_channel_from_template() {
        assert_eq!(
            extract_dialogue_channel("{{dialogue:battle_narration:text}}"),
            Some("battle_narration".into())
        );
        assert_eq!(
            extract_dialogue_channel("{{$dialogue:battle_enemy_speech:text}}"),
            Some("battle_enemy_speech".into())
        );
        assert_eq!(
            extract_dialogue_channel("{{dialogue:main:text}}"),
            Some("main".into())
        );
        assert_eq!(extract_dialogue_channel("static text"), None);
    }
}
