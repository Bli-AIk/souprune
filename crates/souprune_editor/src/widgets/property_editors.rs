//! # Property Editors
//!
//! # 属性编辑器组件
//!
//! ## Module Overview
//!
//! ## 模块概述
//!
//! Property editor widgets for the editor.
//! Generic property editors: Option<T>, HashMap, lists, enum dropdowns, etc.
//!
//! 编辑器的属性编辑器组件。
//! 通用属性编辑器：Option<T>、HashMap、列表、枚举下拉等。

use bevy::prelude::*;
use bevy_workbench::i18n::FluentArgs;
use souprune::core::sequencer::chapter_schema::{
    CameraAction, Chapter, FactCondition, FactModificationDef, FactValueMatch, PlayerAction,
    UIAction,
};

use crate::i18n::{t, t_args};

// ─── 通用编辑器组件 ───────────────────────────────────────────

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

// ─── Option<T> 编辑器 ────────────────────────────────────────

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

// ─── HashMap 编辑器 ──────────────────────────────────────────

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

// ─── 列表编辑器 ──────────────────────────────────────────────

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

// ─── FactValueMatch 编辑器 ───────────────────────────────────

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
                if ui.add(egui::DragValue::new(v)).changed() {
                    changed = true;
                }
            }
            FactValueMatch::Float(v) => {
                if ui.add(egui::DragValue::new(v).speed(0.1)).changed() {
                    changed = true;
                }
            }
            FactValueMatch::Bool(v) => {
                if ui.checkbox(v, "").changed() {
                    changed = true;
                }
            }
            FactValueMatch::String(v) | FactValueMatch::Expr(v) => {
                if ui.text_edit_singleline(v).changed() {
                    changed = true;
                }
            }
        }
    });

    changed
}

// ─── FactCondition 编辑器 ────────────────────────────────────

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
            if ui.add(egui::DragValue::new(value)).changed() {
                changed = true;
            }
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

// ─── PlayerAction 编辑器 ─────────────────────────────────────

pub fn edit_player_action(ui: &mut egui::Ui, action: &mut PlayerAction, world: &World) -> bool {
    let mut changed = false;

    let variants = ["SetMode", "Spawn", "Teleport", "SetActive", "Despawn"];
    let current = match action {
        PlayerAction::SetMode(_) => 0,
        PlayerAction::Spawn { .. } => 1,
        PlayerAction::Teleport(_) => 2,
        PlayerAction::SetActive(_) => 3,
        PlayerAction::Despawn => 4,
    };

    egui::ComboBox::from_id_salt("player_action")
        .selected_text(variants[current])
        .show_ui(ui, |ui| {
            for (i, v) in variants.iter().enumerate() {
                if ui.selectable_label(current == i, *v).clicked() && current != i {
                    *action = match i {
                        0 => PlayerAction::SetMode(vec![]),
                        1 => PlayerAction::Spawn {
                            config_path: String::new(),
                            position: None,
                        },
                        2 => PlayerAction::Teleport(Vec2::ZERO),
                        3 => PlayerAction::SetActive(true),
                        _ => PlayerAction::Despawn,
                    };
                    changed = true;
                }
            }
        });

    match action {
        PlayerAction::SetMode(modes) => {
            changed |= edit_string_list(ui, &t(world, "prop-modes"), modes, world);
        }
        PlayerAction::Spawn {
            config_path,
            position,
        } => {
            changed |= labeled_text(ui, &t(world, "prop-config-path"), config_path);
            let mut has_pos = position.is_some();
            if ui
                .checkbox(&mut has_pos, t(world, "inspector-specify-position"))
                .changed()
            {
                *position = if has_pos { Some(Vec2::ZERO) } else { None };
                changed = true;
            }
            if let Some(pos) = position {
                ui.horizontal(|ui| {
                    changed |= ui
                        .add(egui::DragValue::new(&mut pos.x).prefix("X: ").speed(1.0))
                        .changed();
                    changed |= ui
                        .add(egui::DragValue::new(&mut pos.y).prefix("Y: ").speed(1.0))
                        .changed();
                });
            }
        }
        PlayerAction::Teleport(pos) => {
            ui.horizontal(|ui| {
                ui.label(t(world, "inspector-position"));
                if ui
                    .add(egui::DragValue::new(&mut pos.x).prefix("X: ").speed(1.0))
                    .changed()
                {
                    changed = true;
                }
                if ui
                    .add(egui::DragValue::new(&mut pos.y).prefix("Y: ").speed(1.0))
                    .changed()
                {
                    changed = true;
                }
            });
        }
        PlayerAction::SetActive(active) => {
            if ui.checkbox(active, t(world, "inspector-active")).changed() {
                changed = true;
            }
        }
        PlayerAction::Despawn => {}
    }

    changed
}

// ─── CameraAction 编辑器 ─────────────────────────────────────

