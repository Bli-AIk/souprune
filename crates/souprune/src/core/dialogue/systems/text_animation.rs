//! Text animation systems — bridges TextBlock entities to dialogue channels,
//! applies shake and wave effects to visible glyphs.
//!
//! 文本动画系统 — 桥接 TextBlock 实体到对话通道，对可见字形应用抖动和波浪效果。

use bevy::prelude::*;
use bevy_bitmap_text::{GlyphBaseOffset, GlyphEntity, ShakeEffect};
use bevy_fact_rule_event::LayeredFactDatabase;

use crate::core::dialogue::components::TextBlockDialogueChannel;
use crate::core::dialogue::text_animation_config::TextAnimationConfig;
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
/// matching full-line per-frame shake behavior.
///
/// 无条件应用于所有可见子字形（不仅仅是最近揭示的），与整行逐帧抖动行为一致。
pub fn typewriter_shake_system(
    config: Res<TextAnimationConfig>,
    facts: Res<LayeredFactDatabase>,
    text_block_query: Query<(&TextBlockDialogueChannel, &Children)>,
    glyph_query: Query<&GlyphEntity>,
    mut commands: Commands,
) {
    for (channel, children) in text_block_query.iter() {
        let Some(preset) = config.resolve_channel_preset(&facts, &channel.0) else {
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
    text_block_query: Query<(&TextBlockDialogueChannel, &Children)>,
    mut glyph_query: Query<(&GlyphEntity, &GlyphBaseOffset, &mut Transform)>,
) {
    let elapsed = time.elapsed_secs();

    for (channel, children) in text_block_query.iter() {
        let Some(preset) = config.resolve_channel_preset(&facts, &channel.0) else {
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
mod tests {
    use super::*;
    use bevy_bitmap_text::{GlyphBaseOffset, GlyphEntity, ShakeEffect};
    use bevy_fact_rule_event::{FactValue, LayeredFactDatabase};
    use souprune_schema::dialogue::{
        TextAnimationConfigDef, TextAnimationPresetDef, TextDisplayDef, TextWaveDef,
    };

    use crate::core::fre_facts;

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

    fn spawn_text_block_parent(app: &mut App, glyph: Entity) {
        let parent = app
            .world_mut()
            .spawn(TextBlockDialogueChannel("main".into()))
            .id();
        app.world_mut().entity_mut(glyph).insert(ChildOf(parent));
    }

    #[test]
    fn shake_system_removes_stale_shake_when_preset_has_no_shake() {
        let mut app = App::new();
        app.insert_resource(TextAnimationConfig(TextAnimationConfigDef {
            default_preset: "calm".into(),
            presets: [(
                "calm".into(),
                TextAnimationPresetDef {
                    display: TextDisplayDef::Normal,
                    shake: None,
                    wave: None,
                },
            )]
            .into_iter()
            .collect(),
        }));
        let mut facts = LayeredFactDatabase::new();
        facts.set_global(
            fre_facts::DIALOGUE_TEXT_STYLE,
            FactValue::String("calm".into()),
        );
        app.insert_resource(facts);
        app.add_systems(Update, typewriter_shake_system);

        let glyph = app
            .world_mut()
            .spawn((
                GlyphEntity {
                    char_index: 0,
                    character: 'A',
                },
                ShakeEffect { intensity: 2.0 },
            ))
            .id();
        spawn_text_block_parent(&mut app, glyph);

        app.update();

        assert!(app.world().get::<ShakeEffect>(glyph).is_none());
    }

    #[test]
    fn wave_system_resets_glyph_transform_when_preset_has_no_wave() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.insert_resource(TextAnimationConfig(TextAnimationConfigDef {
            default_preset: "calm".into(),
            presets: [(
                "calm".into(),
                TextAnimationPresetDef {
                    display: TextDisplayDef::Normal,
                    shake: None,
                    wave: None,
                },
            )]
            .into_iter()
            .collect(),
        }));
        let mut facts = LayeredFactDatabase::new();
        facts.set_global(
            fre_facts::DIALOGUE_TEXT_STYLE,
            FactValue::String("calm".into()),
        );
        app.insert_resource(facts);
        app.add_systems(Update, typewriter_wave_system);

        let glyph = app
            .world_mut()
            .spawn((
                GlyphEntity {
                    char_index: 0,
                    character: 'A',
                },
                GlyphBaseOffset(Vec2::new(4.0, 8.0)),
                Transform::from_xyz(99.0, 88.0, 0.0),
            ))
            .id();
        spawn_text_block_parent(&mut app, glyph);

        app.update();

        let transform = app.world().get::<Transform>(glyph).unwrap();
        assert_eq!(transform.translation.truncate(), Vec2::new(4.0, 8.0));
    }

    #[test]
    fn wave_system_applies_configured_wave() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.insert_resource(TextAnimationConfig(TextAnimationConfigDef {
            default_preset: "wave".into(),
            presets: [(
                "wave".into(),
                TextAnimationPresetDef {
                    display: TextDisplayDef::Normal,
                    shake: None,
                    wave: Some(TextWaveDef {
                        amplitude: 2.0,
                        frequency: 1.0,
                        orbit_angle_per_char_deg: None,
                    }),
                },
            )]
            .into_iter()
            .collect(),
        }));
        app.insert_resource(LayeredFactDatabase::new());
        app.add_systems(Update, typewriter_wave_system);

        let glyph = app
            .world_mut()
            .spawn((
                GlyphEntity {
                    char_index: 0,
                    character: 'A',
                },
                GlyphBaseOffset(Vec2::new(4.0, 8.0)),
                Transform::from_xyz(4.0, 8.0, 0.0),
            ))
            .id();
        spawn_text_block_parent(&mut app, glyph);

        app.update();

        let transform = app.world().get::<Transform>(glyph).unwrap();
        assert_ne!(transform.translation.truncate(), Vec2::new(4.0, 8.0));
    }
}
