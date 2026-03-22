//! Organizes the concrete gameplay states that sit on top of Souprune's core infrastructure.
//!
//! 组织建立在 Souprune 核心基础设施之上的具体游戏状态。
//!
//! This file is the boundary between generic runtime systems and game-specific
//! flows. It pulls together the setup, overworld, and battle state modules, and
//! re-exports the shared mode/state primitives that the rest of the game uses to
//! move between those flows.
//!
//! 这个文件是通用运行时系统与具体游戏流程之间的边界。它汇总了 setup、
//! overworld 和 battle 这些状态模块，同时重导出全局共用的模式与状态基元，
//! 让其他系统可以围绕这些流程进行切换。

pub mod app_setup;
pub mod battle;
pub mod overworld;

pub use crate::core::mode::{
    AppState, ModeChanged, ModeScoped, SequenceMode, SequenceSubState, cleanup_entities_system,
    cleanup_mode_scoped_entities, detect_mode_changes, is_mode,
};
