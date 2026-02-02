//! # bridge.rs
//!
//! Bridge between Sequencer and FRE systems, plus input handling for Battle UI.
//!
//! Sequencer 和 FRE 系统之间的桥接，以及战斗 UI 的输入处理。
//!
//! This module provides systems that emit FRE events when specific
//! sequencer events occur (Chapter completion, selection confirmation, etc.),
//! and handles input for Battle UI navigation.
//!
//! 本模块提供系统，在特定 Sequencer 事件发生时发出 FRE 事件
//! （Chapter 完成、选择确认等），并处理战斗 UI 导航的输入。

use crate::app_state::battle::BattleInputManager;
use crate::core::audio;
use crate::core::input::{Action, ActionRegistry, ActionStateExt};
use crate::core::view::components::ViewRoot;
use bevy::prelude::*;
use bevy_fact_rule_event::{FactEvent, FactValue, LayeredFactDatabase};
use leafwing_input_manager::action_state::ActionState;

/// Event emitted when a Chapter completes.
/// This is an internal Bevy event used to bridge Sequencer → FRE.
///
/// Chapter 完成时发出的事件。
/// 这是用于桥接 Sequencer → FRE 的内部 Bevy 事件。
#[derive(Message, Debug, Clone)]
pub struct ChapterCompletedEvent {
    /// Type of chapter that completed (e.g., "DanmakuPerformance", "AwaitInteraction")
    pub chapter_type: String,
    /// Optional result data from the chapter
    pub result: Option<String>,
}

impl ChapterCompletedEvent {
    /// Create a new chapter completed event.
    pub fn new(chapter_type: impl Into<String>) -> Self {
        Self {
            chapter_type: chapter_type.into(),
            result: None,
        }
    }

    /// Create with result data.
    pub fn with_result(chapter_type: impl Into<String>, result: impl Into<String>) -> Self {
        Self {
            chapter_type: chapter_type.into(),
            result: Some(result.into()),
        }
    }
}

/// Event emitted when player confirms a selection in AwaitInteraction.
///
/// 玩家在 AwaitInteraction 中确认选择时发出的事件。
#[derive(Message, Debug, Clone)]
pub struct SelectionConfirmedEvent {
    /// The layer ID where selection was made
    pub layer_id: String,
    /// The selected option index
    pub selection_index: usize,
    /// The selected option ID (if available)
    pub selection_id: Option<String>,
}

impl SelectionConfirmedEvent {
    /// Create a new selection confirmed event.
    pub fn new(layer_id: impl Into<String>, selection_index: usize) -> Self {
        Self {
            layer_id: layer_id.into(),
            selection_index,
            selection_id: None,
        }
    }

    /// Create with selection ID.
    pub fn with_id(
        layer_id: impl Into<String>,
        selection_index: usize,
        selection_id: impl Into<String>,
    ) -> Self {
        Self {
            layer_id: layer_id.into(),
            selection_index,
            selection_id: Some(selection_id.into()),
        }
    }
}

/// System to emit FRE events when chapters complete.
/// Converts internal ChapterCompletedEvent to FactEvent for FRE processing.
///
/// Chapter 完成时发出 FRE 事件的系统。
/// 将内部 ChapterCompletedEvent 转换为 FactEvent 供 FRE 处理。
pub fn emit_chapter_completed_events_system(
    mut chapter_events: MessageReader<ChapterCompletedEvent>,
    mut fact_event_writer: MessageWriter<FactEvent>,
    mut layered_db: ResMut<LayeredFactDatabase>,
) {
    for event in chapter_events.read() {
        // Emit generic chapter completed event
        let mut fact_event =
            FactEvent::new("chapter_completed").with_data("chapter_type", &event.chapter_type);

        if let Some(result) = &event.result {
            fact_event = fact_event.with_data("result", result);
        }

        fact_event_writer.write(fact_event);

        // Emit specific events based on chapter type
        match event.chapter_type.as_str() {
            "DanmakuPerformance" => {
                fact_event_writer.write(FactEvent::new("danmaku_finished"));
            }
            "AmPerformance" => {
                fact_event_writer.write(FactEvent::new("am_performance_finished"));
            }
            "Wait" => {
                fact_event_writer.write(FactEvent::new("wait_finished"));
            }
            _ => {}
        }

        // Update phase fact if needed
        if event.chapter_type == "DanmakuPerformance" || event.chapter_type == "AmPerformance" {
            layered_db.set("phase", "player_turn");
            fact_event_writer.write(FactEvent::new("phase_changed"));
        }

        info!(
            "Battle FRE Bridge: Emitted events for chapter completion: {}",
            event.chapter_type
        );
    }
}

