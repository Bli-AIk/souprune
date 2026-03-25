//! Node property editors for the View Editor panel.
use crate::i18n::t;
use crate::widgets::property_editors::{edit_option_string, labeled_text};
use crate::widgets::view_widgets::{
    edit_color, edit_expression, edit_font_def, edit_option_color, edit_option_transform,
    edit_option_vec2_tuple, edit_string_string_map, edit_tag_list, edit_vec2, edit_vec3,
};
use bevy::prelude::*;
use souprune_schema::view::{
    DataRequirement, InitialFactValue, RepeatDef, SpriteDef, StateSpriteConfig, TextDef,
    ViewBoxLogicDef, ViewNodeDef, Visual,
};

pub(super) fn edit_node_basics(world: &World, ui: &mut egui::Ui, node: &mut ViewNodeDef) -> bool {
    let mut changed = false;
    egui::CollapsingHeader::new(t(world, "view-basics"))
        .default_open(true)
        .show(ui, |ui| {
            changed |= labeled_text(ui, &t(world, "prop-name"), &mut node.name);
            changed |= edit_tag_list(ui, &t(world, "prop-tags"), &mut node.tags);
            changed |= edit_expression(ui, "visible_when", &mut node.visible_when);
        });
    changed
}

pub(super) fn edit_node_sprite(
    world: &World,
    ui: &mut egui::Ui,
    sprite_opt: &mut Option<SpriteDef>,
) -> bool {
    let mut changed = false;
    let mut has = sprite_opt.is_some();
    if ui.checkbox(&mut has, "Sprite").changed() {
        if has {
            *sprite_opt = Some(SpriteDef {
                visual: Visual::default(),
                initial_state: None,
                color: None,
                flip_x: false,
                flip_y: false,
                transform: None,
                pivot: None,
                frame_duration: None,
                visible_when: None,
                material: None,
            });
        } else {
            *sprite_opt = None;
        }
        changed = true;
    }
    let Some(sprite) = sprite_opt else {
        return changed;
    };
    egui::CollapsingHeader::new("  Sprite")
        .default_open(true)
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label("visual:");
                if ui.text_edit_singleline(&mut sprite.visual.0).changed() {
                    changed = true;
                }
            });
            changed |= edit_option_color(ui, &t(world, "view-color"), &mut sprite.color);
            ui.horizontal(|ui| {
                if ui.checkbox(&mut sprite.flip_x, "flip_x").changed() {
                    changed = true;
                }
                if ui.checkbox(&mut sprite.flip_y, "flip_y").changed() {
                    changed = true;
                }
            });
            changed |= edit_option_transform(ui, "Transform", &mut sprite.transform);
            changed |= edit_option_vec2_tuple(ui, "Pivot", &mut sprite.pivot);
            changed |= edit_expression(ui, "visible_when", &mut sprite.visible_when);
            changed |= edit_option_string(ui, "initial_state", &mut sprite.initial_state);
        });
    changed
}

pub(super) fn edit_node_state_sprite(
    world: &World,
    ui: &mut egui::Ui,
    ss_opt: &mut Option<StateSpriteConfig>,
) -> bool {
    let Some(ss) = ss_opt else { return false };
    let mut changed = false;
    egui::CollapsingHeader::new("State Sprite")
        .default_open(true)
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label("default:");
                if ui.text_edit_singleline(&mut ss.default).changed() {
                    changed = true;
                }
            });
            changed |= edit_string_string_map(ui, &t(world, "prop-variants"), &mut ss.variants);
            changed |= edit_option_transform(ui, "Transform", &mut ss.transform);
            changed |= edit_expression(ui, "visible_when", &mut ss.visible_when);
        });
    changed
}

