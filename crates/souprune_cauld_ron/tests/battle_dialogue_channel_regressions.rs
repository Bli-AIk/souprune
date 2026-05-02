//! Regression tests for battle dialogue channel authoring.
//!
//! 战斗对话通道编排的回归测试。

use souprune_schema::battle::BattleSpeechBubbleAdvance;
use souprune_schema::danmaku::{DanmakuPerformance, TimelineCueDef};
use souprune_schema::fre::{FreAsset, RuleEventDef};
use souprune_schema::sequence::{Chapter, FactModificationDef, FactValueMatch, SequenceAsset};
use souprune_schema::view::ViewLayoutAsset;
use std::fs;
use std::path::Path;

#[test]
fn demo_turn_narration_targets_battle_narration_channel() {
    let sequence = read_fixture_sequence(
        "mad_dummy_example",
        "battle/chapters/demo_turn_narration.sequence.ron",
    );

    let mut has_pending_channel = false;
    let mut has_channel_typewriter = false;
    let mut has_channel_focus = false;
    let mut has_channel_voice = false;

    for chapter in sequence.chapters {
        let Chapter::ModifyFact { modifications } = chapter else {
            continue;
        };
        for modification in modifications {
            let FactModificationDef::Set { key, value } = modification else {
                continue;
            };
            match (key.as_str(), value) {
                ("dialogue:pending_channel", FactValueMatch::String(channel))
                    if channel == "battle_narration" =>
                {
                    has_pending_channel = true
                }
                ("dialogue:battle_narration:has_typewriter", FactValueMatch::Bool(true)) => {
                    has_channel_typewriter = true
                }
                ("dialogue:battle_narration:has_focus", FactValueMatch::Bool(false)) => {
                    has_channel_focus = true
                }
                ("dialogue:battle_narration:voice", FactValueMatch::String(path))
                    if path == "assets/audios/voice/voice_typewriter_default.wav" =>
                {
                    has_channel_voice = true;
                }
                _ => {}
            }
        }
    }

    assert!(has_pending_channel);
    assert!(has_channel_typewriter);
    assert!(has_channel_focus);
    assert!(has_channel_voice);
}

#[test]
fn narrative_dialogue_rules_advance_enemy_speech_channel() {
    let rules = read_fixture_fre("undertale_preset", "narrative/dialogue.fre.ron");

    let has_enemy_advance = rules.rules.iter().any(|rule| {
        matches!(
            &rule.event,
            RuleEventDef::ActionEvent {
                action,
                kind: souprune_schema::fre::ActionEventKind::JustPressed,
            } if action == "Confirm"
        ) && rule
            .conditions
            .iter()
            .any(|condition| condition == "$dialogue:battle_enemy_speech:has_focus == true")
            && rule.conditions.iter().any(|condition| {
                condition == "$dialogue:battle_enemy_speech:typewriter_playing == false"
            })
            && rule
                .outputs
                .iter()
                .any(|output| output == "dialogue_advance")
    });

    let has_enemy_skip = rules.rules.iter().any(|rule| {
        matches!(
            &rule.event,
            RuleEventDef::ActionEvent {
                action,
                kind: souprune_schema::fre::ActionEventKind::JustPressed,
            } if action == "Cancel"
        ) && rule
            .conditions
            .iter()
            .any(|condition| condition == "$dialogue:battle_enemy_speech:has_focus == true")
            && rule.conditions.iter().any(|condition| {
                condition == "$dialogue:battle_enemy_speech:typewriter_playing == true"
            })
            && rule
                .outputs
                .iter()
                .any(|output| output == "dialogue_skip_typewriter")
    });

    assert!(has_enemy_advance);
    assert!(has_enemy_skip);
}

