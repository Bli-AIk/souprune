//! Dialogue system core systems.
//!
//! 对话系统核心系统。

use bevy::prelude::*;
use bevy_ecs_typewriter::{Typewriter, TypewriterState};
use bevy_fact_rule_event::{FactEvent, FactValue, LayeredFactDatabase};
use bevy_mortar_bond::{MortarEvent, MortarRuntime};

use super::components::MortarController;
use super::config::DialogueInputConfig;

/// Syncs focused Typewriter state to FRE Facts.
///
/// 将有焦点的 Typewriter 状态同步到 FRE Facts。
///
/// Updates the following FRE facts based on focused typewriter states:
/// - `dialogue:typewriter_playing`: true if any typewriter is playing
/// - `dialogue:all_typewriters_finished`: true if ALL focused typewriters are finished
/// - `dialogue:any_typewriter_finished`: true if ANY focused typewriter is finished
///
/// 根据焦点打字机状态更新以下 FRE facts：
/// - `dialogue:typewriter_playing`：任一打字机正在播放为 true
/// - `dialogue:all_typewriters_finished`：所有焦点打字机完成为 true
/// - `dialogue:any_typewriter_finished`：任一焦点打字机完成为 true
pub fn sync_typewriter_state_to_facts_system(
    query: Query<&Typewriter, With<DialogueControllerEntity>>,
    mut facts: ResMut<LayeredFactDatabase>,
) {
    // Calculate states based on focused typewriters
    let mut any_playing = false;
    let mut all_finished = true;
    let mut any_finished = false;
    let mut has_typewriters = false;

    for tw in query.iter() {
        has_typewriters = true;
        match tw.state {
            TypewriterState::Playing => {
                any_playing = true;
                all_finished = false;
            }
            TypewriterState::Finished => {
                any_finished = true;
            }
            TypewriterState::Idle => {
                // Idle state counts as not playing but not finished either
                // 空闲状态算作未播放但也未完成
            }
            TypewriterState::Paused => {
                // Paused is similar to playing - not finished yet
                // 暂停类似于播放 - 尚未完成
                all_finished = false;
            }
        }
    }

    // If no typewriters exist, consider all finished
    if !has_typewriters {
        all_finished = true;
        any_finished = true;
    }

    // Helper to update fact only if value changed
    // 辅助函数：仅当值变化时更新 fact
    let update_fact = |facts: &mut ResMut<LayeredFactDatabase>, key: &str, value: bool| {
        let current = facts.bypass_change_detection().get_bool(key).unwrap_or(false);
        if current != value {
            facts.set(key, FactValue::Bool(value));
        }
    };

    update_fact(&mut facts, "dialogue:typewriter_playing", any_playing);
    update_fact(&mut facts, "dialogue:all_typewriters_finished", all_finished);
    update_fact(&mut facts, "dialogue:any_typewriter_finished", any_finished);
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
/// Handles dialogue advancement on FRE events.
///
/// 处理 FRE 事件触发的对话步进。
///
/// Listens for the configured advance event and sends `MortarEvent::NextText`
/// when focused typewriters are ready (based on `dialogue:focus_mode` fact).
/// When dialogue ends, emits `dialogue:ended` FRE event for rule-based handling.
///
/// 监听配置的步进事件，当焦点打字机准备就绪时
/// （基于 `dialogue:focus_mode` fact）发送 `MortarEvent::NextText`。
/// 对话结束时发出 `dialogue:ended` FRE 事件供规则处理。
pub fn dialogue_advance_system(
    mut fre_events: MessageReader<FactEvent>,
    config: Res<DialogueInputConfig>,
    mut mortar_events: MessageWriter<MortarEvent>,
    mut fre_event_writer: MessageWriter<FactEvent>,
    query: Query<&Typewriter, With<DialogueControllerEntity>>,
    runtime: Res<MortarRuntime>,
    facts: Res<LayeredFactDatabase>,
) {
    for event in fre_events.read() {
        // Debug: log all events to see what's being received
        trace!("dialogue_advance_system: received event '{}'", event.id.0);

        if event.id.0 != config.advance_event {
            continue;
        }

        // Check if dialogue has focus (FRE fact replaces DialogueFocus component)
        // 检查对话是否有焦点（FRE fact 替代 DialogueFocus 组件）
        let has_focus = facts.get_bool("dialogue:has_focus").unwrap_or(false);
        if !has_focus {
            debug!("dialogue_advance_system: dialogue:has_focus is false, skipping");
            continue;
        }

        info!(
            "dialogue_advance_system: matched '{}', checking runtime state",
            config.advance_event
        );

        // Check if there's an active dialogue
        // Use FRE fact to check simple text dialogue (data-driven)
        let mortar_active = runtime.has_active_dialogues();
        let simple_active = facts
            .get_bool("dialogue:simple_text_active")
            .unwrap_or(false);

        if !mortar_active && !simple_active {
            info!("dialogue_advance_system: no active dialogue, skipping");
            continue;
        }

        // Check if focused typewriters are ready
        let typewriters: Vec<_> = query.iter().collect();

        // If no typewriters exist (no-typewriter dialogue), allow advancement
        if typewriters.is_empty() {
            if mortar_active {
                info!("dialogue_advance_system: no typewriters, sending NextText");
                mortar_events.write(MortarEvent::next_text());
            } else {
                // Simple text without typewriter - emit dialogue:ended event
                info!("dialogue_advance_system: simple text (no typewriter), emitting dialogue:ended");
                fre_event_writer.write(FactEvent::new("dialogue:ended"));
            }
            continue;
        }

        // Get focus mode from FRE fact (data-driven configuration)
        // "all_finished" - all typewriters must be finished before advancing
        // "first_finished" - any typewriter finished allows advancement
        // 从 FRE fact 获取焦点模式（数据驱动配置）
        // "all_finished" - 所有打字机必须完成才能步进
        // "first_finished" - 任一打字机完成即可步进
        let focus_mode = facts
            .get_string("dialogue:focus_mode")
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
            // Simple text with finished typewriter - emit dialogue:ended event
            info!("dialogue_advance_system: simple text finished, emitting dialogue:ended");
            fre_event_writer.write(FactEvent::new("dialogue:ended"));
        }
    }
}

