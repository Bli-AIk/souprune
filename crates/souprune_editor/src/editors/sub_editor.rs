//! # 子编辑器框架
//!
//! 定义子编辑器 trait 和导航栈管理。

use std::path::Path;

use bevy::prelude::*;

/// 子编辑器 trait。
pub trait SubEditor: Send + Sync + 'static {
    /// 唯一标识符。
    fn id(&self) -> &str;

    /// 显示标题。
    fn title(&self) -> String;

    /// 关联的文件扩展名。
    fn file_extensions(&self) -> &[&str];

    /// 渲染编辑器 UI。
    fn ui(&mut self, ui: &mut egui::Ui, world: &mut World);

    /// 加载文件。
    fn load(&mut self, path: &Path, world: &mut World);

    /// 保存当前编辑内容。
    fn save(&self, world: &mut World) -> Result<(), String>;

    /// 是否有未保存的修改。
    fn is_dirty(&self) -> bool {
        false
    }
}

/// 导航栈条目。
pub struct NavEntry {
    pub editor_id: String,
    pub file_path: String,
}

/// 子编辑器管理器资源。
#[derive(Resource, Default)]
pub struct SubEditorManager {
    /// 导航栈。
    pub nav_stack: Vec<NavEntry>,
    /// 当前活动的子编辑器 ID。
    pub active_editor: Option<String>,
}

impl SubEditorManager {
    /// 打开子编辑器。
    pub fn open(&mut self, editor_id: &str, file_path: &str) {
        self.nav_stack.push(NavEntry {
            editor_id: editor_id.to_string(),
            file_path: file_path.to_string(),
        });
        self.active_editor = Some(editor_id.to_string());
    }

    /// 返回上一个编辑器。
    pub fn go_back(&mut self) {
        self.nav_stack.pop();
        self.active_editor = self.nav_stack.last().map(|e| e.editor_id.clone());
    }
}
