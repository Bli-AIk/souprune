//! # 文件路径选择器
//!
//! 提供文件路径编辑 UI：文本框 + 浏览按钮。

/// 渲染文件路径编辑器：文本框 + 浏览按钮。
///
/// 返回路径是否被修改。
pub fn edit_file_path(ui: &mut egui::Ui, label: &str, path: &mut String) -> bool {
    let mut changed = false;
    ui.horizontal(|ui| {
        ui.label(format!("{label}:"));
        if ui.text_edit_singleline(path).changed() {
            changed = true;
        }
        #[cfg(not(target_os = "android"))]
        if ui.small_button("...").on_hover_text("浏览文件").clicked()
            && let Some(picked) = rfd::FileDialog::new()
                .set_title(label)
                .add_filter("RON", &["ron"])
                .add_filter("所有文件", &["*"])
                .pick_file()
        {
            *path = picked.display().to_string();
            changed = true;
        }
    });
    changed
}