/// Handles typewriter skipping on FRE events.
///
/// 处理 FRE 事件触发的打字机跳过。
///
/// Listens for the configured skip event and immediately finishes
/// all typewriters on DialogueControllerEntity that are currently playing.
/// Only responds when `dialogue:has_focus` is true.
///
/// 监听配置的跳过事件，立即完成 DialogueControllerEntity 上所有正在播放的打字机。
/// 仅当 `dialogue:has_focus` 为 true 时响应。
pub fn dialogue_skip_typewriter_system(
    mut fre_events: MessageReader<FactEvent>,
    config: Res<DialogueInputConfig>,
    mut query: Query<&mut Typewriter, With<DialogueControllerEntity>>,
    facts: Res<LayeredFactDatabase>,
) {
    for event in fre_events.read() {
        if event.id.0 != config.skip_typewriter_event {
            continue;
        }

        // Check if dialogue has focus
        // 检查对话是否有焦点
        let has_focus = facts.get_bool("dialogue:has_focus").unwrap_or(false);
        if !has_focus {
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
    mut query: Query<&mut Typewriter, With<DialogueControllerEntity>>,
) {
    let Some(state) = runtime.primary_dialogue_state() else {
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
    query: Query<&Typewriter, With<DialogueControllerEntity>>,
    mut facts: ResMut<LayeredFactDatabase>,
) {
    // Get the first focused typewriter's current text
    // 获取第一个焦点打字机的当前文本
    let typewriter_count = query.iter().count();

    // Check if simple text dialogue is active via FRE fact (data-driven)
    let simple_text_active = facts
        .bypass_change_detection()
        .get_bool("dialogue:simple_text_active")
        .unwrap_or(false);

    // Determine what text to use
    let new_text = if typewriter_count > 0 {
        // Use typewriter's current text (which progresses over time)
        query
            .iter()
            .next()
            .map(|tw| tw.current_text.clone())
            .unwrap_or_default()
    } else if let Some(state) = runtime.primary_dialogue_state() {
        // No typewriter - use Mortar text directly
        // 无打字机 - 直接使用 Mortar 文本
        state.current_text().unwrap_or("").to_string()
    } else if simple_text_active {
        // Simple text dialogue active but no typewriter found
        // Read from the fact itself (already set by caller)
        // 简单文本对话激活但没有找到打字机 - 从 fact 读取（已由调用方设置）
        return;
    } else {
        return; // No dialogue active, nothing to sync
    };

    // Debug: log typewriter state
    if typewriter_count > 0 || runtime.has_active_dialogues() {
        trace!(
            "sync_typewriter_text_to_facts: {} typewriters, dialogue_active={}, text='{}'",
            typewriter_count,
            runtime.has_active_dialogues(),
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
/// Creates an entity with optional Typewriter + MortarController
/// based on FRE facts configuration (data-driven). Sets `dialogue:has_focus` to true.
///
/// 根据 FRE facts 配置（数据驱动），创建一个带有可选 Typewriter + MortarController 的实体。
/// 同时设置 `dialogue:has_focus` 为 true。
pub fn spawn_dialogue_controller_system(
    mut commands: Commands,
    runtime: Res<MortarRuntime>,
    query: Query<Entity, With<DialogueControllerEntity>>,
    mut facts: ResMut<LayeredFactDatabase>,
) {
    let has_controller = !query.is_empty();

    // Check if dialogue should be active via FRE facts
    let mortar_active = runtime.has_active_dialogues();
    let simple_text_active = facts
        .bypass_change_detection()
        .get_bool("dialogue:simple_text_active")
        .unwrap_or(false);
    let has_dialogue = mortar_active || simple_text_active;

    // Configuration from FRE facts
    let has_typewriter = facts
        .bypass_change_detection()
        .get_bool("dialogue:has_typewriter")
        .unwrap_or(true); // Default to true for backward compatibility
    let has_mortar = mortar_active;

    // Spawn controller when dialogue starts
    if has_dialogue && !has_controller {
        info!(
            "spawn_dialogue_controller_system: spawning dialogue controller (mortar={}, typewriter={})",
            has_mortar, has_typewriter
        );

        // Set dialogue:has_focus to true when spawning controller
        // 生成控制器时设置 dialogue:has_focus 为 true
        facts.set("dialogue:has_focus", FactValue::Bool(true));

        let mut entity_commands = commands.spawn(DialogueControllerEntity);

        // Add MortarController if Mortar dialogue is active
        if has_mortar {
            entity_commands.insert(MortarController::new());
        }

        // Add Typewriter if configured
        if has_typewriter {
            // Get simple text from FRE fact if available
            let simple_text = facts
                .bypass_change_detection()
                .get_string("dialogue:simple_text")
                .map(|s| s.to_string());

            // For simple text (no Mortar), initialize typewriter with the text directly
            // For Mortar dialogues, start empty - sync_mortar_text_to_typewriter_system will fill it
            let initial_text = if !has_mortar {
                simple_text.unwrap_or_default()
            } else {
                String::new()
            };

            let mut typewriter = Typewriter::new(&initial_text, 0.03); // 30ms per character
            if !initial_text.is_empty() {
                typewriter.play();
                info!(
                    "spawn_dialogue_controller_system: starting typewriter with simple_text: '{}'",
                    initial_text
                );
            }
            entity_commands.insert(typewriter);
        }

        // For simple text without Typewriter, set the fact directly
        // If there's a typewriter, sync_typewriter_text_to_facts_system will handle it
        if !has_mortar && !has_typewriter {
            if let Some(text) = facts
                .bypass_change_detection()
                .get_string("dialogue:simple_text")
            {
                info!(
                    "spawn_dialogue_controller_system: setting simple_text to fact: '{}'",
                    text
                );
                let text_owned = text.to_string();
                facts.set("dialogue_text", FactValue::String(text_owned));
            }
        }
    }
}

/// System to despawn dialogue controller entity when dialogue ends.
///
/// 当对话结束时销毁对话控制器实体的系统。
///
/// This system only handles cleanup when `dialogue:ended` event is received
/// or when no dialogue is active. State changes are handled by FRE rules.
///
/// 此系统仅在收到 `dialogue:ended` 事件或没有活跃对话时处理清理。
/// 状态变化由 FRE 规则处理。
pub fn despawn_dialogue_controller_system(
    mut commands: Commands,
    mut fre_events: MessageReader<FactEvent>,
    runtime: Res<MortarRuntime>,
    query: Query<Entity, With<DialogueControllerEntity>>,
    mut facts: ResMut<LayeredFactDatabase>,
) {
    // Listen for dialogue:ended event
    let mut should_cleanup = false;
    for event in fre_events.read() {
        if event.id.0 == "dialogue:ended" {
            should_cleanup = true;
            break;
        }
    }

    // Also cleanup if controller exists but no dialogue is active
    if !should_cleanup {
        let mortar_active = runtime.has_active_dialogues();
        let simple_active = facts
            .bypass_change_detection()
            .get_bool("dialogue:simple_text_active")
            .unwrap_or(false);
        let has_controller = !query.is_empty();
        should_cleanup = has_controller && !mortar_active && !simple_active;
    }

    if !should_cleanup {
        return;
    }

    info!("despawn_dialogue_controller_system: dialogue ended, cleaning up controller");

    // Reset dialogue:has_focus when cleaning up
    // 清理时重置 dialogue:has_focus
    facts.set("dialogue:has_focus", FactValue::Bool(false));

    // Despawn controller entity
    // Note: View cleanup should be handled by FRE rules with DespawnView action
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }
}

/// Handles pending dialogue start requests from FRE facts.
///
/// 处理来自 FRE facts 的待处理对话启动请求。
///
/// This system monitors `dialogue:pending_view` fact and other pending dialogue facts.
/// When view path is set (non-empty), it:
/// - Spawns the dialogue view
/// - Starts Mortar dialogue if pending_mortar_path and pending_mortar_node are set
/// - Emits `dialogue:started` FRE event for scene-specific handling (e.g., state change)
/// - Clears all pending facts
///
/// 此系统监控 `dialogue:pending_view` fact 及其他待处理对话 facts。
/// 当视图路径设置（非空）时：
/// - 生成对话视图
/// - 如果 pending_mortar_path 和 pending_mortar_node 设置，启动 Mortar 对话
/// - 发出 `dialogue:started` FRE 事件用于场景特定处理（如状态切换）
/// - 清除所有待处理 facts
pub fn handle_pending_dialogue_start_system(
    mut facts: ResMut<LayeredFactDatabase>,
    mut mortar_events: MessageWriter<MortarEvent>,
    mut spawn_view_writer: MessageWriter<crate::core::view::SpawnViewRequest>,
    mut fre_event_writer: MessageWriter<FactEvent>,
    locale: Res<crate::extra::mortar::CurrentLocale>,
) {
    // Check for pending view (required to trigger dialogue startup)
    let pending_view = facts
        .bypass_change_detection()
        .get_string("dialogue:pending_view")
        .map(|s| s.to_string())
        .filter(|s| !s.is_empty());

    let Some(view_path) = pending_view else {
        return;
    };

    // Read other pending facts
    let mortar_path = facts
        .bypass_change_detection()
        .get_string("dialogue:pending_mortar_path")
        .map(|s| s.to_string())
        .filter(|s| !s.is_empty());
    let mortar_node = facts
        .bypass_change_detection()
        .get_string("dialogue:pending_mortar_node")
        .map(|s| s.to_string())
        .filter(|s| !s.is_empty());

    // Clear all pending facts
    facts.set(
        "dialogue:pending_view",
        FactValue::String(String::new()),
    );
    facts.set(
        "dialogue:pending_mortar_path",
        FactValue::String(String::new()),
    );
    facts.set(
        "dialogue:pending_mortar_node",
        FactValue::String(String::new()),
    );

    // Clear dialogue_text fact to prevent flicker when using typewriter
    let has_typewriter = facts
        .bypass_change_detection()
        .get_bool("dialogue:has_typewriter")
        .unwrap_or(true);
    if has_typewriter {
        facts.set("dialogue_text", FactValue::String(String::new()));
    }

    info!(
        "handle_pending_dialogue_start_system: spawning view '{}', mortar={:?}",
        view_path,
        mortar_path.as_ref().zip(mortar_node.as_ref())
    );

    // Spawn dialogue view
    spawn_view_writer.write(crate::core::view::SpawnViewRequest {
        path: view_path,
    });

    // Start Mortar dialogue if configured
    if let (Some(path), Some(node)) = (mortar_path, mortar_node) {
        // Prepend locale path for localized dialogue files
        let localized_path = format!("shared/locales/{}/{}", locale.0, path);

        info!(
            "handle_pending_dialogue_start_system: starting Mortar dialogue '{}' node '{}'",
            localized_path, node
        );

        mortar_events.write(MortarEvent::start_node(localized_path, node));
    }

    // Emit dialogue:started event for scene-specific handling (e.g., state change)
    // 发出 dialogue:started 事件用于场景特定处理（如状态切换）
    fre_event_writer.write(FactEvent::new("dialogue:started"));
}