#[test]
fn narrative_dialogue_rules_advance_default_dialogue_channel() {
    let rules = read_fixture_fre("undertale_preset", "narrative/dialogue.fre.ron");

    let has_default_advance = rules.rules.iter().any(|rule| {
        matches!(
            &rule.event,
            RuleEventDef::ActionEvent {
                action,
                kind: souprune_schema::fre::ActionEventKind::JustPressed,
            } if action == "Confirm"
        ) && rule
            .conditions
            .iter()
            .any(|condition| condition == "$dialogue:has_focus == true")
            && rule
                .conditions
                .iter()
                .any(|condition| condition == "$dialogue:typewriter_playing == false")
            && rule
                .outputs
                .iter()
                .any(|output| output == "dialogue_advance")
    });

    let has_default_confirm_skip = rules.rules.iter().any(|rule| {
        matches!(
            &rule.event,
            RuleEventDef::ActionEvent {
                action,
                kind: souprune_schema::fre::ActionEventKind::JustPressed,
            } if action == "Confirm"
        ) && rule
            .conditions
            .iter()
            .any(|condition| condition == "$dialogue:has_focus == true")
            && rule
                .conditions
                .iter()
                .any(|condition| condition == "$dialogue:typewriter_playing == true")
            && rule
                .outputs
                .iter()
                .any(|output| output == "dialogue_skip_typewriter")
    });

    let has_default_cancel_skip = rules.rules.iter().any(|rule| {
        matches!(
            &rule.event,
            RuleEventDef::ActionEvent {
                action,
                kind: souprune_schema::fre::ActionEventKind::JustPressed,
            } if action == "Cancel"
        ) && rule
            .conditions
            .iter()
            .any(|condition| condition == "$dialogue:has_focus == true")
            && rule
                .conditions
                .iter()
                .any(|condition| condition == "$dialogue:typewriter_playing == true")
            && rule
                .outputs
                .iter()
                .any(|output| output == "dialogue_skip_typewriter")
    });

    assert!(has_default_advance);
    assert!(has_default_confirm_skip);
    assert!(has_default_cancel_skip);
}

#[test]
fn enemy_speech_confirm_skips_typewriter_before_advancing() {
    let rules = read_fixture_fre("undertale_preset", "narrative/dialogue.fre.ron");

    let has_enemy_confirm_skip = rules.rules.iter().any(|rule| {
        matches!(
            &rule.event,
            RuleEventDef::ActionEvent {
                action,
                kind: souprune_schema::fre::ActionEventKind::JustPressed,
            } if action == "Confirm"
        ) && rule
            .conditions
            .iter()
            .any(|condition| condition == "$dialogue:battle_enemy_speech:has_focus == true")
            && rule.conditions.iter().any(|condition| {
                condition == "$dialogue:battle_enemy_speech:typewriter_playing == true"
            })
            && rule
                .outputs
                .iter()
                .any(|output| output == "dialogue_skip_typewriter")
    });

    assert!(has_enemy_confirm_skip);
}

#[test]
fn overworld_dialogue_view_reads_main_channel_text() {
    let view = read_fixture_view("undertale_preset", "overworld/view/dialogue.view.ron");
    let node = view
        .roots
        .iter()
        .find(|node| node.name == "DialogueBox")
        .expect("overworld dialogue view should define DialogueBox");
    let text = node
        .texts
        .iter()
        .find(|text| text.id == "DialogueText")
        .expect("DialogueBox should define DialogueText");

    assert_eq!(
        node.visible_when.as_deref(),
        Some("$dialogue:active == true")
    );
    assert_eq!(text.content.as_deref(), Some("{{dialogue:main:text}}"));
    assert!(
        view.facts
            .as_ref()
            .is_none_or(|facts| !facts.contains_key("dialogue_text")),
        "overworld dialogue text should come from the dialogue main channel"
    );
}

#[test]
fn cotton_first_turn_uses_typed_battle_speech_bubble_chapters() {
    let sequence = read_fixture_sequence(
        "mad_dummy_example",
        "battle/turns/cotton_first_turn.sequence.ron",
    );

    let bubbles: Vec<_> = sequence
        .chapters
        .iter()
        .filter_map(|chapter| match chapter {
            Chapter::BattleSpeechBubble(request) => Some(request),
            _ => None,
        })
        .collect();

    assert_eq!(bubbles.len(), 2);
    assert_eq!(bubbles[0].mortar_node, "enemy_speech_manual_intro");
    assert_eq!(bubbles[1].mortar_node, "enemy_speech_timed_wave");
    assert!(matches!(
        bubbles[0].advance,
        BattleSpeechBubbleAdvance::Manual
    ));
    assert!(matches!(
        bubbles[1].advance,
        BattleSpeechBubbleAdvance::Timed { duration }
            if (duration - 2.0).abs() < f32::EPSILON
    ));
    assert!(
        sequence
            .chapters
            .iter()
            .all(|chapter| !matches!(chapter, Chapter::Custom { .. })),
        "speech bubbles should not be authored through Custom chapters"
    );
}

