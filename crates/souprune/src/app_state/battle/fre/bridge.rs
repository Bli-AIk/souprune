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
use bevy_fact_rule_event::{FactEvent, FactReader, FactValue, LayeredFactDatabase};

use crate::core::view::ViewRoot;

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

/// Run condition: Check if there are chapter completed events to process.
/// 运行条件：检查是否有章节完成事件需要处理。
pub fn has_chapter_completed_events(events: MessageReader<ChapterCompletedEvent>) -> bool {
    !events.is_empty()
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
            "AlightMotionPerformance" => {
                fact_event_writer.write(FactEvent::new("alight_motion_performance_finished"));
            }
            "Wait" => {
                fact_event_writer.write(FactEvent::new("wait_finished"));
            }
            _ => {}
        }

        // Update phase fact if needed
        if event.chapter_type == "DanmakuPerformance"
            || event.chapter_type == "AlightMotionPerformance"
        {
            layered_db.set("phase", "player_turn");
            fact_event_writer.write(FactEvent::new("phase_changed"));
        }

        info!(
            "Battle FRE Bridge: Emitted events for chapter completion: {}",
            event.chapter_type
        );
    }
}

/// Resource to track the last seen depth and menu_context values.
/// Used to detect transitions into ACT options mode.
///
/// 追踪上次看到的 depth 和 menu_context 值的资源。
/// 用于检测进入 ACT 选项模式的转换。
#[derive(Resource, Default)]
pub struct ActOptionsTracker {
    pub last_depth: Option<i64>,
    pub last_menu_context: Option<i64>,
    /// The enemy index for which ACT data was last copied.
    /// 上次复制 ACT 数据的敌人索引。
    pub last_enemy_index: Option<i64>,
}

/// System to copy enemy ACT data to ViewRoot when entering ACT options.
/// When depth is 2 and menu_context is 1 (ACT), copies the
/// selected enemy's ACT data to current_enemy_* local facts.
/// Also ensures act_count is set for proper navigation.
///
/// 进入 ACT 选项时复制敌人 ACT 数据到 ViewRoot 的系统。
/// 当 depth 为 2 且 menu_context 为 1（ACT）时，复制选中敌人的 ACT 数据到
/// current_enemy_* 局部 facts。同时确保设置 act_count 以正确导航。
pub fn copy_enemy_act_data_system(
    mut tracker: ResMut<ActOptionsTracker>,
    mut view_roots: Query<&mut ViewRoot>,
    layered_db: Res<LayeredFactDatabase>,
) {
    // Get current depth and menu_context from ViewRoot local_facts
    let Ok(mut view_root) = view_roots.single_mut() else {
        return;
    };

    let current_depth = view_root.local_facts.get_int("depth").unwrap_or(0);
    let current_menu_context = view_root.local_facts.get_int("menu_context").unwrap_or(0);
    let enemy_selection = view_root
        .local_facts
        .get_int("enemy_selection")
        .unwrap_or(0);

    // Check if we're in ACT options mode (depth 2, menu_context 1)
    // 检查是否处于 ACT 选项模式（depth 2, menu_context 1）
    let in_act_options = current_depth == 2 && current_menu_context == 1;

    // Check if we need to copy data:
    // 1. Just entered ACT options (transition)
    // 2. In ACT options but act_count is still 0 (data not yet copied)
    // 3. Enemy selection changed while in ACT options
    //
    // 检查是否需要复制数据：
    // 1. 刚进入 ACT 选项（转换）
    // 2. 处于 ACT 选项但 act_count 仍为 0（数据尚未复制）
    // 3. 在 ACT 选项中敌人选择已更改
    let current_act_count = view_root.local_facts.get_int("act_count").unwrap_or(0);
    let entered_act_options =
        in_act_options && (tracker.last_depth != Some(2) || tracker.last_menu_context != Some(1));
    let act_count_not_set = in_act_options && current_act_count == 0;
    let enemy_changed = in_act_options
        && tracker
            .last_enemy_index
            .is_some_and(|idx| idx != enemy_selection);

    let need_copy = entered_act_options || act_count_not_set || enemy_changed;

    if need_copy {
        copy_act_data_for_enemy(&mut view_root, &layered_db, &mut tracker, enemy_selection);
    }

    // Update tracker
    tracker.last_depth = Some(current_depth);
    tracker.last_menu_context = Some(current_menu_context);
}

