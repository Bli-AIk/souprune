//! # sequencer/context.rs
//!
//! ## Module Overview
//!
//! Core types and resources for the battle sequencer.
//!
//! 战斗序列管理器的核心类型和资源。

use super::SequenceAsset;
use super::chapter_schema::Chapter;
use bevy::prelude::*;

/// Execution state of the sequencer.
///
/// 序列管理器的执行状态。
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub enum SequenceExecutionState {
    #[default]
    Idle,
    Processing,
    Waiting,
}

/// [Resource] includes the queue of Chapters that have not yet occurred.
///
/// [Resource] 存放还没发生的章节队列。
#[derive(Resource, Default)]
pub struct SequenceContext {
    pub chapters: Vec<Chapter>,
    pub state: SequenceExecutionState,
}

/// Component for active chapters being processed.
///
/// 正在处理的活动章节的组件。
#[derive(Component)]
pub struct ActiveChapter {
    pub chapter: Chapter,
    pub parent: Option<Entity>,
}

/// Component for wait timer on chapters.
///
/// 章节等待计时器组件。
#[derive(Component)]
pub struct WaitTimer(pub Timer);

/// Component for tracking parallel chapter completion.
///
/// 跟踪并行章节完成的组件。
#[derive(Component)]
pub struct ParallelTracker {
    pub pending_count: usize,
}

/// Marker component for finished chapters.
///
/// 已完成章节的标记组件。
#[derive(Component)]
pub struct ChapterFinished;

/// Resource holding the current sequence flow asset handle.
///
/// 持有当前序列流程资产句柄的资源。
#[derive(Resource)]
pub struct CurrentSequenceFlow(pub Handle<SequenceAsset>);

/// Resource to track the rules handle loaded by the current sequence.
/// When a sequence specifies a `rules_file`, the handle is stored here
/// for processing by state-specific FRE plugins.
///
/// 跟踪当前序列加载的规则句柄的资源。
/// 当序列指定了 `rules_file` 时，句柄会存储在此处，
/// 由特定状态的 FRE 插件处理。
#[derive(Resource, Default)]
pub struct SequenceRulesHandle {
    pub handle: Option<Handle<crate::core::game_action::GameFreAsset>>,
    pub registered: bool,
}
