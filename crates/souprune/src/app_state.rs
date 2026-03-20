//! # app_state.rs
//!
//! ## 模块概述
//!
//! 组织具体游戏状态模块，并重导出通用模式/子状态基础设施。

pub mod app_setup;
pub mod battle;
pub mod overworld;

pub use crate::core::mode::{
    AppState, ModeChanged, ModeScoped, SequenceMode, SequenceSubState, cleanup_entities_system,
    cleanup_mode_scoped_entities, detect_mode_changes, is_mode,
};
