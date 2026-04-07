//! Manages dialogue controller creation, start/end transitions, and runtime cleanup.
//!
//! 管理对话控制器的创建、开始/结束切换，以及运行时清理。
//!
//! Acts as the lifecycle layer of the dialogue subsystem. It decides when a
//! dialogue session should spawn a controller, when pending dialogue state
//! should turn into live view/runtime state, and when everything can be torn
//! down again after the dialogue has ended.
//!
//! 对话子系统的生命周期层。它决定何时该为一次对话会话创建控制器，
//! 何时把待处理的对话状态转换成真实的 View/运行时状态，以及在对话结束后
//! 何时可以把相关内容清理掉。

use bevy::prelude::*;
use bevy_ecs_typewriter::Typewriter;
use bevy_fact_rule_event::{FactEvent, FactValue, LayeredFactDatabase};
use bevy_mortar_bond::{MortarDialogueFinished, MortarEvent, MortarRuntime};

use crate::core::fre_facts;
use crate::core::view::components::{ActiveView, ViewRoot};

use super::super::auto_pause::AutoPauseState;

#[derive(Component)]
pub struct DialogueControllerEntity;

pub fn has_pending_dialogue_ended(facts: Res<LayeredFactDatabase>) -> bool {
    facts
        .get_bool(fre_facts::DIALOGUE_PENDING_ENDED)
        .unwrap_or(false)
}

pub fn emit_pending_dialogue_ended_system(
    mut facts: ResMut<LayeredFactDatabase>,
    mut fre_event_writer: MessageWriter<FactEvent>,
) {
    if facts
        .get_bool(fre_facts::DIALOGUE_PENDING_ENDED)
        .unwrap_or(false)
    {
        info!("emit_pending_dialogue_ended_system: emitting dialogue:ended");
        fre_event_writer.write(FactEvent::new(fre_facts::DIALOGUE_ENDED));
        facts.remove(fre_facts::DIALOGUE_PENDING_ENDED);
    }
}

pub fn handle_mortar_dialogue_finished_system(
    mut mortar_finished: MessageReader<MortarDialogueFinished>,
    mut fre_event_writer: MessageWriter<FactEvent>,
) {
    for finished in mortar_finished.read() {
        info!(
            "handle_mortar_dialogue_finished_system: Mortar dialogue finished (path: {}, node: {})",
            finished.mortar_path, finished.node
        );
        fre_event_writer.write(FactEvent::new(fre_facts::DIALOGUE_ENDED));
    }
}

