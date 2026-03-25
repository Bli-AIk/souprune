//! # fact_editors.rs
//!
//! # fact_editors.rs 文件
//!
//! ## Module Overview
//!
//! ## 模块概述
//!
//! Contains the property-editor widgets for FRE-style fact conditions and
//! modifications. It is the place where inspector panels turn structured fact expressions into
//! editable egui controls instead of hand-building those rows everywhere.
//!
//! 放的是 FRE 风格事实条件和修改操作的属性编辑控件。检查器面板会在这里把结构化的
//! 事实表达式转换成可编辑的 egui 控件，而不是在各处手写这些行编辑逻辑。

use bevy::prelude::*;
use bevy_workbench::i18n::FluentArgs;
use souprune_schema::sequence::{FactCondition, FactModificationDef, FactValueMatch};

use crate::i18n::{t, t_args};

use super::labeled_text;

fn select_fact_value_type(
    ui: &mut egui::Ui,
    value: &mut FactValueMatch,
    current: usize,
    variants: &[&str],
) -> bool {
    let mut changed = false;
    for (i, v) in variants.iter().enumerate() {
        if ui.selectable_label(current == i, *v).clicked() && current != i {
            *value = match i {
                0 => FactValueMatch::Int(0),
                1 => FactValueMatch::Float(0.0),
                2 => FactValueMatch::Bool(false),
                3 => FactValueMatch::String(String::new()),
                _ => FactValueMatch::Expr(String::new()),
            };
            changed = true;
        }
    }
    changed
}

pub fn edit_fact_value_match(ui: &mut egui::Ui, label: &str, value: &mut FactValueMatch) -> bool {
    let mut changed = false;
    ui.label(format!("{label}:"));

    let variants = ["Int", "Float", "Bool", "String", "Expr"];
    let current = match value {
        FactValueMatch::Int(_) => 0,
        FactValueMatch::Float(_) => 1,
        FactValueMatch::Bool(_) => 2,
        FactValueMatch::String(_) => 3,
        FactValueMatch::Expr(_) => 4,
    };

    ui.horizontal(|ui| {
        egui::ComboBox::from_id_salt(egui::Id::new(label).with("type"))
            .selected_text(variants[current])
            .show_ui(ui, |ui| {
                changed |= select_fact_value_type(ui, value, current, &variants);
            });

        match value {
            FactValueMatch::Int(v) => {
                changed |= ui.add(egui::DragValue::new(v)).changed();
            }
            FactValueMatch::Float(v) => {
                changed |= ui.add(egui::DragValue::new(v).speed(0.1)).changed();
            }
            FactValueMatch::Bool(v) => {
                changed |= ui.checkbox(v, "").changed();
            }
            FactValueMatch::String(v) | FactValueMatch::Expr(v) => {
                changed |= ui.text_edit_singleline(v).changed();
            }
        }
    });

    changed
}

fn render_sub_conditions(ui: &mut egui::Ui, subs: &mut Vec<FactCondition>, world: &World) -> bool {
    let mut changed = false;
    for (i, sub) in subs.iter_mut().enumerate() {
        ui.push_id(i, |ui| {
            ui.indent(egui::Id::new(("sub_cond", i)), |ui| {
                changed |= edit_fact_condition(ui, sub, world);
            });
        });
    }
    if ui
        .small_button(t(world, "action-add-subcondition"))
        .clicked()
    {
        subs.push(FactCondition::Always);
        changed = true;
    }
    changed
}

