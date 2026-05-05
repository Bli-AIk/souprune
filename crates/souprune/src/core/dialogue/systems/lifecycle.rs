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
use std::collections::HashSet;

use crate::core::fre_facts;

use super::super::auto_pause::AutoPauseState;

#[derive(Component)]
pub struct DialogueControllerEntity;

/// Internal request to spawn a dialogue controller for a named channel.
///
/// 为命名通道生成对话控制器的内部请求。
#[derive(Message, Debug, Clone)]
pub struct DialogueStartRequest {
    pub channel: String,
    pub mortar_path: Option<String>,
    pub mortar_node: Option<String>,
    pub localized_mortar_path: Option<String>,
    pub has_mortar: bool,
}

fn channel_event<'a>(event_id: &'a str, suffix: &str) -> Option<&'a str> {
    event_id
        .strip_prefix("dialogue:")
        .and_then(|rest| rest.strip_suffix(suffix))
        .filter(|channel| !channel.is_empty())
}

fn channel_ended_event(channel: &str) -> String {
    format!("dialogue:{channel}:ended")
}

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
    controller_query: Query<
        &crate::core::dialogue::DialogueChannel,
        With<DialogueControllerEntity>,
    >,
) {
    for finished in mortar_finished.read() {
        info!(
            "handle_mortar_dialogue_finished_system: Mortar dialogue finished (path: {}, node: {})",
            finished.mortar_path, finished.node
        );
        if let Some(entity) = finished.entity
            && let Ok(channel) = controller_query.get(entity)
        {
            fre_event_writer.write(FactEvent::new(channel_ended_event(&channel.name)));
        }
        fre_event_writer.write(FactEvent::new(fre_facts::DIALOGUE_ENDED));
    }
}

