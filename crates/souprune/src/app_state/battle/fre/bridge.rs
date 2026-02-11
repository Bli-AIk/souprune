//! # bridge.rs
//!
//! Bridge between Sequencer and FRE systems.
//!
//! Sequencer 和 FRE 系统之间的桥接。
//!
//! This module provides systems that emit FRE events when specific
//! sequencer events occur (Chapter completion, selection confirmation, etc.).
//!
//! 本模块提供系统，在特定 Sequencer 事件发生时发出 FRE 事件
//! （Chapter 完成、选择确认等）。
//!
//! NOTE: Battle UI navigation has been moved to FRE rules in battle_menu.fre.ron.
//! The old hardcoded navigation system has been removed.
//!
//! 注意：战斗 UI 导航已移至 battle_menu.fre.ron 中的 FRE 规则。
//! 旧的硬编码导航系统已被移除。

use bevy::prelude::*;
use bevy_fact_rule_event::{FactEvent, LayeredFactDatabase};

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
