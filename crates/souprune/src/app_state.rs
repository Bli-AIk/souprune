//! Application state management — re-exports core mode/state primitives.
//!
//! 应用状态管理 — 重导出核心模式/状态基元。
//!
//! Game-specific state flows (battle, overworld) are in the preset module.
//! This module provides the shared infrastructure for state transitions.
//!
//! 游戏特定的状态流程（战斗、大世界）在 preset 模块中。
//! 此模块提供状态切换的共享基础设施。

pub mod app_setup;

pub use crate::core::mode::{
    AppState, ModeChanged, ModeScoped, SequenceMode, SequenceSubState, cleanup_entities_system,
    cleanup_mode_scoped_entities, detect_mode_changes, is_mode,
};
