//! Converts FRE dialogue input events into typewriter and Mortar progression commands.
//!
//! 把 FRE 对话输入事件转换成打字机与 Mortar 运行时的推进命令。
//!
//! This file handles how the player advances or interrupts dialogue once a
//! dialogue controller exists. It interprets configured input events, decides
//! whether typewriters are ready, asks Mortar for the next line when needed, and
//! supports force-finishing the visible typewriter text.
//!
//! 这个文件负责在对话控制器已经存在时，玩家输入该如何推进或打断对话。
//! 它解释配置好的输入事件，判断打字机是否已经准备好，在需要时向 Mortar 请求
//! 下一段文本，并支持直接把当前打字机文本跳到结尾。

use bevy::prelude::*;
use bevy_ecs_typewriter::{Typewriter, TypewriterState};
use bevy_fact_rule_event::{FactEvent, FactValue, LayeredFactDatabase};
use bevy_mortar_bond::{MortarEvent, MortarRuntime};

use super::lifecycle::DialogueControllerEntity;
use crate::core::dialogue::config::DialogueInputConfig;
use crate::core::fre_facts;

pub fn has_fact_events(events: MessageReader<FactEvent>) -> bool {
    !events.is_empty()
}

pub fn dialogue_advance_system(
    mut fre_events: MessageReader<FactEvent>,
    config: Res<DialogueInputConfig>,
    mut mortar_events: MessageWriter<MortarEvent>,
    mut facts: ResMut<LayeredFactDatabase>,
    query: Query<&Typewriter, With<DialogueControllerEntity>>,
    runtime: Res<MortarRuntime>,
) {
    for event in fre_events.read() {
        trace!("dialogue_advance_system: received event '{}'", event.id.0);

        if event.id.0 != config.advance_event {
            continue;
        }

        let has_focus = facts
            .get_bool(fre_facts::DIALOGUE_HAS_FOCUS)
            .unwrap_or(false);
        if !has_focus {
            debug!("dialogue_advance_system: dialogue:has_focus is false, skipping");
            continue;
        }

        info!(
            "dialogue_advance_system: matched '{}', checking runtime state",
            config.advance_event
        );

        let mortar_active = runtime.has_active_dialogues();
        let simple_active = facts
            .get_bool(fre_facts::DIALOGUE_SIMPLE_TEXT_ACTIVE)
            .unwrap_or(false);

        if !mortar_active && !simple_active {
            info!("dialogue_advance_system: no active dialogue, skipping");
            continue;
        }

        let typewriters: Vec<_> = query.iter().collect();
        if typewriters.is_empty() {
            if mortar_active {
                info!("dialogue_advance_system: no typewriters, sending NextText");
                mortar_events.write(MortarEvent::next_text());
            } else {
                info!(
                    "dialogue_advance_system: simple text (no typewriter), marking dialogue ended"
                );
                facts.set(fre_facts::DIALOGUE_PENDING_ENDED, FactValue::Bool(true));
            }
            continue;
        }

        let focus_mode = facts
            .get_string(fre_facts::DIALOGUE_FOCUS_MODE)
            .unwrap_or("all_finished");
        let require_all_finished = focus_mode == "all_finished";

        let all_ready = if require_all_finished {
            typewriters.iter().all(|tw| {
                tw.state == TypewriterState::Finished || tw.state == TypewriterState::Idle
            })
        } else {
            typewriters.iter().any(|tw| {
                tw.state == TypewriterState::Finished || tw.state == TypewriterState::Idle
            })
        };

        if !all_ready {
            debug!(
                "Dialogue advance blocked: typewriters not ready (focus_mode: {})",
                focus_mode
            );
            continue;
        }

        if mortar_active {
            mortar_events.write(MortarEvent::next_text());
        } else {
            info!("dialogue_advance_system: simple text finished, marking dialogue ended");
            facts.set(fre_facts::DIALOGUE_PENDING_ENDED, FactValue::Bool(true));
        }
    }
}

pub fn dialogue_skip_typewriter_system(
    mut fre_events: MessageReader<FactEvent>,
    config: Res<DialogueInputConfig>,
    mut query: Query<&mut Typewriter, With<DialogueControllerEntity>>,
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

        for mut typewriter in &mut query {
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
    mut typewriter_query: Query<&mut Typewriter, With<DialogueControllerEntity>>,
) {
    for event in events.read() {
        if event.id.0.starts_with(fre_facts::DIALOGUE_STOP_PREFIX) {
            info!("handle_dialogue_stop_event: stopping all typewriters");
            for mut typewriter in typewriter_query.iter_mut() {
                typewriter.stop();
            }
        }
    }
}
