//! # sequencer.rs
//!
//! # sequencer.rs 文件
//!
//! ## Module Overview
//!
//! ## 模块概述
//!
//! Sequencer is the linear sequence manager.
//! It is responsible for managing and executing Chapters,
//! ensuring they proceed in order.
//!
//! Sequencer 是线性序列管理器。
//! 它负责管理和执行章节（Chapter），确保它们按顺序进行。

mod bgm;
mod camera;
pub(crate) mod context;
mod fact_chapter;
pub(crate) mod flow;
mod interaction;
mod load_map;
mod log_chapter;
mod performance;
mod player;
mod run_sequence;
pub(crate) mod tween;
pub mod view_action;
mod view_element;

pub mod chapter_schema;

// Re-export public types
pub use bgm::SequencerBgm;
pub use context::{
    ActiveChapter, ChapterFinished, CurrentSequenceFlow, SequenceContext, SequenceDebugInfo,
    SequenceExecutionState, SequenceRulesHandle, WaitTimer,
};
pub use flow::load_default_chapter_system;

use bevy::ecs::schedule::ScheduleLabel;
use bevy::prelude::*;
use bevy_tween::BevyTweenRegisterSystems;
use bevy_tween::tween::component_tween_system;
use serde::{Deserialize, Serialize};

use crate::core::ron_loader::RonAssetLoader;
use chapter_schema::Chapter;

/// SystemSet for sequencer systems.
/// Callers (BattlePlugin, OverworldPlugin) configure when this set runs.
///
/// Sequencer 系统集。
/// 调用方（BattlePlugin, OverworldPlugin）配置该集合何时运行。
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct SequencerUpdate;

/// Sequence configuration asset loaded from `.sequence.ron` files.
/// Contains the chapter sequence and optional rules file path.
///
/// 从 `.sequence.ron` 文件加载的序列配置资源。
/// 包含章节序列和可选的规则文件路径。
#[derive(Asset, TypePath, Debug, Clone, Deserialize, Serialize)]
pub struct SequenceAsset {
    /// The mode this sequence runs in (e.g., "overworld", "battle").
    /// Sets SequenceMode when this sequence is loaded.
    #[serde(default)]
    pub mode: Option<String>,

    /// Path to the FRE rules file for this sequence (optional).
    /// The rules will be loaded to the Local layer when the sequence starts.
    #[serde(default)]
    pub rules_file: Option<String>,

    /// Named exits for sequence-to-sequence jumps (supports graph editor).
    #[serde(default)]
    pub exits: std::collections::HashMap<String, String>,

    /// The sequence of chapters to execute.
    pub chapters: Vec<Chapter>,
}

/// Convert the shared schema asset into the runtime Sequencer asset.
///
/// This keeps schema consumers away from runtime-only chapter types.
pub fn runtime_asset_from_schema(
    schema: &souprune_schema::sequence::SequenceAsset,
) -> Result<SequenceAsset, String> {
    let serialized =
        ron::to_string(schema).map_err(|e| format!("failed to serialize sequence schema: {e}"))?;
    ron::from_str(&serialized)
        .map_err(|e| format!("failed to deserialize runtime sequence asset: {e}"))
}

/// Convert shared schema chapters into runtime chapters.
pub fn runtime_chapters_from_schema(
    chapters: &[souprune_schema::sequence::Chapter],
) -> Result<Vec<Chapter>, String> {
    let serialized =
        ron::to_string(chapters).map_err(|e| format!("failed to serialize chapters: {e}"))?;
    ron::from_str(&serialized).map_err(|e| format!("failed to deserialize runtime chapters: {e}"))
}

/// 初始化 Sequencer 所需的资源和资源类型。
///
/// 编辑器和游戏均可调用此函数完成资源初始化。
pub fn init_sequencer(app: &mut App) {
    app.init_resource::<context::SequenceContext>()
        .init_resource::<context::SequenceRulesHandle>()
        .init_resource::<context::SequenceDebugInfo>()
        .init_resource::<bgm::SequencerBgm>()
        .init_asset::<SequenceAsset>()
        .register_asset_loader(RonAssetLoader::<SequenceAsset>::new(&["sequence.ron"]));
}

/// 将 Sequencer 的章节处理系统注册到指定 Schedule。
///
/// 游戏使用 `Update`，编辑器使用 `GameSchedule`。
pub fn register_sequencer_systems(app: &mut App, schedule: impl ScheduleLabel + Clone) {
    // Register custom interpolator systems for bevy_tween
    app.add_tween_systems(
        PostUpdate,
        component_tween_system::<tween::ViewBoxSizeInterpolator>(),
    )
    .add_tween_systems(
        PostUpdate,
        component_tween_system::<tween::SpriteAlphaInterpolator>(),
    )
    .add_tween_systems(
        PostUpdate,
        component_tween_system::<tween::ViewBoxAlphaInterpolator>(),
    )
    // Chapter processing systems - split into two groups to avoid tuple size limit
    .add_systems(
        schedule.clone(),
        (
            flow::advance_battle_flow_system,
            player::process_player_action_system,
            camera::process_camera_action_system,
            view_action::process_view_action_system,
            view_action::process_set_view_fact_system,
            interaction::process_await_fact_system,
            view_element::process_modify_view_element_system,
            tween::process_set_view_element_system,
            performance::process_danmaku_performance_system,
            performance::process_am_performance_system,
            flow::process_custom_chapter_system,
            flow::process_spawn_behavior_chapter_system,
        )
            .chain()
            .in_set(SequencerUpdate),
    )
    .add_systems(
        schedule,
        (
            flow::process_wait_chapter_system,
            tween::process_tween_wait_chapter_system,
            performance::process_alight_motion_wait_chapter_system,
            performance::process_danmaku_wait_chapter_system,
            flow::process_parallel_chapter_system,
            fact_chapter::process_conditional_chapter_system,
            fact_chapter::process_fact_switch_chapter_system,
            fact_chapter::process_emit_fact_event_chapter_system,
            fact_chapter::process_modify_fact_chapter_system,
            fact_chapter::process_load_fre_chapter_system,
            fact_chapter::complete_load_fre_chapter_system,
            run_sequence::process_run_sequence_system,
            run_sequence::complete_run_sequence_system,
            load_map::process_load_map_system,
            bgm::process_set_bgm_system,
            log_chapter::process_log_chapter_system,
            interaction::check_await_fact_completion_system,
            flow::cleanup_finished_chapters_system,
            flow::sync_battle_flow_system,
        )
            .chain()
            .in_set(SequencerUpdate)
            .after(flow::advance_battle_flow_system),
    );
}

/// Sequencer 插件 — 在 `Update` 中运行章节处理系统。
pub(crate) struct SequencerPlugin;

impl Plugin for SequencerPlugin {
    fn build(&self, app: &mut App) {
        init_sequencer(app);
        register_sequencer_systems(app, crate::game_schedule(app));
    }
}