fn render_text_def_editor(
    ui: &mut egui::Ui,
    world: &World,
    text: &mut TextDef,
    i: usize,
    to_remove: &mut Option<usize>,
) -> bool {
    let mut changed = false;
    ui.group(|ui| {
        ui.horizontal(|ui| {
            ui.label("id:");
            if ui.text_edit_singleline(&mut text.id).changed() {
                changed = true;
            }
            if ui.small_button("x").clicked() {
                *to_remove = Some(i);
            }
        });
        changed |= edit_option_string(ui, "content", &mut text.content);
        changed |= edit_color(ui, &t(world, "view-color"), &mut text.color);
        changed |= edit_font_def(ui, &t(world, "view-font"), &mut text.font);
        changed |= edit_vec2(ui, "world_scale", &mut text.world_scale);
        changed |= edit_expression(ui, "visible_when", &mut text.visible_when);
    });
    changed
}

pub(super) fn edit_node_texts(world: &World, ui: &mut egui::Ui, texts: &mut Vec<TextDef>) -> bool {
    if texts.is_empty() {
        return false;
    }
    let mut changed = false;
    egui::CollapsingHeader::new("Text")
        .default_open(true)
        .show(ui, |ui| {
            let mut to_remove = None;
            for (i, text) in texts.iter_mut().enumerate() {
                ui.push_id(i, |ui| {
                    changed |= render_text_def_editor(ui, world, text, i, &mut to_remove);
                });
            }
            if let Some(i) = to_remove {
                texts.remove(i);
                changed = true;
            }
        });
    changed
}

pub(super) fn edit_node_view_box(
    world: &World,
    ui: &mut egui::Ui,
    vb_opt: &mut Option<ViewBoxLogicDef>,
) -> bool {
    let Some(vb) = vb_opt else { return false };
    let mut changed = false;
    egui::CollapsingHeader::new("ViewBox")
        .default_open(true)
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(t(world, "view-width"));
                changed |= ui
                    .add(egui::DragValue::new(&mut vb.width).speed(1.0))
                    .changed();
                ui.label(t(world, "view-height"));
                changed |= ui
                    .add(egui::DragValue::new(&mut vb.height).speed(1.0))
                    .changed();
            });
            ui.horizontal(|ui| {
                ui.label("border:");
                changed |= ui
                    .add(egui::DragValue::new(&mut vb.border_width).speed(0.5))
                    .changed();
            });
            changed |= edit_vec3(ui, "offset", &mut vb.offset);
            changed |= edit_option_color(ui, "fill_color", &mut vb.fill_color);
            changed |= edit_option_string(ui, "structure_file", &mut vb.structure_file);
        });
    changed
}

fn edit_repeat_limit(ui: &mut egui::Ui, repeat: &mut RepeatDef) -> bool {
    let mut changed = false;
    let mut has_limit = repeat.limit.is_some();
    if ui.checkbox(&mut has_limit, "limit").changed() {
        repeat.limit = if has_limit { Some(10) } else { None };
        changed = true;
    }
    if let Some(limit) = &mut repeat.limit {
        let mut v = *limit as f64;
        if ui
            .add(egui::DragValue::new(&mut v).speed(1.0).range(0.0..=1000.0))
            .changed()
        {
            *limit = v as usize;
            changed = true;
        }
    }
    changed
}

pub(super) fn edit_node_repeat(
    world: &World,
    ui: &mut egui::Ui,
    repeat_opt: &mut Option<RepeatDef>,
) -> bool {
    let Some(repeat) = repeat_opt else {
        return false;
    };
    let mut changed = false;
    egui::CollapsingHeader::new(t(world, "view-repeat"))
        .default_open(true)
        .show(ui, |ui| {
            changed |= labeled_text(ui, "source", &mut repeat.source);
            ui.horizontal(|ui| {
                changed |= edit_repeat_limit(ui, repeat);
            });
            changed |= edit_option_string(ui, "index_var", &mut repeat.index_var);
            changed |= edit_option_string(ui, "item_var", &mut repeat.item_var);
        });
    changed
}

fn render_data_requirement_row(
    ui: &mut egui::Ui,
    req: &mut DataRequirement,
    i: usize,
    to_remove: &mut Option<usize>,
) -> bool {
    let mut changed = false;
    ui.horizontal(|ui| {
        match req {
            DataRequirement::File(path) => {
                ui.label("File:");
                if ui.text_edit_singleline(path).changed() {
                    changed = true;
                }
            }
            DataRequirement::Interface { interface, expects } => {
                ui.label("Interface:");
                if ui.text_edit_singleline(interface).changed() {
                    changed = true;
                }
                ui.label(format!("({})", expects.join(", ")));
            }
        }
        if ui.small_button("x").clicked() {
            *to_remove = Some(i);
        }
    });
    changed
}

