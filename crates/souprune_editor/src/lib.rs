//! Defines the Souprune editor crate and its top-level plugin entry.
//!
//! 定义 Souprune 编辑器 crate 以及它的顶层插件入口。
//!
//! This crate wraps the sequence, view, asset, and FRE editing tools into a
//! single Bevy plugin built on top of `bevy_workbench`. The file itself stays
//! intentionally small: it names the editor-facing modules and exposes the
//! plugin that lets another app embed the editor.
//!
//! 这个 crate 把序列、View、资源和 FRE 编辑工具封装成一个建立在
//! `bevy_workbench` 之上的 Bevy 插件。这个 crate 根入口刻意保持很薄：它只声明
//! 面向编辑器的模块，并暴露出可被其他应用接入的主插件。

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
