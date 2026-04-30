//! Regression tests for battle dialogue channel authoring.
//!
//! 战斗对话通道编排的回归测试。

use souprune_schema::fre::{FreAsset, RuleEventDef};
use souprune_schema::sequence::{Chapter, FactModificationDef, FactValueMatch, SequenceAsset};
use souprune_schema::view::ViewLayoutAsset;
use std::fs;
use std::path::Path;

#[test]
fn demo_turn_narration_targets_battle_narration_channel() {
    let sequence = read_project_sequence(
        "example_mod",
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
    let rules = read_project_fre("undertale_preset", "narrative/dialogue.fre.ron");

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
fn enemy_speech_confirm_skips_typewriter_before_advancing() {
    let rules = read_project_fre("undertale_preset", "narrative/dialogue.fre.ron");

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
fn undertale_view_renders_enemy_bubble_text_from_container() {
    let view = read_project_view("undertale_preset", "battle/view/undertale.view.ron");
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

fn read_project_sequence(project: &str, relative_path: &str) -> SequenceAsset {
    read_project_ron(project, relative_path)
}

fn read_project_fre(project: &str, relative_path: &str) -> FreAsset {
    read_project_ron(project, relative_path)
}

fn read_project_view(project: &str, relative_path: &str) -> ViewLayoutAsset {
    read_project_ron(project, relative_path)
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

fn read_project_ron<T>(project: &str, relative_path: &str) -> T
where
    T: serde::de::DeserializeOwned,
{
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../projects")
        .join(project)
        .join(relative_path);
    let ron_text = fs::read_to_string(&path)
        .unwrap_or_else(|err| panic!("project RON should be readable: {}: {err}", path.display()));
    ron::Options::default()
        .with_default_extension(ron::extensions::Extensions::IMPLICIT_SOME)
        .from_str(&ron_text)
        .unwrap_or_else(|err| panic!("project RON should parse: {}: {err}", path.display()))
}