#[test]
fn cotton_first_turn_performance_does_not_embed_enemy_speech_bubble() {
    let performance = read_fixture_performance(
        "mad_dummy_example",
        "battle/danmaku/cotton_first_turn.performance.ron",
    );

    let has_battle_speech_cue = performance
        .timeline
        .iter()
        .any(|event| matches!(event.cue, Some(TimelineCueDef::BattleSpeechBubble(_))));

    assert!(!has_battle_speech_cue);
}

#[test]
fn undertale_view_renders_enemy_bubble_text_from_container() {
    let view = read_fixture_view("undertale_preset", "battle/view/undertale.view.ron");
    let node = view
        .roots
        .iter()
        .find(|node| node.name == "EnemySpeechBubble")
        .expect("undertale battle view should define EnemySpeechBubble");

    assert!(
        node.sprite.is_none(),
        "EnemySpeechBubble should be a container; standalone sprite nodes ignore same-node texts"
    );

    let sprite_node = node
        .children
        .iter()
        .find(|child| child.name == "EnemySpeechBubbleSprite")
        .expect("EnemySpeechBubble should have a dedicated sprite child");
    let sprite = sprite_node
        .sprite
        .as_ref()
        .expect("EnemySpeechBubbleSprite should have a sprite");
    let scale = sprite
        .transform
        .as_ref()
        .and_then(|transform| transform.scale.as_ref())
        .expect("EnemySpeechBubbleSprite should have explicit scale");
    assert_static_float(&scale.0, 0.5);
    assert_static_float(&scale.1, 0.5);
    assert_static_float(&scale.2, 1.0);

    let text = node
        .texts
        .iter()
        .find(|text| text.id == "EnemySpeechText")
        .expect("EnemySpeechBubble should define EnemySpeechText");
    assert_eq!(text.font, "speechbubble");
}

fn read_fixture_sequence(project: &str, relative_path: &str) -> SequenceAsset {
    read_fixture_ron(project, relative_path)
}

fn read_fixture_fre(project: &str, relative_path: &str) -> FreAsset {
    read_fixture_ron(project, relative_path)
}

fn read_fixture_performance(project: &str, relative_path: &str) -> DanmakuPerformance {
    read_fixture_ron(project, relative_path)
}

fn read_fixture_view(project: &str, relative_path: &str) -> ViewLayoutAsset {
    read_fixture_ron(project, relative_path)
}

fn assert_static_float(value: &souprune_schema::val::Val<f32>, expected: f32) {
    let Some(actual) = value.as_static() else {
        panic!("expected static float value, got expression");
    };
    assert!(
        (*actual - expected).abs() < f32::EPSILON,
        "expected {expected}, got {actual}"
    );
}

fn read_fixture_ron<T>(project: &str, relative_path: &str) -> T
where
    T: serde::de::DeserializeOwned,
{
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let regression_path = manifest_dir
        .join("tests/fixtures/battle_dialogue_channel_regressions")
        .join(project)
        .join(relative_path);
    let baseline_path = manifest_dir
        .join("tests/fixtures/project_ron_baselines")
        .join(project)
        .join(relative_path);
    let path = if regression_path.exists() {
        regression_path
    } else {
        baseline_path
    };
    let ron_text = fs::read_to_string(&path)
        .unwrap_or_else(|err| panic!("fixture RON should be readable: {}: {err}", path.display()));
    ron::Options::default()
        .with_default_extension(ron::extensions::Extensions::IMPLICIT_SOME)
        .from_str(&ron_text)
        .unwrap_or_else(|err| panic!("fixture RON should parse: {}: {err}", path.display()))
}
