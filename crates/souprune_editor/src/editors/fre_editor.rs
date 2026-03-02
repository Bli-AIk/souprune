//! # FRE 规则编辑器
//!
//! 编辑 `.fre.ron` 文件。

use std::path::{Path, PathBuf};

use bevy::prelude::*;

use super::sub_editor::SubEditor;

/// FRE 规则编辑器。
#[derive(Default)]
pub struct FreEditor {
    file_path: Option<PathBuf>,
    content: String,
    dirty: bool,
}

impl SubEditor for FreEditor {
    fn id(&self) -> &str {
        "fre_editor"
    }

    fn title(&self) -> String {
        let name = self
            .file_path
            .as_ref()
            .and_then(|p| p.file_name())
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "FRE".to_string());
        format!("📏 {name}")
    }

    fn file_extensions(&self) -> &[&str] {
        &["fre.ron"]
    }

    fn ui(&mut self, ui: &mut egui::Ui, _world: &mut World) {
        ui.heading(self.title());
        ui.separator();

        if self.file_path.is_none() {
            ui.label("未打开任何 FRE 文件");
            return;
        }

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
                warn!("加载 FRE 文件失败: {e}");
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
