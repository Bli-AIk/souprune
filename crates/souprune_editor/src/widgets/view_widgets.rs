//! View 编辑器专用组件。
//!
//! 提供 Vec3Tuple、ColorTuple、Transform、Tag 列表等编辑控件。

use souprune::core::sequencer::chapter_schema::Val;
use souprune::core::view::layout::serde_types::{
    SerializableColor, SerializableTransform, SerializableVec2, SerializableVec3, ViewFontDef,
};

use super::val_editor::edit_val_f32;

/// 编辑 Vec3Tuple (Val<f32>, Val<f32>, Val<f32>)。
pub fn edit_vec3(
    ui: &mut egui::Ui,
    label: &str,
    vec: &mut SerializableVec3,
) -> bool {
    let mut changed = false;
    ui.horizontal(|ui| {
        ui.label(format!("{label}:"));
    });
    ui.indent(label, |ui| {
        changed |= edit_val_f32(ui, "X", &mut vec.0);
        changed |= edit_val_f32(ui, "Y", &mut vec.1);
        changed |= edit_val_f32(ui, "Z", &mut vec.2);
    });
    changed
}

/// 编辑 Vec2Tuple (Val<f32>, Val<f32>)。
pub fn edit_vec2(
    ui: &mut egui::Ui,
    label: &str,
    vec: &mut SerializableVec2,
) -> bool {
    let mut changed = false;
    ui.horizontal(|ui| {
        ui.label(format!("{label}:"));
    });
    ui.indent(label, |ui| {
        changed |= edit_val_f32(ui, "X", &mut vec.0);
        changed |= edit_val_f32(ui, "Y", &mut vec.1);
    });
    changed
}

/// 编辑 ColorTuple (Val<f32>, Val<f32>, Val<f32>, Val<f32>) — RGBA。
pub fn edit_color(
    ui: &mut egui::Ui,
    label: &str,
    color: &mut SerializableColor,
) -> bool {
    let mut changed = false;
    ui.horizontal(|ui| {
        ui.label(format!("{label}:"));
        // 静态颜色预览方块
        if let (Val::Static(r), Val::Static(g), Val::Static(b), Val::Static(a)) =
            (&color.0, &color.1, &color.2, &color.3)
        {
            let preview = egui::Color32::from_rgba_unmultiplied(
                (*r * 255.0) as u8,
                (*g * 255.0) as u8,
                (*b * 255.0) as u8,
                (*a * 255.0) as u8,
            );
            let (rect, _) = ui.allocate_exact_size(
                egui::vec2(16.0, 16.0),
                egui::Sense::hover(),
            );
            ui.painter().rect_filled(rect, 2.0, preview);
        }
    });
    ui.indent(label, |ui| {
        changed |= edit_val_f32(ui, "R", &mut color.0);
        changed |= edit_val_f32(ui, "G", &mut color.1);
        changed |= edit_val_f32(ui, "B", &mut color.2);
        changed |= edit_val_f32(ui, "A", &mut color.3);
    });
    changed
}

/// 编辑 Option<SerializableColor>。
pub fn edit_option_color(
    ui: &mut egui::Ui,
    label: &str,
    opt: &mut Option<SerializableColor>,
) -> bool {
    let mut changed = false;
    let mut enabled = opt.is_some();
    ui.horizontal(|ui| {
        if ui.checkbox(&mut enabled, label).changed() {
            if enabled {
                *opt = Some((
                    Val::Static(1.0),
                    Val::Static(1.0),
                    Val::Static(1.0),
                    Val::Static(1.0),
                ));
            } else {
                *opt = None;
            }
            changed = true;
        }
    });
    if let Some(color) = opt {
        changed |= edit_color(ui, &format!("{label}_inner"), color);
    }
    changed
}

/// 编辑 SerializableTransform。
pub fn edit_transform(
    ui: &mut egui::Ui,
    label: &str,
    transform: &mut SerializableTransform,
) -> bool {
    let mut changed = false;
    ui.collapsing(label, |ui| {
        // Translation
        let mut has_t = transform.translation.is_some();
        if ui.checkbox(&mut has_t, "Translation").changed() {
            transform.translation = if has_t {
                Some((Val::Static(0.0), Val::Static(0.0), Val::Static(0.0)))
            } else {
                None
            };
            changed = true;
        }
        if let Some(t) = &mut transform.translation {
            changed |= edit_vec3(ui, "pos", t);
        }

        // Scale
        let mut has_s = transform.scale.is_some();
        if ui.checkbox(&mut has_s, "Scale").changed() {
            transform.scale = if has_s {
                Some((Val::Static(1.0), Val::Static(1.0), Val::Static(1.0)))
            } else {
                None
            };
            changed = true;
        }
        if let Some(s) = &mut transform.scale {
            changed |= edit_vec3(ui, "scale", s);
        }

        // Rotation
        let mut has_r = transform.rotation.is_some();
        if ui.checkbox(&mut has_r, "Rotation").changed() {
            transform.rotation = if has_r { Some(0.0) } else { None };
            changed = true;
        }
        if let Some(r) = &mut transform.rotation
            && ui
                .add(egui::DragValue::new(r).speed(1.0).suffix("deg"))
                .changed()
            {
                changed = true;
            }
    });
    changed
}

