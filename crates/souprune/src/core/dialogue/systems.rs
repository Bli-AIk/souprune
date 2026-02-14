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
/// typewriter is currently playing. Only updates when the value actually changes.
///
/// 根据是否有任何焦点打字机正在播放，更新 `dialogue:typewriter_playing` fact。
/// 仅在值实际变化时更新。
pub fn sync_typewriter_state_to_facts_system(
    query: Query<&Typewriter, (With<MortarController>, With<DialogueFocus>)>,
    mut facts: ResMut<LayeredFactDatabase>,
) {
    let any_playing = query.iter().any(|tw| tw.state == TypewriterState::Playing);

    // Only update if value actually changed to avoid triggering unnecessary View updates
    // Use bypass_change_detection to read without marking as changed
    let current = facts
        .bypass_change_detection()
        .get_bool("dialogue_typewriter_playing")
        .unwrap_or(false);
    if current != any_playing {
        facts.set("dialogue_typewriter_playing", FactValue::Bool(any_playing));
    }
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
        // Debug: log all events to see what's being received
        trace!("dialogue_advance_system: received event '{}'", event.id.0);

        if event.id.0 != config.advance_event {
            continue;
        }

        info!(
            "dialogue_advance_system: matched '{}', checking runtime state",
            config.advance_event
        );

        // Check if there's an active dialogue
        let Some(_state) = &runtime.active_dialogue else {
            info!("dialogue_advance_system: no active dialogue, skipping");
            continue;
        };

        // Check if focused typewriters are ready
        let typewriters: Vec<_> = query.iter().collect();

        // If no typewriters exist (no-typewriter dialogue), allow advancement
        if typewriters.is_empty() {
            info!("dialogue_advance_system: no typewriters, sending NextText");
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

/// Syncs Mortar dialogue text to Typewriter component.
///
/// 将 Mortar 对话文本同步到 Typewriter 组件。
///
/// When Mortar produces new text, updates the Typewriter's source_text
/// and starts playback. This enables the typewriter effect for dialogue.
///
/// 当 Mortar 产生新文本时，更新 Typewriter 的 source_text 并启动播放。
/// 这为对话启用打字机效果。
pub fn sync_mortar_text_to_typewriter_system(
    runtime: Res<bevy_mortar_bond::MortarRuntime>,
    mut query: Query<&mut Typewriter, With<DialogueFocus>>,
) {
    let Some(state) = &runtime.active_dialogue else {
        return;
    };

    let new_text = state.current_text().unwrap_or("");

    for mut typewriter in &mut query {
        // Only update if the source text changed
        if typewriter.source_text != new_text {
            info!(
                "sync_mortar_text_to_typewriter: updating source_text to '{}'",
                new_text
            );
            typewriter.source_text = new_text.to_string();
            typewriter.current_text.clear();
            typewriter.current_char_index = 0;
            typewriter.play();
        }
    }
}

/// Syncs Typewriter current_text to FRE facts for View text binding.
///
/// 将 Typewriter 的 current_text 同步到 FRE facts 用于 View 文本绑定。
///
/// Updates `dialogue_text` fact with the typewriter's current displayed text.
/// Views can reference this with `{{dialogue_text}}` in their text templates.
///
/// 使用打字机当前显示的文本更新 `dialogue_text` fact。
/// View 可在文本模板中使用 `{{dialogue_text}}` 引用。
pub fn sync_typewriter_text_to_facts_system(
    runtime: Res<bevy_mortar_bond::MortarRuntime>,
    query: Query<&Typewriter, With<DialogueFocus>>,
    mut facts: ResMut<LayeredFactDatabase>,
) {
    // Get the first focused typewriter's current text
    // 获取第一个焦点打字机的当前文本
    let typewriter_count = query.iter().count();

    // If no typewriter exists but dialogue is active, use Mortar text directly
    // This handles the first frame before Typewriter entity is spawned
    // 如果没有打字机但对话已激活，直接使用 Mortar 文本
    // 这处理了 Typewriter 实体创建前的第一帧
    let new_text = if typewriter_count > 0 {
        query
            .iter()
            .next()
            .map(|tw| tw.current_text.clone())
            .unwrap_or_default()
    } else if let Some(state) = &runtime.active_dialogue {
        // Fallback to Mortar runtime text if no typewriter yet
        // 如果还没有打字机，回退到 Mortar runtime 文本
        state.current_text().unwrap_or("").to_string()
    } else {
        return; // No dialogue active, nothing to sync
    };

    // Debug: log typewriter state
    if typewriter_count > 0 || runtime.active_dialogue.is_some() {
        trace!(
            "sync_typewriter_text_to_facts: {} typewriters, dialogue_active={}, text='{}'",
            typewriter_count,
            runtime.active_dialogue.is_some(),
            new_text
        );
    }

    // Only update if text actually changed
    let current = facts
        .bypass_change_detection()
        .get_string("dialogue_text")
        .map(|s| s.to_string())
        .unwrap_or_default();

    if current != new_text {
        info!(
            "sync_typewriter_text_to_facts: updating dialogue_text fact: '{}' -> '{}'",
            current, new_text
        );
        facts.set("dialogue_text", FactValue::String(new_text));
    }
}

/// Marker component for the dialogue controller entity.
///
/// 对话控制器实体的标记组件。
#[derive(Component)]
pub struct DialogueControllerEntity;

/// System to spawn a dialogue controller entity when dialogue starts.
///
/// 当对话启动时生成对话控制器实体的系统。
///
/// Creates an entity with optional Typewriter + MortarController + DialogueFocus
/// based on ActiveDialogueState configuration.
///
/// 根据 ActiveDialogueState 配置，创建一个带有可选 Typewriter + MortarController + DialogueFocus 的实体。
pub fn spawn_dialogue_controller_system(
    mut commands: Commands,
    runtime: Res<MortarRuntime>,
    active_state: Res<super::config::ActiveDialogueState>,
    query: Query<Entity, With<DialogueControllerEntity>>,
    mut facts: ResMut<LayeredFactDatabase>,
) {
    let has_controller = !query.is_empty();
    let has_dialogue = runtime.active_dialogue.is_some() || active_state.simple_text.is_some();

    // Spawn controller when dialogue starts
    if has_dialogue && !has_controller {
        info!(
            "spawn_dialogue_controller_system: spawning dialogue controller (mortar={}, typewriter={})",
            active_state.has_mortar, active_state.has_typewriter
        );

        let mut entity_commands = commands.spawn((DialogueControllerEntity, DialogueFocus));

        // Add MortarController if configured
        if active_state.has_mortar {
            entity_commands.insert(MortarController::new());
        }

        // Add Typewriter if configured
        if active_state.has_typewriter {
            entity_commands.insert(Typewriter::new("", 0.03)); // 30ms per character
        }

        // For simple text without Mortar, set the fact directly
        if !active_state.has_mortar {
            if let Some(ref text) = active_state.simple_text {
                info!(
                    "spawn_dialogue_controller_system: setting simple_text to fact: '{}'",
                    text
                );
                facts.set("dialogue_text", FactValue::String(text.clone()));
            }
        }
    }
}

/// System to despawn dialogue controller entity and cleanup when dialogue ends.
///
/// 当对话结束时销毁对话控制器实体并清理的系统。
///
/// This system:
/// 1. Despawns the dialogue controller entity
/// 2. Clears ActiveDialogueState
/// 3. Restores Overworld state to Normal
/// 4. Despawns dialogue Views
///
/// 此系统：
/// 1. 销毁对话控制器实体
/// 2. 清理 ActiveDialogueState
/// 3. 恢复 Overworld 状态为 Normal
/// 4. 销毁对话 View
pub fn despawn_dialogue_controller_system(
    mut commands: Commands,
    runtime: Res<MortarRuntime>,
    mut active_state: ResMut<super::config::ActiveDialogueState>,
    query: Query<Entity, With<DialogueControllerEntity>>,
    view_query: Query<Entity, With<crate::core::view::components::ViewRoot>>,
    current_state: Res<State<crate::app_state::overworld::OverworldSubState>>,
    mut next_state: ResMut<NextState<crate::app_state::overworld::OverworldSubState>>,
) {
    // Don't despawn if dialogue is still active
    let mortar_active = runtime.active_dialogue.is_some();
    let simple_active = active_state.simple_text.is_some() && !active_state.has_mortar;

    // Only trigger cleanup if:
    // 1. Controller exists (dialogue was started)
    // 2. Neither Mortar nor simple dialogue is active (dialogue ended)
    let has_controller = !query.is_empty();
    let should_cleanup = has_controller && !mortar_active && !simple_active;

    if !should_cleanup {
        return;
    }

    info!("despawn_dialogue_controller_system: dialogue ended, cleaning up");

    // 1. Despawn controller entity
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }

    // 2. Clear active state
    *active_state = super::config::ActiveDialogueState::default();

    // 3. Restore Overworld state if in Dialogue state
    if current_state.0 == "Dialogue" {
        info!("despawn_dialogue_controller_system: restoring Overworld state to Normal");
        next_state.set(crate::app_state::overworld::OverworldSubState::default());
    }

    // 4. Despawn all Views (dialogue Views will be cleaned up)
    // Note: This is a simple approach - a more refined approach would track which View
    // was spawned for dialogue and only despawn that one
    for view_entity in view_query.iter() {
        info!(
            "despawn_dialogue_controller_system: despawning view entity {:?}",
            view_entity
        );
        commands.entity(view_entity).despawn();
    }
}
