//! 子编辑器模块。
//!
//! 各子编辑器在桌面端以 tab 方式打开，Android 端以全屏 overlay 方式打开。

#[allow(dead_code)]
mod fre_editor;
#[allow(dead_code)]
mod ron_source_editor;
#[allow(dead_code)]
mod sub_editor;
#[allow(dead_code)]
mod view_editor;

#[allow(unused_imports)]
pub use fre_editor::FreEditor;
#[allow(unused_imports)]
pub use ron_source_editor::RonSourceEditor;
#[allow(unused_imports)]
pub use sub_editor::{NavEntry, SubEditor, SubEditorManager};
#[allow(unused_imports)]
pub use view_editor::ViewEditor;
