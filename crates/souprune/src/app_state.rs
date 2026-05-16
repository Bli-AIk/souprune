//! Application state management — re-exports core mode/state primitives.
//!
//! 应用状态管理 — 重导出核心模式/状态基元。
//!
//! Mode-specific flows are registered by core mode runtime plugins. This module
//! provides the shared infrastructure for state transitions.
//!
//! 模式专属流程由 core 模式运行时插件注册。本模块提供状态切换的共享基础设施。

pub mod app_setup;

pub use crate::core::mode::{
    AppState, ModeChanged, ModeScoped, SequenceMode, SequenceSubState, cleanup_entities_system,
    cleanup_mode_scoped_entities, detect_mode_changes, is_mode,
};