pub fn edit_camera_action(ui: &mut egui::Ui, action: &mut CameraAction, world: &World) -> bool {
    let mut changed = false;

    let variants = ["SetPosition", "SetZoom", "Shake", "FollowPlayer"];
    let current = match action {
        CameraAction::SetPosition(_) => 0,
        CameraAction::SetZoom(_) => 1,
        CameraAction::Shake { .. } => 2,
        CameraAction::FollowPlayer(_) => 3,
    };

    egui::ComboBox::from_id_salt("camera_action")
        .selected_text(variants[current])
        .show_ui(ui, |ui| {
            for (i, v) in variants.iter().enumerate() {
                if ui.selectable_label(current == i, *v).clicked() && current != i {
                    *action = match i {
                        0 => CameraAction::SetPosition(Vec2::ZERO),
                        1 => CameraAction::SetZoom(1.0),
                        2 => CameraAction::Shake {
                            duration: 0.5,
                            intensity: 5.0,
                        },
                        _ => CameraAction::FollowPlayer(true),
                    };
                    changed = true;
                }
            }
        });

    match action {
        CameraAction::SetPosition(pos) => {
            ui.horizontal(|ui| {
                if ui
                    .add(egui::DragValue::new(&mut pos.x).prefix("X: ").speed(1.0))
                    .changed()
                {
                    changed = true;
                }
                if ui
                    .add(egui::DragValue::new(&mut pos.y).prefix("Y: ").speed(1.0))
                    .changed()
                {
                    changed = true;
                }
            });
        }
        CameraAction::SetZoom(zoom) => {
            if ui
                .add(
                    egui::DragValue::new(zoom)
                        .prefix("Zoom: ")
                        .range(0.1..=10.0)
                        .speed(0.05),
                )
                .changed()
            {
                changed = true;
            }
        }
        CameraAction::Shake {
            duration,
            intensity,
        } => {
            changed |= labeled_drag(
                ui,
                &t(world, "prop-duration"),
                duration,
                0.0..=f32::MAX,
                0.1,
            );
            changed |= labeled_drag(ui, &t(world, "prop-intensity"), intensity, 0.0..=100.0, 0.5);
        }
        CameraAction::FollowPlayer(follow) => {
            if ui
                .checkbox(follow, t(world, "inspector-follow-player"))
                .changed()
            {
                changed = true;
            }
        }
    }

    changed
}

// ─── UIAction 编辑器 ─────────────────────────────────────────

pub fn edit_ui_action(ui: &mut egui::Ui, action: &mut UIAction, world: &World) -> bool {
    let mut changed = false;

    let variants = [
        "LoadLayout",
        "Show",
        "Hide",
        "SetText",
        "SetVariable",
        "PlayAnimation",
    ];
    let current = match action {
        UIAction::LoadLayout(_) => 0,
        UIAction::Show(_) => 1,
        UIAction::Hide(_) => 2,
        UIAction::SetText { .. } => 3,
        UIAction::SetVariable { .. } => 4,
        UIAction::PlayAnimation { .. } => 5,
    };

    egui::ComboBox::from_id_salt("ui_action")
        .selected_text(variants[current])
        .show_ui(ui, |ui| {
            for (i, v) in variants.iter().enumerate() {
                if ui.selectable_label(current == i, *v).clicked() && current != i {
                    *action = match i {
                        0 => UIAction::LoadLayout(String::new()),
                        1 => UIAction::Show(String::new()),
                        2 => UIAction::Hide(String::new()),
                        3 => UIAction::SetText {
                            id: String::new(),
                            content: String::new(),
                        },
                        4 => UIAction::SetVariable {
                            name: String::new(),
                            value: String::new(),
                        },
                        _ => UIAction::PlayAnimation {
                            id: String::new(),
                            clip: String::new(),
                        },
                    };
                    changed = true;
                }
            }
        });

    match action {
        UIAction::LoadLayout(path) | UIAction::Show(path) | UIAction::Hide(path) => {
            changed |= labeled_text(ui, &t(world, "prop-path-id"), path);
        }
        UIAction::SetText { id, content } => {
            changed |= labeled_text(ui, &t(world, "prop-element-id"), id);
            changed |= labeled_text(ui, &t(world, "prop-content"), content);
        }
        UIAction::SetVariable { name, value } => {
            changed |= labeled_text(ui, &t(world, "prop-variable-name"), name);
            changed |= labeled_text(ui, &t(world, "prop-value"), value);
        }
        UIAction::PlayAnimation { id, clip } => {
            changed |= labeled_text(ui, &t(world, "prop-element-id"), id);
            changed |= labeled_text(ui, &t(world, "prop-anim-clip"), clip);
        }
    }

    changed
}

// ─── FactModificationDef 编辑器 ──────────────────────────────

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

pub fn chapter_type_label(chapter: &Chapter) -> &'static str {
    match chapter {
        Chapter::SpawnView { .. } => "SpawnView",
        Chapter::AwaitFact { .. } => "AwaitFact",
        Chapter::SetViewFact { .. } => "SetViewFact",
        Chapter::DanmakuPerformance { .. } => "DanmakuPerformance",
        Chapter::AlightMotionPerformance { .. } => "AlightMotionPerformance",
        Chapter::TweenViewElement { .. } => "TweenViewElement",
        Chapter::Wait(_) => "Wait",
        Chapter::Sequence(_) => "Sequence",
        Chapter::Parallel(_) => "Parallel",
        Chapter::SetPlayer(_) => "SetPlayer",
        Chapter::SetUI(_) => "SetUI",
        Chapter::ModifyViewElement { .. } => "ModifyViewElement",
        Chapter::SetCamera(_) => "SetCamera",
        Chapter::Conditional { .. } => "Conditional",
        Chapter::FactSwitch { .. } => "FactSwitch",
        Chapter::EmitFactEvent { .. } => "EmitFactEvent",
        Chapter::ModifyFact { .. } => "ModifyFact",
        Chapter::LoadFre { .. } => "LoadFre",
        Chapter::RunSequence { .. } => "RunSequence",
        Chapter::LoadMap { .. } => "LoadMap",
        Chapter::SetBgm { .. } => "SetBgm",
        Chapter::Custom { .. } => "Custom",
        Chapter::LoadEnemies { .. } => "LoadEnemies",
    }
}
