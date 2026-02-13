//! Dialogue system core systems.
//!
//! 对话系统核心系统。

use bevy::prelude::*;
use bevy_ecs_typewriter::{Typewriter, TypewriterState};
use bevy_fact_rule_event::{FactEvent, FactValue, LayeredFactDatabase};
use bevy_mortar_bond::{MortarEvent, MortarRuntime};

use super::components::{DialogueFocus, MortarController};
use super::config::{DialogueBlockingConfig, DialogueInputConfig};

/// Syncs focused Typewriter state to FRE Facts.
///
/// 将有焦点的 Typewriter 状态同步到 FRE Facts。
///
/// Updates `dialogue:typewriter_playing` fact based on whether any focused
/// typewriter is currently playing.
///
/// 根据是否有任何焦点打字机正在播放，更新 `dialogue:typewriter_playing` fact。
pub fn sync_typewriter_state_to_facts_system(
    query: Query<&Typewriter, (With<MortarController>, With<DialogueFocus>)>,
    mut facts: ResMut<LayeredFactDatabase>,
) {
    let any_playing = query.iter().any(|tw| tw.state == TypewriterState::Playing);

    // Update the fact
    facts.set("dialogue:typewriter_playing", FactValue::Bool(any_playing));
}

/// Handles dialogue advancement on FRE events.
///
/// 处理 FRE 事件触发的对话步进。
///
/// Listens for the configured advance event and sends `MortarEvent::NextText`
/// when all focused typewriters are finished (respecting blocking config).
///
/// 监听配置的步进事件，当所有焦点打字机完成时
/// （遵循阻塞配置）发送 `MortarEvent::NextText`。
pub fn dialogue_advance_system(
    mut fre_events: MessageReader<FactEvent>,
    config: Res<DialogueInputConfig>,
    blocking_config: Res<DialogueBlockingConfig>,
    mut mortar_events: MessageWriter<MortarEvent>,
    query: Query<&Typewriter, (With<MortarController>, With<DialogueFocus>)>,
    runtime: Res<MortarRuntime>,
) {
    for event in fre_events.read() {
        if event.id.0 != config.advance_event {
            continue;
        }

        // Check if there's an active dialogue
        let Some(_state) = &runtime.active_dialogue else {
            continue;
        };

        // Check if focused typewriters are ready
        let typewriters: Vec<_> = query.iter().collect();

        // If no typewriters exist (no-typewriter dialogue), allow advancement
        if typewriters.is_empty() {
            mortar_events.write(MortarEvent::NextText);
            continue;
        }

        let all_ready = if blocking_config.require_all_finished {
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
                "Dialogue advance blocked: typewriters not ready (blocking_mode: {})",
                if blocking_config.require_all_finished {
                    "all"
                } else {
                    "any"
                }
            );
            continue;
        }

        mortar_events.write(MortarEvent::NextText);
    }
}

/// Handles typewriter skipping on FRE events.
///
/// 处理 FRE 事件触发的打字机跳过。
///
/// Listens for the configured skip event and immediately finishes
/// all focused typewriters that are currently playing.
///
/// 监听配置的跳过事件，立即完成所有正在播放的焦点打字机。
pub fn dialogue_skip_typewriter_system(
    mut fre_events: MessageReader<FactEvent>,
    config: Res<DialogueInputConfig>,
    mut query: Query<&mut Typewriter, (With<MortarController>, With<DialogueFocus>)>,
) {
    for event in fre_events.read() {
        if event.id.0 != config.skip_typewriter_event {
            continue;
        }

        for mut typewriter in &mut query {
            if typewriter.state == TypewriterState::Playing {
                // Skip to end - show all text immediately
                typewriter.current_text = typewriter.source_text.clone();
                typewriter.current_char_index = typewriter.source_text.chars().count();
                typewriter.state = TypewriterState::Finished;
                debug!("Typewriter skipped to end");
            }
        }
    }
}