pub fn spawn_dialogue_controller_system(
    mut commands: Commands,
    query: Query<(Entity, &crate::core::dialogue::DialogueChannel), With<DialogueControllerEntity>>,
    mut start_requests: MessageReader<DialogueStartRequest>,
    mut mortar_events: MessageWriter<MortarEvent>,
    mut facts: ResMut<LayeredFactDatabase>,
) {
    for request in start_requests.read() {
        let channel = request.channel.clone();
        let channel_active = facts
            .get_bool(&fre_facts::dialogue_channel_key(&channel, "active"))
            .unwrap_or(false);
        let has_controller_for_channel = query.iter().any(|(_, existing)| existing.name == channel);

        if !channel_active || has_controller_for_channel {
            continue;
        }

        let has_typewriter = facts
            .get_bool(&fre_facts::dialogue_channel_key(&channel, "has_typewriter"))
            .or_else(|| facts.get_bool(fre_facts::DIALOGUE_HAS_TYPEWRITER))
            .unwrap_or(true);
        let has_mortar = facts
            .get_bool(&fre_facts::dialogue_channel_key(&channel, "has_mortar"))
            .unwrap_or(request.has_mortar);

        info!(
            "spawn_dialogue_controller_system: spawning dialogue controller for channel '{}' (mortar={}, typewriter={})",
            channel, has_mortar, has_typewriter
        );

        let mut entity_commands = commands.spawn((
            DialogueControllerEntity,
            crate::core::dialogue::DialogueChannel::new(&channel),
            AutoPauseState::default(),
            super::ghost_text::FloatingTextState::default(),
        ));
        let entity = entity_commands.id();

        if has_mortar {
            let mut controller = super::super::components::MortarController::with_path(
                request.mortar_path.clone().unwrap_or_default(),
            );
            controller.current_node = request.mortar_node.clone();
            entity_commands.insert(controller);
        }

        if has_typewriter {
            let typewriter_speed = facts
                .get_float(&fre_facts::dialogue_channel_key(
                    &channel,
                    "typewriter_speed",
                ))
                .or_else(|| facts.get_float(fre_facts::DIALOGUE_TYPEWRITER_SPEED))
                .map(|n| n as f32)
                .unwrap_or(0.03);
            let typewriter = Typewriter::new("", typewriter_speed);
            entity_commands.insert(typewriter);

            if let Some(voice_path) = facts
                .get_string(&fre_facts::dialogue_channel_key(&channel, "voice"))
                .or_else(|| facts.get_string(fre_facts::DIALOGUE_VOICE))
                && !voice_path.is_empty()
            {
                info!(
                    "spawn_dialogue_controller_system: adding TypewriterVoice with path: '{}'",
                    voice_path
                );
                entity_commands.insert(super::super::components::TypewriterVoice::new(voice_path));
            }
        }

        if request.has_mortar
            && let (Some(localized_path), Some(node)) = (
                request.localized_mortar_path.clone(),
                request.mortar_node.clone(),
            )
        {
            mortar_events.write(MortarEvent::start_node_for(entity, localized_path, node));
        }

        facts.set(
            fre_facts::dialogue_channel_key(&channel, "active"),
            FactValue::Bool(true),
        );
        facts.set(
            fre_facts::dialogue_channel_key(&channel, "finished"),
            FactValue::Bool(false),
        );
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
    query: Query<(Entity, &crate::core::dialogue::DialogueChannel), With<DialogueControllerEntity>>,
    mut facts: ResMut<LayeredFactDatabase>,
    mut mortar_events: MessageWriter<MortarEvent>,
) {
    let mut cleanup_all = false;
    let mut channel_cleanups = HashSet::new();
    for event in fre_events.read() {
        if let Some(channel) =
            channel_event(&event.id.0, ":ended").or_else(|| channel_event(&event.id.0, ":stop"))
        {
            channel_cleanups.insert(channel.to_string());
            continue;
        }

        if event.id.0 == fre_facts::DIALOGUE_ENDED || event.id.0 == fre_facts::DIALOGUE_STOP_PREFIX
        {
            cleanup_all = true;
        }
    }

    if channel_cleanups.is_empty() && !cleanup_all {
        for (entity, channel) in &query {
            let channel_active = facts
                .bypass_change_detection()
                .get_bool(&fre_facts::dialogue_channel_key(&channel.name, "active"))
                .unwrap_or(false);
            if runtime.get_dialogue(entity).is_none() && !channel_active {
                channel_cleanups.insert(channel.name.clone());
            }
        }
    }

    if channel_cleanups.is_empty() && !cleanup_all {
        return;
    }

    info!("despawn_dialogue_controller_system: cleaning up dialogue controllers");
    for (entity, channel) in &query {
        if !cleanup_all && !channel_cleanups.contains(&channel.name) {
            continue;
        }
        facts.set(
            fre_facts::dialogue_channel_key(&channel.name, "has_focus"),
            FactValue::Bool(false),
        );
        facts.set(
            fre_facts::dialogue_channel_key(&channel.name, "active"),
            FactValue::Bool(false),
        );
        facts.set(
            fre_facts::dialogue_channel_key(&channel.name, "has_mortar"),
            FactValue::Bool(false),
        );
        facts.set(
            fre_facts::dialogue_channel_key(&channel.name, "visible"),
            FactValue::Bool(false),
        );
        facts.set(
            fre_facts::dialogue_channel_key(&channel.name, "finished"),
            FactValue::Bool(true),
        );
        mortar_events.write(MortarEvent::stop_dialogue_for(entity));
        commands.entity(entity).despawn();
    }

    let remaining_channels: Vec<String> = query
        .iter()
        .filter(|(_, channel)| !cleanup_all && !channel_cleanups.contains(&channel.name))
        .map(|(_, channel)| channel.name.clone())
        .collect();
    let remaining_has_focus = remaining_channels.iter().any(|channel| {
        facts
            .get_bool(&fre_facts::dialogue_channel_key(channel, "has_focus"))
            .unwrap_or(false)
    });
    let remaining_active = remaining_channels.iter().any(|channel| {
        facts
            .get_bool(&fre_facts::dialogue_channel_key(channel, "active"))
            .unwrap_or(false)
    });
    let remaining_has_mortar = remaining_channels.iter().any(|channel| {
        facts
            .get_bool(&fre_facts::dialogue_channel_key(channel, "has_mortar"))
            .unwrap_or(false)
    });
    facts.set(
        fre_facts::DIALOGUE_HAS_FOCUS,
        FactValue::Bool(remaining_has_focus),
    );
    facts.set(
        fre_facts::DIALOGUE_ACTIVE,
        FactValue::Bool(remaining_active),
    );
    facts.set(
        fre_facts::DIALOGUE_HAS_MORTAR,
        FactValue::Bool(remaining_has_mortar),
    );
}

pub fn has_pending_dialogue_start(facts: Res<LayeredFactDatabase>) -> bool {
    facts
        .get_bool(fre_facts::DIALOGUE_PENDING_START)
        .unwrap_or(false)
}

pub fn handle_pending_dialogue_start_system(
    mut facts: ResMut<LayeredFactDatabase>,
    mut start_request_writer: MessageWriter<DialogueStartRequest>,
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

    let channel = facts
        .bypass_change_detection()
        .get_string(fre_facts::DIALOGUE_PENDING_CHANNEL)
        .map(|s| fre_facts::normalize_dialogue_channel(s).to_string())
        .unwrap_or_else(|| fre_facts::DIALOGUE_DEFAULT_CHANNEL.to_string());
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
        "handle_pending_dialogue_start_system: channel={}, view={:?}, path={:?}, node={:?}",
        channel, pending_view, mortar_path, mortar_node
    );

    facts.set(fre_facts::DIALOGUE_PENDING_START, FactValue::Bool(false));
    facts.set(
        fre_facts::DIALOGUE_PENDING_CHANNEL,
        FactValue::String(fre_facts::DIALOGUE_DEFAULT_CHANNEL.to_string()),
    );
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
        "handle_pending_dialogue_start_system: channel={}, view={:?}, mortar={:?}",
        channel,
        pending_view,
        mortar_path.as_ref().zip(mortar_node.as_ref())
    );

    if let Some(view_path) = pending_view {
        spawn_view_writer.write(crate::core::view::SpawnViewRequest {
            path: view_path,
            mode_scope: None,
            pre_spawn_events: Vec::new(),
            bindings: None,
        });
    }

    let has_mortar = mortar_path.is_some() && mortar_node.is_some();
    let localized_mortar_path = mortar_path.as_ref().map(|path| {
        let config = crate::config::load_config();
        format!("{}/{}/{}", config.game.locales_directory, locale.0, path)
    });

    facts.set(
        fre_facts::dialogue_channel_key(&channel, "active"),
        FactValue::Bool(has_mortar),
    );
    facts.set(
        fre_facts::dialogue_channel_key(&channel, "has_mortar"),
        FactValue::Bool(has_mortar),
    );
    facts.set(
        fre_facts::dialogue_channel_key(&channel, "finished"),
        FactValue::Bool(false),
    );

    start_request_writer.write(DialogueStartRequest {
        channel: channel.clone(),
        mortar_path,
        mortar_node,
        localized_mortar_path,
        has_mortar,
    });

    let dialogue_active = has_mortar;
    facts.set(fre_facts::DIALOGUE_ACTIVE, FactValue::Bool(dialogue_active));
    facts.set(fre_facts::DIALOGUE_HAS_MORTAR, FactValue::Bool(has_mortar));
    fre_event_writer.write(FactEvent::new(fre_facts::DIALOGUE_STARTED));
}

#[cfg(test)]
mod tests {
    use crate::core::dialogue::DialogueChannel;

    #[test]
    fn controller_channel_matches_normalized_name() {
        let channel = DialogueChannel::new("");
        assert_eq!(channel.name, "main");
    }
}
