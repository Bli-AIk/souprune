use crate::core::dialogue::systems::text_animation::*;

use bevy_bitmap_text::{
    GlyphBaseOffset, GlyphEntity, GlyphReveal, ShakeEffect, TextBlock, TwitchEffect,
};
use bevy_ecs_typewriter::Typewriter;
use bevy_fact_rule_event::{FactValue, LayeredFactDatabase};
use souprune_schema::dialogue::{
    TextAnimationConfigDef, TextAnimationPresetDef, TextDisplayDef, TextShakeDef, TextShakeModeDef,
    TextWaveDef,
};
use std::time::Duration;

use crate::core::dialogue::components::TextBlockDialogueChannel;
use crate::core::dialogue::systems::lifecycle::DialogueControllerEntity;
use crate::core::fre_facts;
use crate::core::view::components::text::ViewTextAnimationStyle;
use crate::{GameUpdateSchedule, ScheduleLabel};

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
fn typewriter_reveal_system_keeps_full_text_while_advancing_visible_count() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_systems(Update, sync_typewriter_reveal_to_textblocks_system);

    let mut typewriter = Typewriter::new("hello", 0.03);
    typewriter.current_text = "h".to_string();
    typewriter.current_char_index = 1;
    let controller = app
        .world_mut()
        .spawn((
            DialogueControllerEntity,
            crate::core::dialogue::DialogueChannel::new("main"),
            typewriter,
        ))
        .id();

    let text_entity = app
        .world_mut()
        .spawn((TextBlockDialogueChannel("main".into()), TextBlock::new("h")))
        .id();

    app.update();

    let text_block = app.world().get::<TextBlock>(text_entity).unwrap();
    assert_eq!(text_block.full_text(), "hello");
    let reveal = app.world().get::<GlyphReveal>(text_entity).unwrap();
    assert_eq!(reveal.visible_count, 1);

    {
        let mut typewriter = app.world_mut().get_mut::<Typewriter>(controller).unwrap();
        typewriter.current_text = "he".to_string();
        typewriter.current_char_index = 2;
    }

    app.update();

    let text_block = app.world().get::<TextBlock>(text_entity).unwrap();
    assert_eq!(text_block.full_text(), "hello");
    let reveal = app.world().get::<GlyphReveal>(text_entity).unwrap();
    assert_eq!(reveal.visible_count, 2);
}

#[test]
fn shake_system_removes_stale_shake_when_preset_has_no_shake() {
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
fn shake_system_removes_stale_twitch_when_preset_has_no_shake() {
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
    app.add_systems(Update, typewriter_shake_system);

    let glyph = app
        .world_mut()
        .spawn((
            GlyphEntity {
                char_index: 0,
                character: 'A',
            },
            TwitchEffect { offset: Vec2::ONE },
        ))
        .id();
    spawn_text_block_parent(&mut app, glyph);

    app.update();

    assert!(app.world().get::<TwitchEffect>(glyph).is_none());
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

#[test]
fn random_single_shake_can_skip_intervals() {
    let seed = 7;
    let skipped = (0..16)
        .any(|tick| random_single_target_index(tick as f32, 1.0, 0.35, 0.5, seed, 4).is_none());

    assert!(skipped);
}

#[test]
fn random_single_shake_does_not_walk_glyphs_sequentially() {
    let seed = 7;
    let selected = (0..8)
        .filter_map(|tick| random_single_target_index(tick as f32, 1.0, 1.0, 0.5, seed, 4))
        .collect::<Vec<_>>();

    assert_ne!(selected, vec![0, 1, 2, 3, 0, 1, 2, 3]);
}

#[test]
fn random_single_shake_only_marks_one_visible_glyph_per_interval() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.insert_resource(TextAnimationConfig(TextAnimationConfigDef {
        default_preset: "random".into(),
        presets: [(
            "random".into(),
            TextAnimationPresetDef {
                display: TextDisplayDef::Normal,
                shake: Some(TextShakeDef {
                    intensity: 1.0,
                    mode: TextShakeModeDef::RandomSingle {
                        interval_seconds: 1.0,
                        chance: 1.0,
                        duration_seconds: 0.5,
                    },
                }),
                wave: None,
            },
        )]
        .into_iter()
        .collect(),
    }));
    app.insert_resource(LayeredFactDatabase::new());
    app.add_systems(Update, typewriter_shake_system);

    let parent = app
        .world_mut()
        .spawn(ViewTextAnimationStyle("random".into()))
        .id();
    let glyphs = (0..3)
        .map(|char_index| {
            app.world_mut()
                .spawn(GlyphEntity {
                    char_index,
                    character: 'A',
                })
                .id()
        })
        .collect::<Vec<_>>();
    for glyph in &glyphs {
        app.world_mut().entity_mut(*glyph).insert(ChildOf(parent));
    }

    app.update();

    let marked = glyphs
        .iter()
        .filter(|glyph| app.world().get::<ShakeEffect>(**glyph).is_some())
        .count();
    assert_eq!(marked, 1);
}

#[test]
fn random_single_shake_removes_stale_twitch_from_all_visible_glyphs() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.insert_resource(TextAnimationConfig(TextAnimationConfigDef {
        default_preset: "random".into(),
        presets: [(
            "random".into(),
            TextAnimationPresetDef {
                display: TextDisplayDef::Normal,
                shake: Some(TextShakeDef {
                    intensity: 1.0,
                    mode: TextShakeModeDef::RandomSingle {
                        interval_seconds: 1.0,
                        chance: 1.0,
                        duration_seconds: 0.5,
                    },
                }),
                wave: None,
            },
        )]
        .into_iter()
        .collect(),
    }));
    app.insert_resource(LayeredFactDatabase::new());
    app.add_systems(Update, typewriter_shake_system);

    let parent = app
        .world_mut()
        .spawn(ViewTextAnimationStyle("random".into()))
        .id();
    let glyphs = (0..3)
        .map(|char_index| {
            app.world_mut()
                .spawn((
                    GlyphEntity {
                        char_index,
                        character: 'A',
                    },
                    TwitchEffect { offset: Vec2::ONE },
                ))
                .id()
        })
        .collect::<Vec<_>>();
    for glyph in &glyphs {
        app.world_mut().entity_mut(*glyph).insert(ChildOf(parent));
    }

    app.update();

    assert!(
        glyphs
            .iter()
            .all(|glyph| app.world().get::<TwitchEffect>(*glyph).is_none())
    );
}

