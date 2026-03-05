//! # File Path Picker
//!
//! # 文件路径选择器
//!
//! ## Module Overview
//!
//! ## 模块概述
//!
//! File path picker widget for the editor.
//! Provides file path editing UI: text box + browse button.
//!
//! 编辑器的文件路径选择器组件。
//! 提供文件路径编辑 UI：文本框 + 浏览按钮。

/// 渲染文件路径编辑器：文本框 + 浏览按钮。
///
/// 返回路径是否被修改。
pub fn edit_file_path(
    ui: &mut egui::Ui,
    label: &str,
    path: &mut String,
    world: &bevy::prelude::World,
) -> bool {
    let mut changed = false;
    ui.horizontal(|ui| {
        ui.label(format!("{label}:"));
        if ui.text_edit_singleline(path).changed() {
            changed = true;
        }
        #[cfg(not(target_os = "android"))]
        if ui
            .small_button("...")
            .on_hover_text(crate::i18n::t(world, "widget-browse-file"))
            .clicked()
            && let Some(picked) = rfd::FileDialog::new()
                .set_title(label)
                .add_filter("RON", &["ron"])
                .add_filter("All files", &["*"])
                .pick_file()
        {
            *path = picked.display().to_string();
            changed = true;
        }
    });
    changed
}
