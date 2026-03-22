//! # Property Editors
//!
//! Generic property editor widgets for the editor.

use bevy::prelude::*;

use crate::i18n::t;

mod action_editors;
mod fact_editors;

pub use action_editors::{
    chapter_type_label, edit_camera_action, edit_player_action, edit_ui_action,
};
pub use fact_editors::{edit_fact_condition, edit_fact_modifications, edit_fact_value_match};

pub fn labeled_text(ui: &mut egui::Ui, label: &str, value: &mut String) -> bool {
    ui.label(format!("{label}:"));
    ui.text_edit_singleline(value).changed()
}

pub fn labeled_drag(
    ui: &mut egui::Ui,
    label: &str,
    value: &mut f32,
    range: std::ops::RangeInclusive<f32>,
    speed: f64,
) -> bool {
    ui.label(format!("{label}:"));
    ui.add(egui::DragValue::new(value).range(range).speed(speed))
        .changed()
}

pub fn edit_option_string(ui: &mut egui::Ui, label: &str, opt: &mut Option<String>) -> bool {
    let mut changed = false;
    let mut enabled = opt.is_some();
    ui.horizontal(|ui| {
        if ui.checkbox(&mut enabled, label).changed() {
            if enabled {
                *opt = Some(String::new());
            } else {
                *opt = None;
            }
            changed = true;
        }
        if let Some(val) = opt
            && ui.text_edit_singleline(val).changed()
        {
            changed = true;
        }
    });
    changed
}

pub fn edit_option_f32(ui: &mut egui::Ui, label: &str, opt: &mut Option<f32>) -> bool {
    let mut changed = false;
    let mut enabled = opt.is_some();
    ui.horizontal(|ui| {
        if ui.checkbox(&mut enabled, label).changed() {
            if enabled {
                *opt = Some(0.0);
            } else {
                *opt = None;
            }
            changed = true;
        }
        if let Some(val) = opt
            && ui
                .add(egui::DragValue::new(val).range(0.0..=f32::MAX).speed(0.1))
                .changed()
        {
            changed = true;
        }
    });
    changed
}

pub fn edit_option_vec2(ui: &mut egui::Ui, label: &str, opt: &mut Option<(f32, f32)>) -> bool {
    let mut changed = false;
    let mut enabled = opt.is_some();
    if ui.checkbox(&mut enabled, label).changed() {
        if enabled {
            *opt = Some((0.0, 0.0));
        } else {
            *opt = None;
        }
        changed = true;
    }
    if let Some((x, y)) = opt {
        ui.horizontal(|ui| {
            if ui
                .add(egui::DragValue::new(x).prefix("X: ").speed(1.0))
                .changed()
            {
                changed = true;
            }
            if ui
                .add(egui::DragValue::new(y).prefix("Y: ").speed(1.0))
                .changed()
            {
                changed = true;
            }
        });
    }
    changed
}

pub fn edit_hashmap_string<V: std::fmt::Debug>(
    ui: &mut egui::Ui,
    label: &str,
    map: &mut std::collections::HashMap<String, V>,
) -> bool {
    ui.label(format!("{label}: {}", map.len()));
    for (k, v) in map.iter() {
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new(k).monospace());
            ui.label(format!("{v:?}"));
        });
    }
    false
}

pub fn edit_hashmap_string_flat(
    ui: &mut egui::Ui,
    label: &str,
    map: &mut std::collections::HashMap<String, String>,
    world: &World,
) -> bool {
    let mut changed = false;
    ui.label(format!("{label}: {}", map.len()));

    let keys: Vec<String> = map.keys().cloned().collect();
    let mut to_remove = None;
    for key in &keys {
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new(key).monospace());
            if let Some(val) = map.get_mut(key)
                && ui.text_edit_singleline(val).changed()
            {
                changed = true;
            }
            if ui.small_button("✕").clicked() {
                to_remove = Some(key.clone());
            }
        });
    }
    if let Some(key) = to_remove {
        map.remove(&key);
        changed = true;
    }

    ui.horizontal(|ui| {
        if ui.small_button(t(world, "action-add-item")).clicked() {
            let new_key = format!("key_{}", map.len());
            map.insert(new_key, String::new());
            changed = true;
        }
    });

    changed
}

pub fn edit_string_list(
    ui: &mut egui::Ui,
    label: &str,
    list: &mut Vec<String>,
    world: &World,
) -> bool {
    let mut changed = false;
    ui.label(format!("{label}:"));

    let mut to_remove = None;
    for (i, item) in list.iter_mut().enumerate() {
        ui.horizontal(|ui| {
            if ui.text_edit_singleline(item).changed() {
                changed = true;
            }
            if ui.small_button("✕").clicked() {
                to_remove = Some(i);
            }
        });
    }
    if let Some(idx) = to_remove {
        list.remove(idx);
        changed = true;
    }
    if ui.small_button(t(world, "action-add-item")).clicked() {
        list.push(String::new());
        changed = true;
    }

    changed
}
