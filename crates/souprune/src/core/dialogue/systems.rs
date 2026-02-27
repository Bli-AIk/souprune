//! Dialogue system core systems.
//!
//! 对话系统核心系统。

use bevy::prelude::*;
use bevy_ecs_typewriter::{Typewriter, TypewriterState};
use bevy_fact_rule_event::{FactEvent, FactValue, LayeredFactDatabase};
use bevy_mortar_bond::{MortarDialogueFinished, MortarEvent, MortarRuntime};

use super::components::{MortarController, TypewriterVoice};
use super::config::DialogueInputConfig;
use crate::core::view::components::{ActiveView, ViewRoot};

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
            TypewriterState::Paused => {
                // Paused still counts as "playing" for skip purposes - text is not fully shown
                // 暂停仍算作"正在播放"用于跳过 - 文本尚未完全显示
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
        }
    }

    // If no typewriters exist, consider all finished
    if !has_typewriters {
        all_finished = true;
        any_finished = true;
    }

    // Use bypass_change_detection to avoid triggering change detection unless values differ
    // 使用 bypass_change_detection 避免在值相同时触发 change detection
    let db = facts.bypass_change_detection();

    // Helper to update fact only if value changed
    // 辅助函数：仅当值变化时更新 fact
    let mut changed = false;
    if db.set_if_changed("dialogue:typewriter_playing", any_playing) {
        changed = true;
    }
    if db.set_if_changed("dialogue:all_typewriters_finished", all_finished) {
        changed = true;
    }
    if db.set_if_changed("dialogue:any_typewriter_finished", any_finished) {
        changed = true;
    }

    // Manually set changed flag if any value was updated
    // 如果有任何值被更新，手动设置 changed 标志
    if changed {
        facts.set_changed();
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
/// Run condition: Check if there are fact events to process.
/// 运行条件：检查是否有 fact 事件需要处理。
pub fn has_fact_events(events: MessageReader<FactEvent>) -> bool {
    !events.is_empty()
}

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
    mut facts: ResMut<LayeredFactDatabase>,
    query: Query<&Typewriter, With<DialogueControllerEntity>>,
    runtime: Res<MortarRuntime>,
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
                // Simple text without typewriter - mark for dialogue:ended
                // This will be processed by emit_pending_dialogue_ended_system
                // 简单文本无打字机 - 标记为需要发送 dialogue:ended
                // 这将由 emit_pending_dialogue_ended_system 处理
                info!(
                    "dialogue_advance_system: simple text (no typewriter), marking dialogue ended"
                );
                facts.set("dialogue:pending_ended", FactValue::Bool(true));
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
            // Simple text with finished typewriter - mark for dialogue:ended
            // 简单文本打字机完成 - 标记为需要发送 dialogue:ended
            info!("dialogue_advance_system: simple text finished, marking dialogue ended");
            facts.set("dialogue:pending_ended", FactValue::Bool(true));
        }
    }
}

/// Run condition: Check if there's pending dialogue end to emit.
/// 运行条件：检查是否有待发送的对话结束事件。
pub fn has_pending_dialogue_ended(facts: Res<LayeredFactDatabase>) -> bool {
    facts.get_bool("dialogue:pending_ended").unwrap_or(false)
}

/// Emits dialogue:ended event when pending_ended fact is set.
/// This is a separate system to avoid MessageReader/MessageWriter conflict.
///
/// 当 pending_ended fact 被设置时发出 dialogue:ended 事件。
/// 这是一个独立的系统以避免 MessageReader/MessageWriter 冲突。
pub fn emit_pending_dialogue_ended_system(
    mut facts: ResMut<LayeredFactDatabase>,
    mut fre_event_writer: MessageWriter<FactEvent>,
) {
    if facts.get_bool("dialogue:pending_ended").unwrap_or(false) {
        info!("emit_pending_dialogue_ended_system: emitting dialogue:ended");
        fre_event_writer.write(FactEvent::new("dialogue:ended"));
        facts.remove("dialogue:pending_ended");
    }
}

