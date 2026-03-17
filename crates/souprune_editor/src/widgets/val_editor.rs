//! # Value<T> Editor
//!
//! # Value<T> 编辑器
//!
//! ## Module Overview
//!
//! ## 模块概述
//!
//! Value<T> editor widget for the editor.
//! Supports switching between static values and FRE expression modes.
//!
//! 编辑器的 Value<T> 编辑器组件。
//! 支持在静态值和 FRE 表达式两种模式之间切换。

use souprune::core::sequencer::chapter_schema::Value;

/// 渲染 `Value<f32>` 编辑器：静态值/表达式切换。
// TODO: i18n — add `world: &World` parameter and replace "静態"/"表達式" with
//       `t(world, "widget-static")` / `t(world, "widget-expression")`.
//       Deferred because it cascades through view_widgets.rs (20+ call sites).
pub fn edit_val_f32(ui: &mut egui::Ui, label: &str, val: &mut Value<f32>) -> bool {
    let mut changed = false;
    ui.horizontal(|ui| {
        ui.label(format!("{label}:"));
        let is_expr = matches!(val, Value::Expr(_));
        if ui.selectable_label(!is_expr, "Static").clicked() && is_expr {
            *val = Value::Static(0.0);
            changed = true;
        }
        if ui.selectable_label(is_expr, "Expr").clicked() && !is_expr {
            *val = Value::Expr(String::new());
            changed = true;
        }

        match val {
            Value::Static(v) => {
                if ui.add(egui::DragValue::new(v).speed(0.1)).changed() {
                    changed = true;
                }
            }
            Value::Expr(expr) => {
                if ui.text_edit_singleline(expr).changed() {
                    changed = true;
                }
            }
        }
    });
    changed
}

/// 渲染 `Value<bool>` 编辑器。
pub fn edit_val_bool(ui: &mut egui::Ui, label: &str, val: &mut Value<bool>) -> bool {
    let mut changed = false;
    ui.horizontal(|ui| {
        ui.label(format!("{label}:"));
        let is_expr = matches!(val, Value::Expr(_));
        if ui.selectable_label(!is_expr, "S").clicked() && is_expr {
            *val = Value::Static(false);
            changed = true;
        }
        if ui.selectable_label(is_expr, "E").clicked() && !is_expr {
            *val = Value::Expr(String::new());
            changed = true;
        }

        match val {
            Value::Static(v) => {
                if ui.checkbox(v, "").changed() {
                    changed = true;
                }
            }
            Value::Expr(expr) => {
                if ui.text_edit_singleline(expr).changed() {
                    changed = true;
                }
            }
        }
    });
    changed
}
