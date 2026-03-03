//! # View 编辑器
//!
//! 编辑 `.view.ron` / `.view_layout.ron` 文件。

use std::path::{Path, PathBuf};

use bevy::prelude::*;

use super::sub_editor::SubEditor;

/// View 编辑器。
#[derive(Default)]
pub struct ViewEditor {
    file_path: Option<PathBuf>,
    content: String,
    dirty: bool,
}

impl SubEditor for ViewEditor {
    fn id(&self) -> &str {
        "view_editor"
    }

    fn title(&self) -> String {
        self.file_path
            .as_ref()
            .and_then(|p| p.file_name())
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "View".to_string())
    }

    fn file_extensions(&self) -> &[&str] {
        &["view.ron", "view_layout.ron"]
    }

    fn ui(&mut self, ui: &mut egui::Ui, _world: &mut World) {
        ui.heading(self.title());
        ui.separator();

        if self.file_path.is_none() {
            ui.label("未打开任何 View 文件");
            return;
        }

        // RON 源码编辑区
        egui::ScrollArea::vertical().show(ui, |ui| {
            if ui
                .add(
                    egui::TextEdit::multiline(&mut self.content)
                        .font(egui::TextStyle::Monospace)
                        .desired_width(f32::INFINITY),
                )
                .changed()
            {
                self.dirty = true;
            }
        });
    }

    fn load(&mut self, path: &Path, _world: &mut World) {
        match std::fs::read_to_string(path) {
            Ok(content) => {
                self.content = content;
                self.file_path = Some(path.to_path_buf());
                self.dirty = false;
            }
            Err(e) => {
                warn!("加载 View 文件失败: {e}");
            }
        }
    }

    fn save(&self, _world: &mut World) -> Result<(), String> {
        let path = self
            .file_path
            .as_ref()
            .ok_or("未指定文件路径".to_string())?;
        std::fs::write(path, &self.content).map_err(|e| format!("保存失败: {e}"))
    }

    fn is_dirty(&self) -> bool {
        self.dirty
    }
}
