//! # Generic RON Source Editor
//!
//! # 通用 RON 源码编辑器
//!
//! ## Module Overview
//!
//! ## 模块概述
//!
//! Generic RON source editor for the editor.
//! Edits any `.ron` file with basic text editing functionality.
//! Also serves as a fallback for Performance editor and State configuration editor.
//!
//! 编辑器的通用 RON 源码编辑器。
//! 编辑任意 `.ron` 文件，提供基本文本编辑功能。
//! 同时作为 Performance 编辑器和 State 配置编辑器的回退。

use std::path::{Path, PathBuf};

use bevy::prelude::*;

use super::sub_editor::SubEditor;

/// 通用 RON 源码编辑器。
pub struct RonSourceEditor {
    file_path: Option<PathBuf>,
    content: String,
    dirty: bool,
    editor_id: String,
    icon: &'static str,
}

impl RonSourceEditor {
    pub fn performance() -> Self {
        Self {
            file_path: None,
            content: String::new(),
            dirty: false,
            editor_id: "performance_editor".to_string(),
            icon: "[PERF]",
        }
    }

    pub fn state_config() -> Self {
        Self {
            file_path: None,
            content: String::new(),
            dirty: false,
            editor_id: "state_editor".to_string(),
            icon: "[CFG]",
        }
    }

    pub fn generic() -> Self {
        Self {
            file_path: None,
            content: String::new(),
            dirty: false,
            editor_id: "ron_source".to_string(),
            icon: "[RON]",
        }
    }
}

impl SubEditor for RonSourceEditor {
    fn id(&self) -> &str {
        &self.editor_id
    }

    fn title(&self) -> String {
        let name = self
            .file_path
            .as_ref()
            .and_then(|p| p.file_name())
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "RON".to_string());
        format!("{} {name}", self.icon)
    }

    fn file_extensions(&self) -> &[&str] {
        match self.editor_id.as_str() {
            "performance_editor" => &["performance.ron"],
            "state_editor" => &["flow.ron"],
            _ => &["ron"],
        }
    }

    fn ui(&mut self, ui: &mut egui::Ui, world: &mut World) {
        ui.heading(self.title());

        if self.file_path.is_none() {
            ui.label(crate::i18n::t(world, "label-no-file-open"));
            return;
        }

        ui.horizontal(|ui| {
            if self.dirty {
                ui.label(
                    egui::RichText::new(crate::i18n::t(world, "label-unsaved"))
                        .color(egui::Color32::YELLOW),
                );
            }
            if ui.button(crate::i18n::t(world, "action-save")).clicked()
                && let Err(e) = self.save_inner()
            {
                warn!("{e}");
            }
        });
        ui.separator();

        egui::ScrollArea::vertical().show(ui, |ui| {
            if ui
                .add(
                    egui::TextEdit::multiline(&mut self.content)
                        .font(egui::TextStyle::Monospace)
                        .desired_width(f32::INFINITY)
                        .desired_rows(30),
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
                warn!("加载文件失败: {e}");
            }
        }
    }

    fn save(&self, _world: &mut World) -> Result<(), String> {
        self.save_inner()
    }

    fn is_dirty(&self) -> bool {
        self.dirty
    }
}

impl RonSourceEditor {
    fn save_inner(&self) -> Result<(), String> {
        let path = self
            .file_path
            .as_ref()
            .ok_or("No file path specified".to_string())?;
        std::fs::write(path, &self.content).map_err(|e| format!("Save failed: {e}"))
    }
}