/// Copy the selected enemy's ACT data from the layered database into ViewRoot local_facts.
/// Looks up action_labels, action_sequences, action_params, and act_count using
/// multiple naming patterns (global, id-prefixed, index-prefixed).
fn copy_act_data_for_enemy(
    view_root: &mut ViewRoot,
    layered_db: &LayeredFactDatabase,
    tracker: &mut ActOptionsTracker,
    enemy_selection: i64,
) {
    // Get enemy IDs array - clone to avoid borrow issues
    // Fall back to enemy_names if enemy_ids is not available
    let enemy_ids_opt = view_root
        .local_facts
        .get_string_list("enemy_ids")
        .or_else(|| view_root.local_facts.get_string_list("enemy_names"))
        .map(|v| v.to_vec());

    let Some(ids) = enemy_ids_opt else { return };
    let enemy_index = enemy_selection as usize;
    if enemy_index >= ids.len() {
        return;
    }

    let enemy_id = ids[enemy_index].clone();
    info!(
        "ACT Options: Entering for enemy ID '{}' (index {})",
        enemy_id, enemy_index
    );

    // Try to find enemy action data with various naming patterns
    // Pattern 1: "action_labels" (global, for single-enemy demo)
    // Pattern 2: "dummy.action_labels" (id prefix)
    // Pattern 3: "enemy_0.action_labels" (index prefix)
    let lower_id = enemy_id.to_lowercase();

    // Get action_labels (display names)
    let action_labels = layered_db
        .get_string_list("action_labels")
        .or_else(|| layered_db.get_string_list(&format!("{lower_id}.action_labels")))
        .or_else(|| layered_db.get_string_list(&format!("enemy_{enemy_index}.action_labels")))
        .map(|v| v.to_vec());

    // Get action_sequences (sequence paths)
    let action_sequences = layered_db
        .get_string_list("action_sequences")
        .or_else(|| layered_db.get_string_list(&format!("{lower_id}.action_sequences")))
        .or_else(|| layered_db.get_string_list(&format!("enemy_{enemy_index}.action_sequences")))
        .map(|v| v.to_vec());

    // Get action_params (parameters for sequences)
    let action_params = layered_db
        .get_string_list("action_params")
        .or_else(|| layered_db.get_string_list(&format!("{lower_id}.action_params")))
        .or_else(|| layered_db.get_string_list(&format!("enemy_{enemy_index}.action_params")))
        .map(|v| v.to_vec());

    // Get act_count
    let act_count = layered_db
        .get_int("act_count")
        .or_else(|| layered_db.get_int(&format!("{lower_id}.act_count")))
        .or_else(|| layered_db.get_int(&format!("enemy_{enemy_index}.act_count")));

    // Set action facts in ViewRoot local_facts using generic naming
    let labels_len = action_labels.as_ref().map(|k| k.len());
    if let Some(labels) = action_labels {
        info!(
            "ACT Options: Found {} action labels for {}",
            labels.len(),
            enemy_id
        );
        view_root
            .local_facts
            .set("action_labels", FactValue::StringList(labels));
    } else {
        warn!(
            "ACT Options: No action_labels found for enemy ID '{}'",
            enemy_id
        );
    }

    if let Some(sequences) = action_sequences {
        info!(
            "ACT Options: Found {} action sequences for {}",
            sequences.len(),
            enemy_id
        );
        view_root
            .local_facts
            .set("action_sequences", FactValue::StringList(sequences));
    } else {
        warn!(
            "ACT Options: No action_sequences found for enemy ID '{}'",
            enemy_id
        );
    }

    if let Some(params) = action_params {
        info!(
            "ACT Options: Found {} action params for {}",
            params.len(),
            enemy_id
        );
        view_root
            .local_facts
            .set("action_params", FactValue::StringList(params));
    } else {
        warn!(
            "ACT Options: No action_params found for enemy ID '{}'",
            enemy_id
        );
    }

    if let Some(count) = act_count {
        info!("ACT Options: act_count = {} for {}", count, enemy_id);
        view_root
            .local_facts
            .set("act_count", FactValue::Int(count));
    } else if let Some(len) = labels_len {
        // Fall back to length of action_labels
        info!("ACT Options: Using labels.len() = {} as act_count", len);
        view_root
            .local_facts
            .set("act_count", FactValue::Int(len as i64));
    }

    // Update tracker with current enemy index
    tracker.last_enemy_index = Some(enemy_selection);
}