pub fn edit_fact_condition(
    ui: &mut egui::Ui,
    condition: &mut FactCondition,
    world: &World,
) -> bool {
    let mut changed = false;

    let variants = [
        "Equals",
        "GreaterThan",
        "LessThan",
        "GreaterOrEqual",
        "LessOrEqual",
        "Exists",
        "NotExists",
        "IsTrue",
        "IsFalse",
        "And",
        "Or",
        "Not",
        "Always",
    ];
    let current = match condition {
        FactCondition::Equals { .. } => 0,
        FactCondition::GreaterThan { .. } => 1,
        FactCondition::LessThan { .. } => 2,
        FactCondition::GreaterOrEqual { .. } => 3,
        FactCondition::LessOrEqual { .. } => 4,
        FactCondition::Exists(_) => 5,
        FactCondition::NotExists(_) => 6,
        FactCondition::IsTrue(_) => 7,
        FactCondition::IsFalse(_) => 8,
        FactCondition::And(_) => 9,
        FactCondition::Or(_) => 10,
        FactCondition::Not(_) => 11,
        FactCondition::Always => 12,
    };

    egui::ComboBox::from_id_salt("condition_type")
        .selected_text(variants[current])
        .show_ui(ui, |ui| {
            for (i, v) in variants.iter().enumerate() {
                if ui.selectable_label(current == i, *v).clicked() && current != i {
                    *condition = match i {
                        0 => FactCondition::Equals {
                            key: String::new(),
                            value: FactValueMatch::Bool(true),
                        },
                        1 => FactCondition::GreaterThan {
                            key: String::new(),
                            value: 0,
                        },
                        2 => FactCondition::LessThan {
                            key: String::new(),
                            value: 0,
                        },
                        3 => FactCondition::GreaterOrEqual {
                            key: String::new(),
                            value: 0,
                        },
                        4 => FactCondition::LessOrEqual {
                            key: String::new(),
                            value: 0,
                        },
                        5 => FactCondition::Exists(String::new()),
                        6 => FactCondition::NotExists(String::new()),
                        7 => FactCondition::IsTrue(String::new()),
                        8 => FactCondition::IsFalse(String::new()),
                        9 => FactCondition::And(vec![]),
                        10 => FactCondition::Or(vec![]),
                        11 => FactCondition::Not(Box::new(FactCondition::Always)),
                        _ => FactCondition::Always,
                    };
                    changed = true;
                }
            }
        });

    match condition {
        FactCondition::Equals { key, value } => {
            changed |= labeled_text(ui, &t(world, "prop-key"), key);
            changed |= edit_fact_value_match(ui, &t(world, "prop-value"), value);
        }
        FactCondition::GreaterThan { key, value }
        | FactCondition::LessThan { key, value }
        | FactCondition::GreaterOrEqual { key, value }
        | FactCondition::LessOrEqual { key, value } => {
            changed |= labeled_text(ui, &t(world, "prop-key"), key);
            ui.label(t(world, "fre-value"));
            changed |= ui.add(egui::DragValue::new(value)).changed();
        }
        FactCondition::Exists(key)
        | FactCondition::NotExists(key)
        | FactCondition::IsTrue(key)
        | FactCondition::IsFalse(key) => {
            changed |= labeled_text(ui, &t(world, "prop-key"), key);
        }
        FactCondition::And(subs) | FactCondition::Or(subs) => {
            let mut args = FluentArgs::new();
            args.set("count", subs.len() as i64);
            ui.label(t_args(world, "widget-subconditions", &args));
            changed |= render_sub_conditions(ui, subs, world);
        }
        FactCondition::Not(inner) => {
            ui.indent("not_inner", |ui| {
                changed |= edit_fact_condition(ui, inner, world);
            });
        }
        FactCondition::Always => {}
    }

    changed
}

fn render_modification_row(
    ui: &mut egui::Ui,
    i: usize,
    modif: &mut FactModificationDef,
    to_remove: &mut Option<usize>,
) -> bool {
    let mut changed = false;
    let variants = ["Set", "Increment", "Remove", "Toggle"];
    let current = match modif {
        FactModificationDef::Set { .. } => 0,
        FactModificationDef::Increment { .. } => 1,
        FactModificationDef::Remove(_) => 2,
        FactModificationDef::Toggle(_) => 3,
    };

    egui::ComboBox::from_id_salt(("mod_type", i))
        .selected_text(variants[current])
        .width(80.0)
        .show_ui(ui, |ui| {
            for (j, v) in variants.iter().enumerate() {
                if ui.selectable_label(current == j, *v).clicked() && current != j {
                    *modif = match j {
                        0 => FactModificationDef::Set {
                            key: String::new(),
                            value: FactValueMatch::Int(0),
                        },
                        1 => FactModificationDef::Increment {
                            key: String::new(),
                            amount: 1,
                        },
                        2 => FactModificationDef::Remove(String::new()),
                        _ => FactModificationDef::Toggle(String::new()),
                    };
                    changed = true;
                }
            }
        });

    match modif {
        FactModificationDef::Set { key, value } => {
            changed |= ui.text_edit_singleline(key).changed();
            changed |= edit_fact_value_match(ui, "=", value);
        }
        FactModificationDef::Increment { key, amount } => {
            changed |= ui.text_edit_singleline(key).changed();
            changed |= ui.add(egui::DragValue::new(amount).prefix("+")).changed();
        }
        FactModificationDef::Remove(key) | FactModificationDef::Toggle(key) => {
            changed |= ui.text_edit_singleline(key).changed();
        }
    }

    if ui.small_button("✕").clicked() {
        *to_remove = Some(i);
    }
    changed
}

pub fn edit_fact_modifications(
    ui: &mut egui::Ui,
    modifications: &mut Vec<FactModificationDef>,
    world: &World,
) -> bool {
    let mut changed = false;
    let mut args = FluentArgs::new();
    args.set("count", modifications.len() as i64);
    ui.label(t_args(world, "label-modification-count", &args));

    let mut to_remove = None;
    for (i, modif) in modifications.iter_mut().enumerate() {
        ui.push_id(i, |ui| {
            ui.horizontal(|ui| {
                changed |= render_modification_row(ui, i, modif, &mut to_remove);
            });
        });
    }

    if let Some(idx) = to_remove {
        modifications.remove(idx);
        changed = true;
    }
    if ui
        .small_button(t(world, "action-add-modification"))
        .clicked()
    {
        modifications.push(FactModificationDef::Set {
            key: String::new(),
            value: FactValueMatch::Int(0),
        });
        changed = true;
    }

    changed
}
