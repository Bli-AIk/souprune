//! # Value<T> Editor
//!
//! # Value<T> 编辑器
//!
//! ## Module Overview
//!
//! ## 模块概述
//!
//! Value<T> / Val<T> editor widget for the editor.
//! Supports switching between static values and FRE expression modes.
//!
//! 编辑器的 Value<T> / Val<T> 编辑器组件。
//! 支持在静态值和 FRE 表达式两种模式之间切换。

use souprune::editor_api::values::RuntimeExprValue as Value;
use souprune_schema::Val;

pub trait ExprValue<T> {
    fn is_expr(&self) -> bool;
    fn set_static(&mut self, value: T);
    fn set_expr(&mut self, expr: String);
    fn static_mut(&mut self) -> Option<&mut T>;
    fn expr_mut(&mut self) -> Option<&mut String>;
}

impl<T> ExprValue<T> for Value<T> {
    fn is_expr(&self) -> bool {
        matches!(self, Value::Expr(_))
    }

    fn set_static(&mut self, value: T) {
        *self = Value::Static(value);
    }

    fn set_expr(&mut self, expr: String) {
        *self = Value::Expr(expr);
    }

    fn static_mut(&mut self) -> Option<&mut T> {
        match self {
            Value::Static(value) => Some(value),
            Value::Expr(_) => None,
        }
    }

    fn expr_mut(&mut self) -> Option<&mut String> {
        match self {
            Value::Expr(expr) => Some(expr),
            Value::Static(_) => None,
        }
    }
}

impl<T> ExprValue<T> for Val<T> {
    fn is_expr(&self) -> bool {
        matches!(self, Val::Expr(_))
    }

    fn set_static(&mut self, value: T) {
        *self = Val::Static(value);
    }

    fn set_expr(&mut self, expr: String) {
        *self = Val::Expr(expr);
    }

    fn static_mut(&mut self) -> Option<&mut T> {
        match self {
            Val::Static(value) => Some(value),
            Val::Expr(_) => None,
        }
    }

    fn expr_mut(&mut self) -> Option<&mut String> {
        match self {
            Val::Expr(expr) => Some(expr),
            Val::Static(_) => None,
        }
    }
}

/// 渲染 `Value<f32>` 编辑器：静态值/表达式切换。
// TODO: i18n — add `world: &World` parameter and replace "静態"/"表達式" with
//       `t(world, "widget-static")` / `t(world, "widget-expression")`.
//       Deferred because it cascades through view_widgets.rs (20+ call sites).
pub fn edit_val_f32<V: ExprValue<f32>>(ui: &mut egui::Ui, label: &str, val: &mut V) -> bool {
    let mut changed = false;
    ui.horizontal(|ui| {
        ui.label(format!("{label}:"));
        let is_expr = val.is_expr();
        if ui.selectable_label(!is_expr, "Static").clicked() && is_expr {
            val.set_static(0.0);
            changed = true;
        }
        if ui.selectable_label(is_expr, "Expr").clicked() && !is_expr {
            val.set_expr(String::new());
            changed = true;
        }

        if let Some(v) = val.static_mut() {
            if ui.add(egui::DragValue::new(v).speed(0.1)).changed() {
                changed = true;
            }
        } else if let Some(expr) = val.expr_mut()
            && ui.text_edit_singleline(expr).changed()
        {
            changed = true;
        }
    });
    changed
}

/// 渲染 `Value<bool>` 编辑器。
pub fn edit_val_bool<V: ExprValue<bool>>(ui: &mut egui::Ui, label: &str, val: &mut V) -> bool {
    let mut changed = false;
    ui.horizontal(|ui| {
        ui.label(format!("{label}:"));
        let is_expr = val.is_expr();
        if ui.selectable_label(!is_expr, "S").clicked() && is_expr {
            val.set_static(false);
            changed = true;
        }
        if ui.selectable_label(is_expr, "E").clicked() && !is_expr {
            val.set_expr(String::new());
            changed = true;
        }

        if let Some(v) = val.static_mut() {
            if ui.checkbox(v, "").changed() {
                changed = true;
            }
        } else if let Some(expr) = val.expr_mut()
            && ui.text_edit_singleline(expr).changed()
        {
            changed = true;
        }
    });
    changed
}