pub(super) fn edit_data_requirements(
    world: &World,
    ui: &mut egui::Ui,
    requires: &mut Vec<DataRequirement>,
) -> bool {
    if requires.is_empty() {
        return false;
    }
    let mut changed = false;
    egui::CollapsingHeader::new(t(world, "view-data-requirements"))
        .default_open(false)
        .show(ui, |ui| {
            let mut to_remove = None;
            for (i, req) in requires.iter_mut().enumerate() {
                ui.push_id(i, |ui| {
                    changed |= render_data_requirement_row(ui, req, i, &mut to_remove);
                });
            }
            if let Some(i) = to_remove {
                requires.remove(i);
                changed = true;
            }
        });
    changed
}

pub(super) fn edit_initial_facts(
    world: &World,
    ui: &mut egui::Ui,
    facts_opt: &mut Option<std::collections::HashMap<String, InitialFactValue>>,
) -> bool {
    let Some(facts) = facts_opt else {
        return false;
    };
    let mut changed = false;
    egui::CollapsingHeader::new(t(world, "view-initial-facts"))
        .default_open(false)
        .show(ui, |ui| {
            let keys: Vec<String> = facts.keys().cloned().collect();
            for key in &keys {
                if let Some(val) = facts.get_mut(key) {
                    changed |= edit_initial_fact_value(ui, key, val);
                }
            }
        });
    changed
}

fn edit_initial_fact_value(ui: &mut egui::Ui, key: &str, val: &mut InitialFactValue) -> bool {
    let mut changed = false;
    ui.horizontal(|ui| {
        ui.label(format!("{key}:"));
        match val {
            InitialFactValue::Int(v) => {
                let mut f = *v as f64;
                if ui.add(egui::DragValue::new(&mut f).speed(1.0)).changed() {
                    *v = f as i64;
                    changed = true;
                }
            }
            InitialFactValue::Float(v) => {
                changed |= ui.add(egui::DragValue::new(v).speed(0.1)).changed();
            }
            InitialFactValue::Bool(v) => {
                changed |= ui.checkbox(v, "").changed();
            }
            InitialFactValue::String(s) => {
                changed |= ui.text_edit_singleline(s).changed();
            }
            InitialFactValue::StringList(list) => {
                ui.label(format!("[{}]", list.join(", ")));
            }
            InitialFactValue::IntList(list) => {
                ui.label(format!("{list:?}"));
            }
        }
    });
    changed
}

// ─── Tree path helpers ───────────────────────────────────────
pub(super) fn find_node_by_path<'a>(
    roots: &'a [ViewNodeDef],
    path: &[usize],
) -> Option<&'a ViewNodeDef> {
    if path.is_empty() {
        return None;
    }
    let mut current = roots.get(path[0])?;
    for &idx in &path[1..] {
        current = current.children.get(idx)?;
    }
    Some(current)
}

pub(super) fn find_node_by_path_mut<'a>(
    roots: &'a mut [ViewNodeDef],
    path: &[usize],
) -> Option<&'a mut ViewNodeDef> {
    if path.is_empty() {
        return None;
    }
    let mut current = roots.get_mut(path[0])?;
    for &idx in &path[1..] {
        current = current.children.get_mut(idx)?;
    }
    Some(current)
}

/// 获取路径所在父节点的 children 切片和该节点的索引。
pub(super) fn parent_children_mut<'a>(
    roots: &'a mut Vec<ViewNodeDef>,
    path: &[usize],
) -> Option<(&'a mut Vec<ViewNodeDef>, usize)> {
    if path.is_empty() {
        return None;
    }
    if path.len() == 1 {
        return Some((roots, path[0]));
    }
    let parent_path = &path[..path.len() - 1];
    let idx = *path.last().unwrap();
    let parent = find_node_by_path_mut(roots, parent_path)?;
    Some((&mut parent.children, idx))
}
