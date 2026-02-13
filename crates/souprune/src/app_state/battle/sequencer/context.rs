//! # sequencer/context.rs
//!
//! ## Module Overview
//!
//! Core types and resources for the battle sequencer.
//!
//! 战斗序列管理器的核心类型和资源。

use super::super::SequenceAsset;
use super::super::chapter_schema::Chapter;
use bevy::prelude::*;

/// Execution state of the battle sequencer.
///
/// 战斗序列管理器的执行状态。
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub enum BattleExecutionState {
    #[default]
    Idle,
    Processing,
    Waiting,
}

/// [Resource] includes the queue of Chapters that have not yet occurred
///
/// [Resource] 存放还没发生的章节队列
#[derive(Resource, Default)]
pub struct BattleContext {
    pub chapters: Vec<Chapter>,
    pub state: BattleExecutionState,
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
