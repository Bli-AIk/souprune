//! Collects the systems that advance the active sequence flow chapter by chapter.
//!
//! 汇总按章节推进当前序列流程所需的系统。
//!
//! This module is the orchestration layer for sequence execution. Its submodules
//! cover root-flow loading, chapter lifecycle management, and the small set of
//! chapter types that directly mutate flow progression rather than spawning a
//! dedicated subsystem.
//!
//! 这个模块是序列执行的编排层。它的子模块分别负责根流程加载、章节生命周期管理，
//! 以及那些会直接推动流程前进、而不是另起一个专门子系统的章节类型。

mod actions;
mod lifecycle;
mod loading;

pub use actions::{
    process_custom_chapter_system,
    process_spawn_behavior_chapter_system,
};
pub use lifecycle::{
    advance_battle_flow_system, cleanup_finished_chapters_system, process_parallel_chapter_system,
    process_wait_chapter_system, spawn_chapter,
};
pub use loading::{load_default_chapter_system, sync_battle_flow_system};
