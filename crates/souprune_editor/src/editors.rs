//! 子编辑器模块。
//!
//! 各子编辑器在桌面端以 tab 方式打开，Android 端以全屏 overlay 方式打开。

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