/// System to emit FRE events when player confirms a selection.
/// Updates relevant facts and emits selection events.
///
/// 玩家确认选择时发出 FRE 事件的系统。
/// 更新相关事实并发出选择事件。
pub fn emit_selection_confirmed_events_system(
    mut selection_events: MessageReader<SelectionConfirmedEvent>,
    mut fact_event_writer: MessageWriter<FactEvent>,
    mut layered_db: ResMut<LayeredFactDatabase>,
) {
    for event in selection_events.read() {
        // Update facts
        layered_db.set("last_selection_layer", event.layer_id.clone());
        layered_db.set("last_selection_index", event.selection_index as i64);

        if let Some(ref selection_id) = event.selection_id {
            layered_db.set("last_selection_id", selection_id.clone());
        }

        // Emit selection confirmed event
        let mut fact_event = FactEvent::new("selection_confirmed")
            .with_data("layer_id", &event.layer_id)
            .with_data("selection_index", event.selection_index.to_string());

        if let Some(ref selection_id) = event.selection_id {
            fact_event = fact_event.with_data("selection_id", selection_id);
        }

        fact_event_writer.write(fact_event);

        // Emit specific action events based on known layer IDs
        if event.layer_id == "BattleMainMenu" {
            let action_name = match event.selection_index {
                0 => "fight",
                1 => "act",
                2 => "item",
                3 => "mercy",
                _ => "unknown",
            };

            layered_db.set("player_last_action", action_name);
            fact_event_writer.write(FactEvent::new(format!("player_action_{}", action_name)));
        }

        info!(
            "Battle FRE Bridge: Selection confirmed in layer '{}', index {}",
            event.layer_id, event.selection_index
        );
    }
}

// ============================================================================
// Battle UI Navigation System
// 战斗 UI 导航系统
// ============================================================================

/// System that directly handles navigation input and updates ViewRoot.local_facts
/// in Battle mode. This enables menu navigation during AwaitInteraction chapters.
///
/// 在 Battle 模式下直接处理导航输入并更新 ViewRoot.local_facts 的系统。
/// 这使得在 AwaitInteraction 章节期间可以进行菜单导航。
#[cfg(all(feature = "bevy_kira_audio", not(feature = "firewheel")))]
pub fn battle_view_navigation_system(
    audio: Res<bevy_kira_audio::Audio>,
    asset_server: Res<AssetServer>,
    registry: Res<ActionRegistry>,
    input_query: Query<&ActionState<Action>, With<BattleInputManager>>,
    mut view_root_query: Query<&mut ViewRoot>,
    mut selection_events: MessageWriter<SelectionConfirmedEvent>,
) {
    let Ok(action_state) = input_query.single() else {
        return;
    };

    for mut view_root in view_root_query.iter_mut() {
        // Check if the view is active (menu is visible)
        let active = view_root.local_facts.get_bool("active").unwrap_or(false);

        if !active {
            continue;
        }

        let depth = view_root.local_facts.get_int("depth").unwrap_or(0);
        let selection = view_root.local_facts.get_int("selection").unwrap_or(0);

        // Battle main menu has 4 options: FIGHT, ACT, ITEM, MERCY
        let max_selection = if depth == 0 { 3 } else { 0 };

        // Handle horizontal navigation (Left/Right)
        if action_state.action_just_pressed(&registry, "Left") && selection > 0 {
            view_root
                .local_facts
                .set("selection", FactValue::Int(selection - 1));
            audio::play_sound(&audio, &asset_server, "choice.wav");
            debug!(
                "Battle navigation: selection {} -> {}",
                selection,
                selection - 1
            );
        }

        if action_state.action_just_pressed(&registry, "Right") && selection < max_selection {
            view_root
                .local_facts
                .set("selection", FactValue::Int(selection + 1));
            audio::play_sound(&audio, &asset_server, "choice.wav");
            debug!(
                "Battle navigation: selection {} -> {}",
                selection,
                selection + 1
            );
        }

        // Handle confirm
        if action_state.action_just_pressed(&registry, "Confirm") {
            audio::play_sound(&audio, &asset_server, "confirm.wav");

            // Emit selection confirmed event
            selection_events.write(SelectionConfirmedEvent::new(
                "BattleMainMenu",
                selection as usize,
            ));

            debug!("Battle confirm: depth {} selection {}", depth, selection);
        }
    }
}

/// Firewheel audio backend variant
#[cfg(feature = "firewheel")]
pub fn battle_view_navigation_system(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    registry: Res<ActionRegistry>,
    input_query: Query<&ActionState<Action>, With<BattleInputManager>>,
    mut view_root_query: Query<&mut ViewRoot>,
    mut selection_events: MessageWriter<SelectionConfirmedEvent>,
) {
    let Ok(action_state) = input_query.single() else {
        return;
    };

    for mut view_root in view_root_query.iter_mut() {
        let active = view_root.local_facts.get_bool("active").unwrap_or(false);

        if !active {
            continue;
        }

        let depth = view_root.local_facts.get_int("depth").unwrap_or(0);
        let selection = view_root.local_facts.get_int("selection").unwrap_or(0);

        let max_selection = if depth == 0 { 3 } else { 0 };

        if action_state.action_just_pressed(&registry, "Left") && selection > 0 {
            view_root
                .local_facts
                .set("selection", FactValue::Int(selection - 1));
            audio::play_sound(&mut commands, &asset_server, "choice.wav");
            debug!(
                "Battle navigation: selection {} -> {}",
                selection,
                selection - 1
            );
        }

        if action_state.action_just_pressed(&registry, "Right") && selection < max_selection {
            view_root
                .local_facts
                .set("selection", FactValue::Int(selection + 1));
            audio::play_sound(&mut commands, &asset_server, "choice.wav");
            debug!(
                "Battle navigation: selection {} -> {}",
                selection,
                selection + 1
            );
        }

        if action_state.action_just_pressed(&registry, "Confirm") {
            audio::play_sound(&mut commands, &asset_server, "confirm.wav");

            selection_events.write(SelectionConfirmedEvent::new(
                "BattleMainMenu",
                selection as usize,
            ));

            debug!("Battle confirm: depth {} selection {}", depth, selection);
        }
    }
}