/// 编辑 Option<SerializableTransform>。
pub fn edit_option_transform(
    ui: &mut egui::Ui,
    label: &str,
    opt: &mut Option<SerializableTransform>,
) -> bool {
    let mut changed = false;
    let mut enabled = opt.is_some();
    if ui.checkbox(&mut enabled, label).changed() {
        if enabled {
            *opt = Some(SerializableTransform {
                translation: None,
                rotation: None,
                scale: None,
            });
        } else {
            *opt = None;
        }
        changed = true;
    }
    if let Some(t) = opt {
        changed |= edit_transform(ui, &format!("{label}_inner"), t);
    }
    changed
}

/// 编辑 Option<SerializableVec2>。
pub fn edit_option_vec2_tuple(
    ui: &mut egui::Ui,
    label: &str,
    opt: &mut Option<SerializableVec2>,
) -> bool {
    let mut changed = false;
    let mut enabled = opt.is_some();
    if ui.checkbox(&mut enabled, label).changed() {
        if enabled {
            *opt = Some((Val::Static(0.5), Val::Static(0.5)));
        } else {
            *opt = None;
        }
        changed = true;
    }
    if let Some(v) = opt {
        changed |= edit_vec2(ui, &format!("{label}_v"), v);
    }
    changed
}

/// 编辑标签列表 (Vec<String>)。
pub fn edit_tag_list(ui: &mut egui::Ui, label: &str, tags: &mut Vec<String>) -> bool {
    let mut changed = false;
    ui.horizontal_wrapped(|ui| {
        ui.label(format!("{label}:"));
        let mut to_remove = None;
        for (i, tag) in tags.iter().enumerate() {
            let resp = ui.small_button(format!("{tag} x"));
            if resp.clicked() {
                to_remove = Some(i);
            }
        }
        if let Some(i) = to_remove {
            tags.remove(i);
            changed = true;
        }
        if ui.small_button("+").clicked() {
            tags.push(String::from("new_tag"));
            changed = true;
        }
    });
    changed
}

/// 编辑 ViewFontDef 枚举。
pub fn edit_font_def(ui: &mut egui::Ui, label: &str, font: &mut ViewFontDef) -> bool {
    let mut changed = false;
    ui.horizontal(|ui| {
        ui.label(format!("{label}:"));
        let current = match font {
            ViewFontDef::DeterminationMono => "DeterminationMono",
            ViewFontDef::DeterminationSans => "DeterminationSans",
            ViewFontDef::Hud => "Hud",
            ViewFontDef::BattleHud => "BattleHud",
        };
        egui::ComboBox::from_id_salt(label)
            .selected_text(current)
            .show_ui(ui, |ui| {
                if ui.selectable_label(matches!(font, ViewFontDef::DeterminationMono), "DeterminationMono").clicked() {
                    *font = ViewFontDef::DeterminationMono;
                    changed = true;
                }
                if ui.selectable_label(matches!(font, ViewFontDef::DeterminationSans), "DeterminationSans").clicked() {
                    *font = ViewFontDef::DeterminationSans;
                    changed = true;
                }
                if ui.selectable_label(matches!(font, ViewFontDef::Hud), "Hud").clicked() {
                    *font = ViewFontDef::Hud;
                    changed = true;
                }
                if ui.selectable_label(matches!(font, ViewFontDef::BattleHud), "BattleHud").clicked() {
                    *font = ViewFontDef::BattleHud;
                    changed = true;
                }
            });
    });
    changed
}

/// 编辑 HashMap<String, String> 键值对。
pub fn edit_string_string_map(
    ui: &mut egui::Ui,
    label: &str,
    map: &mut std::collections::HashMap<String, String>,
) -> bool {
    let mut changed = false;
    ui.collapsing(label, |ui| {
        let mut to_remove = None;
        let mut keys: Vec<String> = map.keys().cloned().collect();
        keys.sort();
        for key in &keys {
            ui.horizontal(|ui| {
                ui.label(format!("{key}:"));
                if let Some(val) = map.get_mut(key)
                    && ui.text_edit_singleline(val).changed() {
                        changed = true;
                    }
                if ui.small_button("x").clicked() {
                    to_remove = Some(key.clone());
                }
            });
        }
        if let Some(k) = to_remove {
            map.remove(&k);
            changed = true;
        }
        if ui.small_button("+ 添加").clicked() {
            let new_key = format!("state_{}", map.len());
            map.insert(new_key, String::new());
            changed = true;
        }
    });
    changed
}

/// 编辑可选表达式字符串 (visible_when)。
pub fn edit_expression(
    ui: &mut egui::Ui,
    label: &str,
    expr: &mut Option<String>,
) -> bool {
    let mut changed = false;
    let mut enabled = expr.is_some();
    ui.horizontal(|ui| {
        if ui.checkbox(&mut enabled, label).changed() {
            if enabled {
                *expr = Some(String::from("true"));
            } else {
                *expr = None;
            }
            changed = true;
        }
        if let Some(e) = expr
            && ui.text_edit_singleline(e).changed() {
                changed = true;
            }
    });
    changed
}
