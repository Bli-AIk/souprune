//! # Editor Panels
//!
//! # 编辑器面板
//!
//! ## Module Overview
//!
//! ## 模块概述
//!
//! This module contains all dockable panels for the SoupRune editor.
//!
//! 此模块包含 SoupRune 编辑器的所有可停靠面板。

mod asset_browser;
mod chapter_inspector;
pub(crate) mod fre_panel;
pub(crate) mod playback;
pub(crate) mod sequence_timeline;

pub use asset_browser::AssetBrowserPanel;
pub use chapter_inspector::ChapterInspectorPanel;
pub use fre_panel::FrePanel;
pub use playback::PlaybackPanel;
pub use sequence_timeline::SequenceTimelinePanel;