pub fn spawn_dialogue_controller_system(
    mut commands: Commands,
    runtime: Res<MortarRuntime>,
    query: Query<Entity, With<DialogueControllerEntity>>,
    facts: Res<LayeredFactDatabase>,
    mut active_view_query: Query<&mut ViewRoot, With<ActiveView>>,
) {
    let has_controller = !query.is_empty();
    let dialogue_active = facts.get_bool(fre_facts::DIALOGUE_ACTIVE).unwrap_or(false);
    let simple_text_active = facts
        .get_bool(fre_facts::DIALOGUE_SIMPLE_TEXT_ACTIVE)
        .unwrap_or(false);
    let has_dialogue = dialogue_active || simple_text_active;
    let has_typewriter = facts
        .get_bool(fre_facts::DIALOGUE_HAS_TYPEWRITER)
        .unwrap_or(true);
    let has_mortar = facts
        .get_bool(fre_facts::DIALOGUE_HAS_MORTAR)
        .unwrap_or(false)
        || runtime.has_active_dialogues();

    if dialogue_active || simple_text_active || has_controller || runtime.has_active_dialogues() {
        debug!(
            "spawn_dialogue_controller_system: dialogue_active={}, simple_text_active={}, has_controller={}, has_mortar={}, runtime_active={}",
            dialogue_active,
            simple_text_active,
            has_controller,
            has_mortar,
            runtime.has_active_dialogues()
        );
    }

    if has_dialogue && !has_controller {
        info!(
            "spawn_dialogue_controller_system: spawning dialogue controller (mortar={}, simple_text={}, typewriter={})",
            has_mortar, simple_text_active, has_typewriter
        );

        let mut entity_commands = commands.spawn(DialogueControllerEntity);

        // Always insert auto-pause state for character tracking
        // 始终插入自动停顿状态以追踪字符进度
        entity_commands.insert(AutoPauseState::default());

        if has_mortar {
            entity_commands.insert(super::super::components::MortarController::new());
        }

        if has_typewriter {
            let simple_text = facts
                .get_string(fre_facts::DIALOGUE_SIMPLE_TEXT)
                .map(|s| s.to_string());
            let initial_text = if !has_mortar {
                simple_text.unwrap_or_default()
            } else {
                String::new()
            };
            let typewriter_speed = facts
                .get_float(fre_facts::DIALOGUE_TYPEWRITER_SPEED)
                .map(|n| n as f32)
                .unwrap_or(0.03);
            let mut typewriter = Typewriter::new(&initial_text, typewriter_speed);
            if !initial_text.is_empty() {
                typewriter.play();
                info!(
                    "spawn_dialogue_controller_system: starting typewriter with simple_text: '{}'",
                    initial_text
                );
            }
            entity_commands.insert(typewriter);

            if let Some(voice_path) = facts.get_string(fre_facts::DIALOGUE_VOICE)
                && !voice_path.is_empty()
            {
                info!(
                    "spawn_dialogue_controller_system: adding TypewriterVoice with path: '{}'",
                    voice_path
                );
                entity_commands.insert(super::super::components::TypewriterVoice::new(voice_path));
            }
        }

        if !has_mortar
            && !has_typewriter
            && let Some(text) = facts.get_string(fre_facts::DIALOGUE_SIMPLE_TEXT)
        {
            info!(
                "spawn_dialogue_controller_system: setting simple_text to View local_facts: '{}'",
                text
            );
            let text_owned = text.to_string();
            for mut view_root in active_view_query.iter_mut() {
                view_root
                    .local_facts
                    .set("dialogue_text", FactValue::String(text_owned.clone()));
            }
        }
    }
}

pub fn should_check_dialogue_despawn(
    fre_events: MessageReader<FactEvent>,
    query: Query<Entity, With<DialogueControllerEntity>>,
) -> bool {
    !fre_events.is_empty() || !query.is_empty()
}

pub fn despawn_dialogue_controller_system(
    mut commands: Commands,
    mut fre_events: MessageReader<FactEvent>,
    runtime: Res<MortarRuntime>,
    query: Query<Entity, With<DialogueControllerEntity>>,
    mut facts: ResMut<LayeredFactDatabase>,
) {
    let mut should_cleanup = false;
    for event in fre_events.read() {
        if event.id.0 == fre_facts::DIALOGUE_ENDED {
            should_cleanup = true;
            break;
        }
    }

    if !should_cleanup {
        let mortar_active = runtime.has_active_dialogues();
        let simple_active = facts
            .bypass_change_detection()
            .get_bool(fre_facts::DIALOGUE_SIMPLE_TEXT_ACTIVE)
            .unwrap_or(false);
        let dialogue_active_fact = facts
            .bypass_change_detection()
            .get_bool(fre_facts::DIALOGUE_ACTIVE)
            .unwrap_or(false);
        let has_controller = !query.is_empty();

        should_cleanup =
            has_controller && !mortar_active && !simple_active && !dialogue_active_fact;
    }

    if !should_cleanup {
        return;
    }

    info!("despawn_dialogue_controller_system: dialogue ended, cleaning up controller");
    facts.set(fre_facts::DIALOGUE_HAS_FOCUS, FactValue::Bool(false));
    facts.set(fre_facts::DIALOGUE_ACTIVE, FactValue::Bool(false));
    facts.set(fre_facts::DIALOGUE_HAS_MORTAR, FactValue::Bool(false));

    for entity in query.iter() {
        commands.entity(entity).despawn();
    }
}

