//! # Sub Editors
//!
//! # 子编辑器模块
//!
//! ## Module Overview
//!
//! ## 模块概述
//!
//! This module contains sub-editors for editing different file types in the SoupRune editor.
//! Sub-editors open as tabs on desktop and as full-screen overlays on Android.
//!
//! 此模块包含用于在 SoupRune 编辑器中编辑不同文件类型的子编辑器。
//! 子编辑器在桌面端以标签页方式打开，在 Android 端以全屏覆盖层方式打开。

#[allow(dead_code)]
mod fre_editor;
#[allow(dead_code)]
mod ron_source_editor;
mod sub_editor;
#[allow(dead_code)]
mod view_editor;

#[allow(unused_imports)]
pub use fre_editor::FreEditor;
#[allow(unused_imports)]
pub use ron_source_editor::RonSourceEditor;
pub use sub_editor::SubEditorManager;

#[allow(unused_imports)]
pub use sub_editor::{NavEntry, SubEditor};
#[allow(unused_imports)]
pub use view_editor::ViewEditor;
