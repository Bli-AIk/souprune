//! # Val<T> 编辑器
//!
//! 支持静态值和 FRE 表达式两种模式的切换编辑器。

use souprune::core::sequencer::chapter_schema::Val;

/// 渲染 `Val<f32>` 编辑器：静态值/表达式切换。
pub fn edit_val_f32(ui: &mut egui::Ui, label: &str, val: &mut Val<f32>) -> bool {
    let mut changed = false;
    ui.horizontal(|ui| {
        ui.label(format!("{label}:"));
        let is_expr = matches!(val, Val::Expr(_));
        if ui.selectable_label(!is_expr, "静态").clicked() && is_expr {
            *val = Val::Static(0.0);
            changed = true;
        }
        if ui.selectable_label(is_expr, "表达式").clicked() && !is_expr {
            *val = Val::Expr(String::new());
            changed = true;
        }

        match val {
            Val::Static(v) => {
                if ui.add(egui::DragValue::new(v).speed(0.1)).changed() {
                    changed = true;
                }
            }
            Val::Expr(expr) => {
                if ui.text_edit_singleline(expr).changed() {
                    changed = true;
                }
            }
        }
    });
    changed
}

/// 渲染 `Val<bool>` 编辑器。
pub fn edit_val_bool(ui: &mut egui::Ui, label: &str, val: &mut Val<bool>) -> bool {
    let mut changed = false;
    ui.horizontal(|ui| {
        ui.label(format!("{label}:"));
        let is_expr = matches!(val, Val::Expr(_));
        if ui.selectable_label(!is_expr, "S").clicked() && is_expr {
            *val = Val::Static(false);
            changed = true;
        }
        if ui.selectable_label(is_expr, "E").clicked() && !is_expr {
            *val = Val::Expr(String::new());
            changed = true;
        }

        match val {
            Val::Static(v) => {
                if ui.checkbox(v, "").changed() {
                    changed = true;
                }
            }
            Val::Expr(expr) => {
                if ui.text_edit_singleline(expr).changed() {
                    changed = true;
                }
            }
        }
    });
    changed
}