pub fn has_pending_dialogue_start(facts: Res<LayeredFactDatabase>) -> bool {
    facts
        .get_bool(fre_facts::DIALOGUE_PENDING_START)
        .unwrap_or(false)
}

pub fn handle_pending_dialogue_start_system(
    mut facts: ResMut<LayeredFactDatabase>,
    mut mortar_events: MessageWriter<MortarEvent>,
    mut spawn_view_writer: MessageWriter<crate::core::view::SpawnViewRequest>,
    mut fre_event_writer: MessageWriter<FactEvent>,
    locale: Res<crate::extra::mortar::CurrentLocale>,
) {
    let pending_start = facts
        .bypass_change_detection()
        .get_bool(fre_facts::DIALOGUE_PENDING_START)
        .unwrap_or(false);

    if !pending_start {
        return;
    }

    info!("handle_pending_dialogue_start_system: pending_start=true, processing dialogue");

    let pending_view = facts
        .bypass_change_detection()
        .get_string(fre_facts::DIALOGUE_PENDING_VIEW)
        .map(|s| s.to_string())
        .filter(|s| !s.is_empty());
    let mortar_path = facts
        .bypass_change_detection()
        .get_string(fre_facts::DIALOGUE_PENDING_MORTAR_PATH)
        .map(|s| s.to_string())
        .filter(|s| !s.is_empty());
    let mortar_node = facts
        .bypass_change_detection()
        .get_string(fre_facts::DIALOGUE_PENDING_MORTAR_NODE)
        .map(|s| s.to_string())
        .filter(|s| !s.is_empty());

    info!(
        "handle_pending_dialogue_start_system: view={:?}, path={:?}, node={:?}",
        pending_view, mortar_path, mortar_node
    );

    facts.set(fre_facts::DIALOGUE_PENDING_START, FactValue::Bool(false));
    facts.set(
        fre_facts::DIALOGUE_PENDING_VIEW,
        FactValue::String(String::new()),
    );
    facts.set(
        fre_facts::DIALOGUE_PENDING_MORTAR_PATH,
        FactValue::String(String::new()),
    );
    facts.set(
        fre_facts::DIALOGUE_PENDING_MORTAR_NODE,
        FactValue::String(String::new()),
    );

    info!(
        "handle_pending_dialogue_start_system: view={:?}, mortar={:?}",
        pending_view,
        mortar_path.as_ref().zip(mortar_node.as_ref())
    );

    if let Some(view_path) = pending_view {
        spawn_view_writer.write(crate::core::view::SpawnViewRequest {
            path: view_path,
            mode_scope: None,
            bindings: None,
        });
    }

    let has_mortar = mortar_path.is_some() && mortar_node.is_some();
    if let (Some(path), Some(node)) = (mortar_path.clone(), mortar_node.clone()) {
        let localized_path = format!("shared/locales/{}/{}", locale.0, path);

        info!(
            "handle_pending_dialogue_start_system: starting Mortar dialogue '{}' node '{}'",
            localized_path, node
        );

        mortar_events.write(MortarEvent::start_node(localized_path, node));
    }

    let simple_text_active = facts
        .bypass_change_detection()
        .get_bool(fre_facts::DIALOGUE_SIMPLE_TEXT_ACTIVE)
        .unwrap_or(false);
    let dialogue_active = has_mortar || simple_text_active;
    facts.set(fre_facts::DIALOGUE_ACTIVE, FactValue::Bool(dialogue_active));
    facts.set(fre_facts::DIALOGUE_HAS_MORTAR, FactValue::Bool(has_mortar));
    fre_event_writer.write(FactEvent::new(fre_facts::DIALOGUE_STARTED));
}
