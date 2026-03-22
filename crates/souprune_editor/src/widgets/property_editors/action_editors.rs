//! # action_editors.rs
//!
//! # action_editors.rs 文件
//!
//! ## Module Overview
//!
//! ## 模块概述
//!
//! This file provides the property-editor widgets for action-like sequence data. It contains the
//! UI needed to edit player, camera, and UI actions, plus the helper that reports the display name
//! for each chapter variant in list-oriented editor views.
//!
//! 这个文件提供了动作类序列数据的属性编辑控件。它负责编辑玩家、相机和 UI 动作，同时提供
//! 一个辅助函数，用来给列表式编辑界面中的章节变体显示可读名称。

use bevy::prelude::*;
use souprune_schema::sequence::{CameraAction, Chapter, PlayerAction, UIAction};

use crate::i18n::t;

use super::{edit_string_list, labeled_drag, labeled_text};

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
                        2 => PlayerAction::Teleport((0.0, 0.0)),
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
                *position = if has_pos { Some((0.0, 0.0)) } else { None };
                changed = true;
            }
            if let Some(pos) = position {
                ui.horizontal(|ui| {
                    changed |= ui
                        .add(egui::DragValue::new(&mut pos.0).prefix("X: ").speed(1.0))
                        .changed();
                    changed |= ui
                        .add(egui::DragValue::new(&mut pos.1).prefix("Y: ").speed(1.0))
                        .changed();
                });
            }
        }
        PlayerAction::Teleport(pos) => {
            ui.horizontal(|ui| {
                ui.label(t(world, "inspector-position"));
                changed |= ui
                    .add(egui::DragValue::new(&mut pos.0).prefix("X: ").speed(1.0))
                    .changed();
                changed |= ui
                    .add(egui::DragValue::new(&mut pos.1).prefix("Y: ").speed(1.0))
                    .changed();
            });
        }
        PlayerAction::SetActive(active) => {
            changed |= ui.checkbox(active, t(world, "inspector-active")).changed();
        }
        PlayerAction::Despawn => {}
    }

    changed
}

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
                        0 => CameraAction::SetPosition((0.0, 0.0)),
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
                changed |= ui
                    .add(egui::DragValue::new(&mut pos.0).prefix("X: ").speed(1.0))
                    .changed();
                changed |= ui
                    .add(egui::DragValue::new(&mut pos.1).prefix("Y: ").speed(1.0))
                    .changed();
            });
        }
        CameraAction::SetZoom(zoom) => {
            changed |= ui
                .add(
                    egui::DragValue::new(zoom)
                        .prefix("Zoom: ")
                        .range(0.1..=10.0)
                        .speed(0.05),
                )
                .changed();
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
            changed |= ui
                .checkbox(follow, t(world, "inspector-follow-player"))
                .changed();
        }
    }

    changed
}

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
        Chapter::SplitBattleBox { .. } => "SplitBattleBox",
        Chapter::MergeBattleBoxes { .. } => "MergeBattleBoxes",
        Chapter::Log { .. } => "Log",
    }
}
