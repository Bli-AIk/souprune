//! Registers the workbench panels that make up the Souprune editor layout.
//!
//! 注册构成 Souprune 编辑器布局的各个 workbench 面板。
//!
//! Its job is narrow: define which
//! editor panels exist and in what bootstrap phase they are attached. The panel
//! implementations live elsewhere; this module only registers them with the
//! editor shell.
//!
//! 职责很窄：定义编辑器有哪些面板，以及它们
//! 在启动期什么时候注册。具体面板实现放在别处，这里只负责把它们挂到编辑器
//! 外壳上。

use bevy::prelude::*;
use bevy_workbench::prelude::*;

use crate::panels;

pub(super) fn register_panels(app: &mut App) {
    app.register_panel(panels::AssetBrowserPanel::new());
    app.register_panel(panels::SequenceTimelinePanel::new());
    app.register_panel(panels::ChapterInspectorPanel::new());
    app.register_panel(panels::PlaybackPanel::new());
    app.register_panel(panels::FrePanel::new());
    app.register_panel(panels::ViewEditorPanel::new());
}
