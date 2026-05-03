//! Ghost text effect — UT/DR-specific ghost-mode text rendering.
//!
//! 幽灵文本效果 — UT/DR 特有的幽灵模式文本渲染。

use bevy::prelude::*;
use bevy_bitmap_text::{DynamicGlyphCache, FontId, GlyphBaseOffset, GlyphEntity, GlyphKey};
use bevy_ecs_typewriter::{Typewriter, TypewriterState};
use bevy_fact_rule_event::LayeredFactDatabase;
use souprune_schema::dialogue::TextDisplayDef;

use crate::core::dialogue::components::DialogueChannel;
use crate::core::dialogue::systems::lifecycle::DialogueControllerEntity;
use crate::core::dialogue::text_animation_config::TextAnimationConfig;
use crate::core::fre_facts;

// ── GhostFade component ────────────────────────────────────────────────

/// Ghost text fade-out effect — glyph fades and despawns after linger time.
///
/// 幽灵文本渐隐效果 — 字形在停留时间后渐隐并消失。
///
/// This is a UT/DR-specific component (Napstablook, Mad Dummy style).
/// The fade begins after `linger_seconds` have elapsed since spawn;
/// alpha decreases linearly over 2 seconds, then the entity is despawned.
///
/// 这是 UT/DR 特有的组件（Napstablook、Mad Dummy 风格）。
/// 渐隐在生成后 `linger_seconds` 秒开始；alpha 在 2 秒内线性降低，然后实体被销毁。
#[derive(Component, Debug, Clone, Reflect)]
#[reflect(Component)]
pub struct GhostFade {
    /// How long to wait before fading starts (seconds).
    /// 渐隐开始前的等待时间（秒）。
    pub linger_seconds: f32,
    /// Accumulated time since spawn (seconds).
    /// 自生成以来累积的时间（秒）。
    pub elapsed: f32,
}

// ── GhostTextState ─────────────────────────────────────────────────────

/// Tracks the previous char_index per dialogue controller for change detection.
///
/// 追踪每个对话控制器的前一次 char_index 用于变化检测。
#[derive(Component, Debug, Default, Reflect)]
#[reflect(Component)]
pub struct GhostTextState {
    pub last_char_index: usize,
}

// ── Systems ────────────────────────────────────────────────────────────

/// Animate ghost text fade-out — reduces sprite alpha, despawns when done.
///
/// 幽灵文本渐隐动画 — 降低精灵透明度，完成后销毁实体。
pub fn ghost_fade_system(
    time: Res<Time>,
    mut commands: Commands,
    mut query: Query<(Entity, &mut GhostFade, &mut Sprite)>,
) {
    for (entity, mut fade, mut sprite) in query.iter_mut() {
        fade.elapsed += time.delta_secs();
        let fade_progress = ((fade.elapsed - fade.linger_seconds) / 2.0).clamp(0.0, 1.0);
        if fade_progress > 0.0 {
            let mut color: bevy::color::Srgba = sprite.color.into();
            color.alpha = (1.0 - fade_progress).max(0.0);
            sprite.color = color.into();
        }
        if fade_progress >= 1.0 {
            commands.entity(entity).despawn();
        }
    }
}

/// Spawn a single glyph entity at an arbitrary position, independent of any TextBlock.
///
/// 在任意位置生成单个字形实体，不依赖任何 TextBlock。
fn spawn_standalone_glyph(
    commands: &mut Commands,
    cache: &DynamicGlyphCache,
    character: char,
    font: &FontId,
    size_px: u32,
    world_scale: f32,
    position: Vec2,
    color: bevy::color::Srgba,
) -> Option<Entity> {
    if character == ' ' || character == '\n' {
        return None;
    }

    let key = GlyphKey {
        font_id: font.clone(),
        character,
        size_px,
    };

    let glyph_info = cache.glyph_map_get(&key)?;
    let uv_rect = glyph_info.pixel_rect;

    if uv_rect.width() == 0.0 || uv_rect.height() == 0.0 {
        return None;
    }

    let scale_factor = world_scale / size_px as f32;
    let sprite_size = Vec2::new(
        uv_rect.width() * scale_factor,
        uv_rect.height() * scale_factor,
    );

    let entity = commands
        .spawn((
            GlyphEntity {
                char_index: 0,
                character,
            },
            GlyphBaseOffset(position),
            Sprite {
                image: cache.atlas_image.clone(),
                custom_size: Some(sprite_size),
                rect: Some(uv_rect),
                color: color.into(),
                ..Default::default()
            },
            Transform::from_translation(position.extend(0.002)),
            Visibility::Inherited,
        ))
        .id();

    Some(entity)
}

// ── Ghost text spawn system ────────────────────────────────────────────

/// Watches dialogue controller entities for typewriter advances in ghost mode
/// and spawns standalone ghost glyphs at random positions.
///
/// 在幽灵模式下监视对话控制器实体的打字机推进，并在随机位置生成独立的幽灵字形。
pub fn ghost_text_spawn_system(
    config: Res<TextAnimationConfig>,
    facts: Res<LayeredFactDatabase>,
    cache: Res<DynamicGlyphCache>,
    mut controller_query: Query<
        (&Typewriter, &DialogueChannel, &mut GhostTextState),
        With<DialogueControllerEntity>,
    >,
    mut commands: Commands,
) {
    let mut rng_state: u64 = 0xdead_beef_cafe_babe;

    for (typewriter, channel, mut ghost_state) in controller_query.iter_mut() {
        if typewriter.state != TypewriterState::Playing {
            ghost_state.last_char_index = typewriter.current_char_index;
            continue;
        }

        let preset_name = facts
            .get_string(&fre_facts::dialogue_channel_key(
                &channel.name,
                fre_facts::DIALOGUE_TEXT_STYLE,
            ))
            .filter(|s| !s.is_empty());

        let Some(preset) = config.resolve_preset(preset_name) else {
            ghost_state.last_char_index = typewriter.current_char_index;
            continue;
        };

        let TextDisplayDef::Ghost {
            spawn_area,
            linger_seconds,
        } = &preset.display
        else {
            ghost_state.last_char_index = typewriter.current_char_index;
            continue;
        };

        let current = typewriter.current_char_index;
        let prev = ghost_state.last_char_index;
        if current <= prev {
            continue;
        }

        let chars: Vec<char> = typewriter.source_text.chars().collect();
        let end = current.min(chars.len());
        for &ch in chars[prev..end].iter() {
            if ch == ' ' || ch == '\n' {
                continue;
            }

            // Xorshift64
            rng_state ^= rng_state << 13;
            rng_state ^= rng_state >> 7;
            rng_state ^= rng_state << 17;
            let x =
                spawn_area.x + ((rng_state as f32) / (u64::MAX as f32)) * spawn_area.w;
            rng_state ^= rng_state << 13;
            rng_state ^= rng_state >> 7;
            rng_state ^= rng_state << 17;
            let y =
                spawn_area.y + ((rng_state as f32) / (u64::MAX as f32)) * spawn_area.h;
            let position = Vec2::new(x, y);

            if let Some(glyph_entity) = spawn_standalone_glyph(
                &mut commands,
                &cache,
                ch,
                &FontId::from_name("maintext"),
                32,
                1.0,
                position,
                bevy::color::Srgba::WHITE,
            ) {
                commands.entity(glyph_entity).insert(GhostFade {
                    linger_seconds: *linger_seconds,
                    elapsed: 0.0,
                });
            }
        }

        ghost_state.last_char_index = current;
    }
}