#[derive(ScheduleLabel, Debug, Hash, PartialEq, Eq, Clone)]
struct TestGameUpdate;

#[test]
fn dialogue_twitch_runs_before_bitmap_twitch_transform_application() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.insert_resource(GameUpdateSchedule(TestGameUpdate.intern()));
    app.insert_resource(TextAnimationConfig(TextAnimationConfigDef {
        default_preset: "twitch".into(),
        presets: [(
            "twitch".into(),
            TextAnimationPresetDef {
                display: TextDisplayDef::Normal,
                shake: Some(TextShakeDef {
                    intensity: 2.0,
                    mode: TextShakeModeDef::Twitch {
                        average_frames: 1,
                        frame_variation: 0,
                    },
                }),
                wave: None,
            },
        )]
        .into_iter()
        .collect(),
    }));
    app.insert_resource(LayeredFactDatabase::new());
    let schedule = crate::game_schedule(&app);
    app.add_systems(
        schedule,
        typewriter_shake_system.in_set(TextAnimationSystemSet),
    );
    app.add_systems(
        schedule,
        bevy_bitmap_text::systems::bitmap_text_animation_systems().after(TextAnimationSystemSet),
    );

    let parent = app
        .world_mut()
        .spawn(ViewTextAnimationStyle("twitch".into()))
        .id();
    let glyph = app
        .world_mut()
        .spawn((
            GlyphEntity {
                char_index: 0,
                character: 'A',
            },
            GlyphBaseOffset(Vec2::new(10.0, 20.0)),
            Transform::from_xyz(10.0, 20.0, 0.0),
        ))
        .id();
    app.world_mut().entity_mut(glyph).insert(ChildOf(parent));

    app.world_mut()
        .resource_mut::<Time>()
        .advance_by(Duration::from_secs_f32(1.0 / 60.0));
    app.world_mut().run_schedule(TestGameUpdate);

    assert!(app.world().resource::<Time>().elapsed_secs() >= 1.0 / 60.0);
    assert_eq!(app.world().get::<Children>(parent).unwrap().len(), 1);
    assert!(
        app.world()
            .resource::<TextAnimationConfig>()
            .resolve_preset(Some("twitch"))
            .is_some()
    );
    assert!(app.world().get::<TwitchEffect>(glyph).is_some());
    let transform = app.world().get::<Transform>(glyph).unwrap();
    assert_ne!(transform.translation.truncate(), Vec2::new(10.0, 20.0));
}
