//! # SoupRune Editor
//!
//! # SoupRune 编辑器
//!
//! ## Module Overview
//!
//! ## 模块概述
//!
//! Sequence-driven editor built on bevy_workbench.
//!
//! 基于 bevy_workbench 的序列驱动编辑器。

mod bootstrap;
mod data;
mod editors;
mod i18n;
pub mod icons;
mod panels;
mod platform;
mod sequencer_bridge;
pub mod widgets;

use bevy::prelude::*;

/// SoupRune 编辑器主插件。
pub struct SoupRuneEditorPlugin;

impl Plugin for SoupRuneEditorPlugin {
    fn build(&self, app: &mut App) {
        bootstrap::build_editor_app(app);
    }
}