/// Listens for MortarDialogueFinished messages and emits dialogue:ended FRE event.
///
/// 监听 MortarDialogueFinished 消息并发出 dialogue:ended FRE 事件。
///
/// This bridges the Mortar dialogue system with the FRE-driven dialogue cleanup.
/// When a Mortar dialogue finishes naturally (not via StopDialogue), this system
/// emits the corresponding FRE event for rule-based handling.
///
/// 这将 Mortar 对话系统与 FRE 驱动的对话清理桥接起来。
/// 当 Mortar 对话自然结束时（非通过 StopDialogue），此系统发出相应的 FRE 事件供规则处理。
pub fn handle_mortar_dialogue_finished_system(
    mut mortar_finished: MessageReader<MortarDialogueFinished>,
    mut fre_event_writer: MessageWriter<FactEvent>,
) {
    for finished in mortar_finished.read() {
        info!(
            "handle_mortar_dialogue_finished_system: Mortar dialogue finished (path: {}, node: {})",
            finished.mortar_path, finished.node
        );
        fre_event_writer.write(FactEvent::new("dialogue:ended"));
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

        // Note: Focus checking is done at the FRE rule level (dialogue.fre.ron)
        // This system just executes the skip action unconditionally
        // 注意：焦点检查在 FRE 规则层完成 (dialogue.fre.ron)
        // 此系统只负责无条件执行跳过操作

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
                // Skip to end - show all text immediately
                typewriter.current_text = typewriter.source_text.clone();
                typewriter.current_char_index = typewriter.source_text.chars().count();
                typewriter.state = TypewriterState::Finished;
                info!("Typewriter skipped to end");
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
            trace!(
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

/// Syncs Typewriter current_text to View's local_facts for text binding.
///
/// 将 Typewriter 的 current_text 同步到 View 的 local_facts 用于文本绑定。
///
/// Updates `dialogue_text` fact in the ActiveView's local_facts.
/// Views can reference this with `{{dialogue_text}}` in their text templates.
///
/// 使用打字机当前显示的文本更新 ActiveView 的 local_facts 中的 `dialogue_text` fact。
/// View 可在文本模板中使用 `{{dialogue_text}}` 引用。
pub fn sync_typewriter_text_to_facts_system(
    runtime: Res<bevy_mortar_bond::MortarRuntime>,
    query: Query<&Typewriter, With<DialogueControllerEntity>>,
    mut active_view_query: Query<&mut ViewRoot, With<ActiveView>>,
    facts: Res<LayeredFactDatabase>,
) {
    // Get the first focused typewriter's current text
    // 获取第一个焦点打字机的当前文本
    let typewriter_count = query.iter().count();

    // Check if simple text dialogue is active via FRE fact (data-driven)
    let simple_text_active = facts
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

    // Update dialogue_text and dialogue_visible in all ActiveView's local_facts
    // 更新所有 ActiveView 的 local_facts 中的 dialogue_text 和 dialogue_visible
    let dialogue_visible = !new_text.is_empty();
    for mut view_root in active_view_query.iter_mut() {
        let current = view_root
            .local_facts
            .get_string("dialogue_text")
            .map(|s| s.to_string())
            .unwrap_or_default();

        if current != new_text {
            trace!(
                "sync_typewriter_text_to_facts: updating {} dialogue_text: '{}' -> '{}'",
                view_root.namespace, current, new_text
            );
            view_root
                .local_facts
                .set("dialogue_text", FactValue::String(new_text.clone()));
            // Also set dialogue_visible for view visibility control
            // 同时设置 dialogue_visible 用于视图可见性控制
            view_root
                .local_facts
                .set("dialogue_visible", FactValue::Bool(dialogue_visible));
        }
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
    facts: Res<LayeredFactDatabase>,
    mut active_view_query: Query<&mut ViewRoot, With<ActiveView>>,
) {
    let has_controller = !query.is_empty();

    // Check if dialogue should be active via FRE facts
    // Use dialogue:active fact set by handle_pending_dialogue_start_system
    // This avoids timing issues where Mortar hasn't started yet
    // 使用 handle_pending_dialogue_start_system 设置的 dialogue:active fact
    // 这避免了 Mortar 尚未启动时的时序问题
    let dialogue_active = facts.get_bool("dialogue:active").unwrap_or(false);
    let simple_text_active = facts
        .get_bool("dialogue:simple_text_active")
        .unwrap_or(false);
    let has_dialogue = dialogue_active || simple_text_active;

    // Configuration from FRE facts
    let has_typewriter = facts.get_bool("dialogue:has_typewriter").unwrap_or(true); // Default to true for backward compatibility

    // Check if this is a Mortar dialogue (set by handle_pending_dialogue_start_system)
    // 检查是否是 Mortar 对话（由 handle_pending_dialogue_start_system 设置）
    let has_mortar =
        facts.get_bool("dialogue:has_mortar").unwrap_or(false) || runtime.has_active_dialogues();

    // DEBUG: Log state every frame when there's any dialogue-related activity
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

    // Spawn controller when dialogue starts
    if has_dialogue && !has_controller {
        info!(
            "spawn_dialogue_controller_system: spawning dialogue controller (mortar={}, simple_text={}, typewriter={})",
            has_mortar, simple_text_active, has_typewriter
        );

        // NOTE: dialogue:has_focus is now fully controlled by FRE rules.
        // FRE rules should set dialogue:has_focus as part of their dialogue setup.
        // This allows scenarios like Battle encounter intro to run without blocking menu.
        // 注意：dialogue:has_focus 现在完全由 FRE 规则控制。
        // FRE 规则应在对话设置中设置 dialogue:has_focus。
        // 这允许像战斗遭遇开场这样的场景运行而不阻塞菜单。

        let mut entity_commands = commands.spawn(DialogueControllerEntity);

        // Add MortarController if Mortar dialogue is active
        if has_mortar {
            entity_commands.insert(MortarController::new());
        }

        // Add Typewriter if configured
        if has_typewriter {
            // Get simple text from FRE fact if available
            let simple_text = facts
                .get_string("dialogue:simple_text")
                .map(|s| s.to_string());

            // For simple text (no Mortar), initialize typewriter with the text directly
            // For Mortar dialogues, start empty - sync_mortar_text_to_typewriter_system will fill it
            let initial_text = if !has_mortar {
                simple_text.unwrap_or_default()
            } else {
                String::new()
            };

            // Read typewriter speed from FRE fact, default to 0.03 (30ms per char)
            // 从 FRE fact 读取打字机速度，默认为 0.03（每字符30ms）
            let typewriter_speed = facts
                .get_float("dialogue:typewriter_speed")
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

            // Add TypewriterVoice if dialogue:voice fact is set
            // 如果设置了 dialogue:voice fact，添加 TypewriterVoice
            if let Some(voice_path) = facts.get_string("dialogue:voice")
                && !voice_path.is_empty()
            {
                info!(
                    "spawn_dialogue_controller_system: adding TypewriterVoice with path: '{}'",
                    voice_path
                );
                entity_commands.insert(super::components::TypewriterVoice::new(voice_path));
            }
        }

        // For simple text without Typewriter, set the dialogue_text in View's local_facts
        // If there's a typewriter, sync_typewriter_text_to_facts_system will handle it
        if !has_mortar
            && !has_typewriter
            && let Some(text) = facts.get_string("dialogue:simple_text")
        {
            info!(
                "spawn_dialogue_controller_system: setting simple_text to View local_facts: '{}'",
                text
            );
            let text_owned = text.to_string();
            // Update all ActiveView's local_facts
            for mut view_root in active_view_query.iter_mut() {
                view_root
                    .local_facts
                    .set("dialogue_text", FactValue::String(text_owned.clone()));
            }
        }
    }
}

/// Run condition: Check if despawn_dialogue_controller_system should run.
/// 运行条件：检查 despawn_dialogue_controller_system 是否应该运行。
pub fn should_check_dialogue_despawn(
    fre_events: MessageReader<FactEvent>,
    query: Query<Entity, With<DialogueControllerEntity>>,
) -> bool {
    // Run if there's a dialogue:ended event OR if there's a controller to potentially clean up
    !fre_events.is_empty() || !query.is_empty()
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
    // Check both runtime and FRE facts to avoid race conditions
    // 也在控制器存在但没有活跃对话时清理
    // 同时检查 runtime 和 FRE facts 以避免竞态条件
    if !should_cleanup {
        let mortar_active = runtime.has_active_dialogues();
        let simple_active = facts
            .bypass_change_detection()
            .get_bool("dialogue:simple_text_active")
            .unwrap_or(false);
        // Also check dialogue:active fact to handle the frame where Mortar hasn't started yet
        // 同时检查 dialogue:active fact 以处理 Mortar 尚未启动的那一帧
        let dialogue_active_fact = facts
            .bypass_change_detection()
            .get_bool("dialogue:active")
            .unwrap_or(false);
        let has_controller = !query.is_empty();

        // Only cleanup if controller exists AND both runtime and fact indicate no dialogue
        // 仅在控制器存在且 runtime 和 fact 都指示没有对话时清理
        should_cleanup =
            has_controller && !mortar_active && !simple_active && !dialogue_active_fact;
    }

    if !should_cleanup {
        return;
    }

    info!("despawn_dialogue_controller_system: dialogue ended, cleaning up controller");

    // Reset dialogue-related facts when cleaning up (Local layer for scene scope)
    // 清理时重置对话相关的 facts（Local 层用于场景作用域）
    facts.set("dialogue:has_focus", FactValue::Bool(false));
    facts.set("dialogue:active", FactValue::Bool(false));
    facts.set("dialogue:has_mortar", FactValue::Bool(false));

    // Despawn controller entity
    // Note: View cleanup should be handled by FRE rules with DespawnView action
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }
}

/// Run condition: Check if there's a pending dialogue start.
/// 运行条件：检查是否有待启动的对话。
pub fn has_pending_dialogue_start(facts: Res<LayeredFactDatabase>) -> bool {
    facts.get_bool("dialogue:pending_start").unwrap_or(false)
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
/// Handle pending dialogue startup based on FRE facts.
///
/// 基于 FRE facts 处理待启动的对话。
///
/// This system monitors `dialogue:pending_start` fact as trigger.
/// When true, it reads other pending facts and starts the dialogue.
///
/// 该系统监听 `dialogue:pending_start` fact 作为触发器。
/// 为 true 时，读取其他 pending facts 并启动对话。
///
/// **Facts used**:
/// - `dialogue:pending_start` (bool): Trigger - set to true to start dialogue
/// - `dialogue:pending_view` (string): Optional view to spawn (empty = no view)
/// - `dialogue:pending_mortar_path` (string): Mortar file path (without locale prefix)
/// - `dialogue:pending_mortar_node` (string): Mortar node name
///
/// This unified approach replaces scene-specific StartDialogue actions,
/// ensuring consistent behavior across Overworld and Battle.
///
/// 这种统一方式替代了场景特定的 StartDialogue action，
/// 确保 Overworld 和 Battle 中的行为一致。
pub fn handle_pending_dialogue_start_system(
    mut facts: ResMut<LayeredFactDatabase>,
    mut mortar_events: MessageWriter<MortarEvent>,
    mut spawn_view_writer: MessageWriter<crate::core::view::SpawnViewRequest>,
    mut fre_event_writer: MessageWriter<FactEvent>,
    locale: Res<crate::extra::mortar::CurrentLocale>,
) {
    // Check trigger fact
    let pending_start = facts
        .bypass_change_detection()
        .get_bool("dialogue:pending_start")
        .unwrap_or(false);

    if !pending_start {
        return;
    }

    info!("handle_pending_dialogue_start_system: pending_start=true, processing dialogue");

    // Read pending view (optional - empty means no view to spawn)
    let pending_view = facts
        .bypass_change_detection()
        .get_string("dialogue:pending_view")
        .map(|s| s.to_string())
        .filter(|s| !s.is_empty());

    // Read Mortar configuration
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

    info!(
        "handle_pending_dialogue_start_system: view={:?}, path={:?}, node={:?}",
        pending_view, mortar_path, mortar_node
    );

    // Clear all pending facts (use set() to write to Local layer,
    // since FRE rules write to Local layer by default)
    // 清除所有待处理的 facts（使用 set() 写入 Local 层，
    // 因为 FRE 规则默认写入 Local 层）
    facts.set("dialogue:pending_start", FactValue::Bool(false));
    facts.set("dialogue:pending_view", FactValue::String(String::new()));
    facts.set(
        "dialogue:pending_mortar_path",
        FactValue::String(String::new()),
    );
    facts.set(
        "dialogue:pending_mortar_node",
        FactValue::String(String::new()),
    );

    // NOTE: dialogue_text is now initialized by View's `facts:` section to empty string.
    // No need to clear it here - the View spawn will handle it.
    // 注意：dialogue_text 现在由 View 的 `facts:` 部分初始化为空字符串。
    // 不需要在此清除 - View 生成时会处理。

    info!(
        "handle_pending_dialogue_start_system: view={:?}, mortar={:?}",
        pending_view,
        mortar_path.as_ref().zip(mortar_node.as_ref())
    );

    // Spawn dialogue view if specified
    if let Some(view_path) = pending_view {
        spawn_view_writer.write(crate::core::view::SpawnViewRequest { path: view_path });
    }

    // Start Mortar dialogue if configured
    let has_mortar = mortar_path.is_some() && mortar_node.is_some();
    if let (Some(path), Some(node)) = (mortar_path.clone(), mortar_node.clone()) {
        // Prepend locale path for localized dialogue files
        let localized_path = format!("shared/locales/{}/{}", locale.0, path);

        info!(
            "handle_pending_dialogue_start_system: starting Mortar dialogue '{}' node '{}'",
            localized_path, node
        );

        mortar_events.write(MortarEvent::start_node(localized_path, node));
    }

    // Set dialogue:active to indicate dialogue is starting (Local layer for scene scope)
    // This allows spawn_dialogue_controller_system to spawn the controller
    // before runtime.has_active_dialogues() returns true
    // 设置 dialogue:active 表示对话正在启动（Local 层用于场景作用域）
    // 这允许 spawn_dialogue_controller_system 在 runtime.has_active_dialogues() 返回 true 之前生成控制器
    let simple_text_active = facts
        .bypass_change_detection()
        .get_bool("dialogue:simple_text_active")
        .unwrap_or(false);
    let dialogue_active = has_mortar || simple_text_active;
    facts.set("dialogue:active", FactValue::Bool(dialogue_active));

    // Set dialogue:has_mortar to distinguish from simple_text
    // 设置 dialogue:has_mortar 以区分于 simple_text
    facts.set("dialogue:has_mortar", FactValue::Bool(has_mortar));

    // Emit dialogue:started event for scene-specific handling
    fre_event_writer.write(FactEvent::new("dialogue:started"));
}

/// System to replay Typewriter when View's depth returns to 0.
///
/// 当 View 的 depth 返回 0 时重播 Typewriter 的系统。
///
/// This system monitors the `depth` fact in ActiveView's local_facts.
/// When `depth` changes from non-zero to 0 and `dialogue:replay_on_resume` is true,
/// it restarts the Typewriter component to replay the text effect.
///
/// 该系统监控 ActiveView 的 local_facts 中的 `depth` fact。
/// 当 `depth` 从非零变为 0 且 `dialogue:replay_on_resume` 为 true 时，
/// 重启 Typewriter 组件以重播文本效果。
///
/// **Configuration fact** (in View's local_facts):
/// - `dialogue:replay_on_resume` (bool): If true, replay typewriter when resuming (default: false)
pub fn replay_typewriter_on_depth_resume_system(
    mut active_view_query: Query<&mut ViewRoot, With<ActiveView>>,
    mut typewriter_query: Query<&mut Typewriter, With<DialogueControllerEntity>>,
    mut prev_depth: Local<Option<i64>>,
) {
    // Get current depth from ActiveView's local_facts
    let current_depth = active_view_query
        .iter()
        .next()
        .and_then(|view| view.local_facts.get_int("depth"));

    // Log depth changes for debugging
    if *prev_depth != current_depth {
        debug!(
            "replay_typewriter_on_depth_resume: depth changed {:?} -> {:?}",
            *prev_depth, current_depth
        );
    }

    // Check if depth changed from 0 to non-zero (pause typewriter)
    let left_zero = match (*prev_depth, current_depth) {
        (Some(prev), Some(curr)) if prev == 0 && curr != 0 => true,
        (None, Some(curr)) if curr != 0 => true,
        _ => false,
    };

    // Check if depth changed from non-zero to 0 (resume/replay typewriter)
    let resumed_to_zero = match (*prev_depth, current_depth) {
        (Some(prev), Some(curr)) if prev != 0 && curr == 0 => true,
        _ => false,
    };

    // Update previous depth
    *prev_depth = current_depth;

    // Pause typewriter when depth leaves 0
    if left_zero {
        info!("replay_typewriter_on_depth_resume: depth left 0, pausing typewriters");
        for mut typewriter in typewriter_query.iter_mut() {
            typewriter.pause();
        }
        return;
    }

    if !resumed_to_zero {
        return;
    }

    info!("replay_typewriter_on_depth_resume: depth returned to 0");

    // Check if replay_on_resume is enabled (from View's local_facts)
    // This fact can be dynamically controlled by FRE rules (e.g., set to false during narration)
    // 检查 replay_on_resume 是否启用（从 View 的 local_facts 读取）
    // 此 fact 可由 FRE 规则动态控制（如在旁白期间设为 false）
    let replay_enabled = active_view_query
        .iter()
        .next()
        .map(|view| {
            view.local_facts
                .get_bool("dialogue:replay_on_resume")
                .unwrap_or(false)
        })
        .unwrap_or(false);

    info!(
        "replay_typewriter_on_depth_resume: replay_enabled={}",
        replay_enabled
    );

    // Check if there's a typewriter to restart/resume
    let typewriter_count = typewriter_query.iter().count();
    info!(
        "replay_typewriter_on_depth_resume: found {} typewriters",
        typewriter_count
    );

    if replay_enabled {
        // Restart typewriter to replay the effect
        for mut typewriter in typewriter_query.iter_mut() {
            info!(
                "replay_typewriter_on_depth_resume: restarting typewriter, source_text='{}'",
                typewriter.source_text
            );
            typewriter.restart();
        }

        // Immediately clear dialogue_text in View to prevent one-frame flash
        // sync_typewriter_text_to_facts_system runs before this system, so the View
        // would show full text until next frame unless we clear it here
        // 立即清空 View 中的 dialogue_text 以防止一帧闪烁
        // sync_typewriter_text_to_facts_system 在此系统之前运行，所以如果不在此清空，
        // View 会显示完整文本直到下一帧
        for mut view_root in active_view_query.iter_mut() {
            view_root
                .local_facts
                .set("dialogue_text", FactValue::String(String::new()));
        }
    } else {
        // Just resume the paused typewriter without restarting
        for mut typewriter in typewriter_query.iter_mut() {
            info!(
                "replay_typewriter_on_depth_resume: resuming typewriter, source_text='{}'",
                typewriter.source_text
            );
            typewriter.resume();
        }
    }
}

/// System to play typewriter voice sound when characters are displayed.
///
/// 当字符显示时播放打字机音效的系统。
///
/// This system monitors the Typewriter's current_char_index and plays
/// the configured voice sound each time it increases.
///
/// 此系统监控 Typewriter 的 current_char_index，每次增加时播放配置的音效。
pub fn typewriter_voice_system(
    mut query: Query<(&Typewriter, &mut TypewriterVoice)>,
    audio: Res<bevy_kira_audio::Audio>,
    asset_server: Res<AssetServer>,
) {
    use bevy_kira_audio::AudioControl;

    for (typewriter, mut voice) in query.iter_mut() {
        // Only play when typewriter is playing and char index has increased
        if typewriter.state != TypewriterState::Playing {
            continue;
        }

        // Check if character index has increased
        if typewriter.current_char_index > voice.last_char_index {
            // Play voice sound
            let handle: Handle<bevy_kira_audio::AudioSource> = asset_server.load(&voice.sound_path);
            audio.play(handle);

            // Update last observed index
            voice.last_char_index = typewriter.current_char_index;
        } else if typewriter.current_char_index < voice.last_char_index {
            // Typewriter was reset, update tracking
            voice.last_char_index = typewriter.current_char_index;
        }
    }
}

/// System to stop typewriters on FRE "dialogue:stop" event.
///
/// 在 FRE "dialogue:stop" 事件时停止打字机的系统。
///
/// This system listens for the `dialogue:stop` event and stops all
/// DialogueControllerEntity typewriters. The event is emitted by sequences
/// (via FRE EmitFactEvent) to explicitly stop dialogue before showing other content.
///
/// 此系统监听 `dialogue:stop` 事件并停止所有 DialogueControllerEntity 的打字机。
/// 该事件由序列（通过 FRE EmitFactEvent）发出，用于在显示其他内容前显式停止对话。
///
/// Note: This system provides the "stop typewriter" capability. Business logic
/// (when to stop) is controlled by FRE rules/sequences, not hardcoded here.
///
/// 注意：此系统提供"停止打字机"的能力。业务逻辑（何时停止）由 FRE 规则/序列控制，
/// 不在此处硬编码。
pub fn handle_dialogue_stop_event_system(
    mut events: MessageReader<FactEvent>,
    mut typewriter_query: Query<&mut Typewriter, With<DialogueControllerEntity>>,
) {
    for event in events.read() {
        if event.id.0.starts_with("dialogue:stop") {
            info!("handle_dialogue_stop_event: stopping all typewriters");
            for mut typewriter in typewriter_query.iter_mut() {
                typewriter.stop();
            }
        }
    }
}
