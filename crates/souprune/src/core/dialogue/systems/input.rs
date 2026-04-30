//! Converts FRE dialogue input events into typewriter and Mortar progression commands.
//!
//! 把 FRE 对话输入事件转换成打字机与 Mortar 运行时的推进命令。
//!
//! Handles how the player advances or interrupts dialogue once a
//! dialogue controller exists. It interprets configured input events, decides
//! whether typewriters are ready, asks Mortar for the next line when needed, and
//! supports force-finishing the visible typewriter text.
//!
//! 负责在对话控制器已经存在时，玩家输入该如何推进或打断对话。
//! 它解释配置好的输入事件，判断打字机是否已经准备好，在需要时向 Mortar 请求
//! 下一段文本，并支持直接把当前打字机文本跳到结尾。

use bevy::prelude::*;
use bevy_ecs_typewriter::{Typewriter, TypewriterState};
use bevy_fact_rule_event::{FactEvent, LayeredFactDatabase};
use bevy_mortar_bond::{MortarEvent, MortarRuntime};

use super::lifecycle::DialogueControllerEntity;
use crate::core::dialogue::config::DialogueInputConfig;
use crate::core::fre_facts;

pub fn has_fact_events(events: MessageReader<FactEvent>) -> bool {
    !events.is_empty()
}

fn stop_event_channel(event_id: &str) -> Option<&str> {
    event_id
        .strip_prefix("dialogue:")
        .and_then(|rest| rest.strip_suffix(":stop"))
        .filter(|channel| !channel.is_empty())
}

pub fn dialogue_advance_system(
    mut fre_events: MessageReader<FactEvent>,
    config: Res<DialogueInputConfig>,
    mut mortar_events: MessageWriter<MortarEvent>,
    facts: Res<LayeredFactDatabase>,
    query: Query<
        (
            Entity,
            &crate::core::dialogue::DialogueChannel,
            Option<&Typewriter>,
        ),
        With<DialogueControllerEntity>,
    >,
    runtime: Res<MortarRuntime>,
) {
    for event in fre_events.read() {
        trace!("dialogue_advance_system: received event '{}'", event.id.0);

        if event.id.0 != config.advance_event {
            continue;
        }

        info!(
            "dialogue_advance_system: matched '{}', checking runtime state",
            config.advance_event
        );

        for (entity, channel, typewriter) in query.iter() {
            let has_focus = facts
                .get_bool(&fre_facts::dialogue_channel_key(&channel.name, "has_focus"))
                .or_else(|| facts.get_bool(fre_facts::DIALOGUE_HAS_FOCUS))
                .unwrap_or(false);
            if !has_focus {
                continue;
            }

            if runtime.get_dialogue(entity).is_none() {
                continue;
            }

            let Some(typewriter) = typewriter else {
                info!(
                    "dialogue_advance_system: no typewriter for channel '{}', sending NextText",
                    channel.name
                );
                mortar_events.write(MortarEvent::next_text_for(entity));
                continue;
            };

            let ready = typewriter.state == TypewriterState::Finished
                || typewriter.state == TypewriterState::Idle;
            if !ready {
                debug!(
                    "Dialogue advance blocked for channel '{}': typewriter not ready",
                    channel.name
                );
                continue;
            }

            mortar_events.write(MortarEvent::next_text_for(entity));
        }
    }
}

pub fn dialogue_skip_typewriter_system(
    mut fre_events: MessageReader<FactEvent>,
    config: Res<DialogueInputConfig>,
    facts: Res<LayeredFactDatabase>,
    mut query: Query<
        (&crate::core::dialogue::DialogueChannel, &mut Typewriter),
        With<DialogueControllerEntity>,
    >,
) {
    for event in fre_events.read() {
        debug!(
            "dialogue_skip_typewriter_system: received event '{}', expecting '{}'",
            event.id.0, config.skip_typewriter_event
        );
        if event.id.0 != config.skip_typewriter_event {
            continue;
        }

        info!("dialogue_skip_typewriter_system: processing skip event");

        let typewriter_count = query.iter().count();
        debug!(
            "dialogue_skip_typewriter_system: found {} typewriters",
            typewriter_count
        );

        for (channel, mut typewriter) in &mut query {
            let has_focus = facts
                .get_bool(&fre_facts::dialogue_channel_key(&channel.name, "has_focus"))
                .or_else(|| facts.get_bool(fre_facts::DIALOGUE_HAS_FOCUS))
                .unwrap_or(false);
            if !has_focus {
                continue;
            }
            debug!(
                "dialogue_skip_typewriter_system: typewriter state = {:?}",
                typewriter.state
            );
            if typewriter.state == TypewriterState::Playing
                || typewriter.state == TypewriterState::Paused
            {
                typewriter.current_text = typewriter.source_text.clone();
                typewriter.current_char_index = typewriter.source_text.chars().count();
                typewriter.state = TypewriterState::Finished;
                info!("Typewriter skipped to end");
            }
        }
    }
}

pub fn handle_dialogue_stop_event_system(
    mut events: MessageReader<FactEvent>,
    mut typewriter_query: Query<
        (
            Entity,
            &crate::core::dialogue::DialogueChannel,
            &mut Typewriter,
        ),
        With<DialogueControllerEntity>,
    >,
    mut mortar_events: MessageWriter<MortarEvent>,
) {
    for event in events.read() {
        let channel_filter = stop_event_channel(&event.id.0);
        let stop_all = event.id.0 == fre_facts::DIALOGUE_STOP_PREFIX;
        if !stop_all && channel_filter.is_none() {
            continue;
        }

        info!(
            "handle_dialogue_stop_event: stopping dialogue '{}'",
            event.id.0
        );
        for (entity, channel, mut typewriter) in typewriter_query.iter_mut() {
            if stop_all || channel_filter == Some(channel.name.as_str()) {
                typewriter.stop();
                mortar_events.write(MortarEvent::stop_dialogue_for(entity));
            }
        }
    }
}
