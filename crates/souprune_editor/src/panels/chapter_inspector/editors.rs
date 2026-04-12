//! # editors.rs
//!
//! # editors.rs 文件
//!
//! ## Module Overview
//!
//! ## 模块概述
//!
//! Renders the type-specific property editors used by the chapter inspector. It maps
//! each sequence `Chapter` variant to the controls that can edit it, so the inspector panel can
//! stay focused on selection and layout instead of carrying every field editor inline.
//!
//! 负责渲染章节检查器里按类型分派的属性编辑器。它把每个序列 `Chapter` 变体映射到
//! 对应的编辑控件，让检查器面板本身只负责选择和布局，而不用把所有字段编辑逻辑都堆在一起。

use bevy::prelude::*;
use bevy_workbench::i18n::FluentArgs;
use souprune_schema::sequence::{
    Chapter, ElementModification, ElementSelector, GapPolicy, LogLevel, SplitAxis,
};

use crate::i18n::{t, t_args};
use crate::widgets;
use crate::widgets::property_editors::*;

/// 渲染章节属性编辑器。返回 true 如果有修改。
pub(super) fn render_chapter_properties(
    ui: &mut egui::Ui,
    chapter: &mut Chapter,
    world: &World,
) -> bool {
    let mut changed = false;

    match chapter {
        Chapter::Wait(duration) => {
            changed |= labeled_drag(
                ui,
                &t(world, "prop-duration-sec"),
                duration,
                0.0..=f32::MAX,
                0.1,
            );
        }
        Chapter::SpawnView {
            view_layout,
            bindings,
        } => {
            changed |= widgets::path_picker::edit_file_path(
                ui,
                &t(world, "prop-view-layout-file"),
                view_layout,
                world,
            );
            ui.separator();
            changed |= edit_hashmap_string(ui, &t(world, "prop-bindings"), bindings);
        }
        Chapter::AwaitFact { condition, local } => {
            changed |= labeled_text(ui, &t(world, "prop-condition"), condition);
            changed |= ui
                .checkbox(local, t(world, "inspector-use-local-facts"))
                .changed();
        }
        Chapter::SetViewFact { key, value } => {
            changed |= labeled_text(ui, &t(world, "prop-fact-key"), key);
            changed |= edit_fact_value_match(ui, &t(world, "prop-value"), value);
        }
        Chapter::DanmakuPerformance {
            performance,
            translation,
        } => {
            changed |= widgets::path_picker::edit_file_path(
                ui,
                &t(world, "prop-perf-file"),
                performance,
                world,
            );
            changed |= edit_option_vec2(ui, &t(world, "prop-position"), translation);
        }
        Chapter::AlightMotionPerformance {
            amproj_path,
            alight_motion_config,
            wait_for_completion,
        } => {
            changed |= widgets::path_picker::edit_file_path(
                ui,
                &t(world, "prop-amproj-file"),
                amproj_path,
                world,
            );
            changed |= edit_option_string(ui, &t(world, "prop-am-config"), alight_motion_config);
            changed |= ui
                .checkbox(wait_for_completion, t(world, "inspector-wait-completion"))
                .changed();
        }
        Chapter::SetViewElement {
            selector,
            duration,
            wait_for_completion,
            ..
        } => {
            changed |= edit_element_selector(ui, selector, world);
            let mut animated = duration.is_some();
            if ui
                .checkbox(&mut animated, t(world, "inspector-animated"))
                .changed()
            {
                *duration = if animated { Some(0.5) } else { None };
                changed = true;
            }
            if let Some(dur) = duration {
                changed |=
                    labeled_drag(ui, &t(world, "prop-duration-sec"), dur, 0.0..=f32::MAX, 0.1);
            }
            changed |= ui
                .checkbox(wait_for_completion, t(world, "inspector-wait-completion"))
                .changed();
        }
        Chapter::Sequence(children) | Chapter::Parallel(children) => {
            let mut args = FluentArgs::new();
            args.set("count", children.len() as i64);
            ui.label(t_args(world, "label-sub-chapters", &args));
        }
        Chapter::SetPlayer(action) => changed |= edit_player_action(ui, action, world),
        Chapter::SetUI(action) => changed |= edit_ui_action(ui, action, world),
        Chapter::SetCamera(action) => changed |= edit_camera_action(ui, action, world),
        Chapter::ModifyViewElement {
            selector,
            modification,
        } => {
            changed |= edit_element_selector(ui, selector, world);
            changed |= edit_element_modification(ui, modification, world);
        }
        Chapter::Conditional { condition, .. } => {
            changed |= edit_fact_condition(ui, condition, world);
        }
        Chapter::FactSwitch {
            fact_key,
            cases,
            default,
        } => {
            changed |= labeled_text(ui, &t(world, "prop-fact-key"), fact_key);
            let mut args = FluentArgs::new();
            args.set("count", cases.len() as i64);
            ui.label(t_args(world, "label-branch-count", &args));
            for (i, (value, _)) in cases.iter_mut().enumerate() {
                ui.push_id(i, |ui| {
                    let mut args = FluentArgs::new();
                    args.set("index", (i as i64) + 1);
                    changed |=
                        edit_fact_value_match(ui, &t_args(world, "prop-branch", &args), value);
                });
            }
            let has_default = default.is_some();
            let mut enable_default = has_default;
            if ui
                .checkbox(&mut enable_default, t(world, "inspector-default-branch"))
                .changed()
            {
                if enable_default && !has_default {
                    *default = Some(Box::new(Chapter::Wait(0.0)));
                } else if !enable_default {
                    *default = None;
                }
                changed = true;
            }
        }
        Chapter::EmitFactEvent { event_id, data } => {
            changed |= labeled_text(ui, &t(world, "prop-event-id"), event_id);
            ui.separator();
            changed |= edit_hashmap_string_flat(ui, &t(world, "prop-data"), data, world);
        }
        Chapter::ModifyFact { modifications } => {
            changed |= edit_fact_modifications(ui, modifications, world);
        }
        Chapter::LoadFre { files, .. } => {
            changed |= edit_string_list(ui, &t(world, "prop-fre-files"), files, world);
        }
        Chapter::RunSequence {
            path,
            path_fact,
            params,
        } => {
            changed |= edit_option_string(ui, &t(world, "prop-seq-path"), path);
            changed |= edit_option_string(ui, &t(world, "prop-dynamic-path-fact"), path_fact);
            if !params.is_empty() {
                ui.separator();
                let mut args = FluentArgs::new();
                args.set("count", params.len() as i64);
                ui.label(t_args(world, "label-param-count", &args));
            }
        }
        Chapter::LoadMap {
            path,
            generate_collision,
            process_objects,
        } => {
            changed |=
                widgets::path_picker::edit_file_path(ui, &t(world, "prop-map-path"), path, world);
            changed |= ui
                .checkbox(generate_collision, t(world, "inspector-generate-collision"))
                .changed();
            changed |= ui
                .checkbox(process_objects, t(world, "inspector-process-objects"))
                .changed();
        }
        Chapter::SetBgm { path, fade_in } => {
            changed |= edit_option_string(ui, &t(world, "prop-bgm-path"), path);
            changed |= edit_option_f32(ui, &t(world, "prop-fade-in-sec"), fade_in);
        }
        Chapter::Custom {
            action_type,
            params,
        } => {
            changed |= labeled_text(ui, &t(world, "prop-action-type"), action_type);
            ui.separator();
            changed |= edit_hashmap_string_flat(ui, &t(world, "prop-params"), params, world);
        }
        Chapter::LoadEnemies { enemies } => {
            changed |= edit_string_list(ui, &t(world, "prop-files"), enemies, world);
        }
        Chapter::SplitBattleBox {
            source,
            result,
            axis,
            position,
            gap,
            gap_policy,
            duration,
            ..
        } => {
            changed |= labeled_text(ui, &t(world, "prop-source-box"), source);
            ui.separator();
            changed |= labeled_text(ui, &t(world, "prop-result-left"), &mut result.0);
            changed |= labeled_text(ui, &t(world, "prop-result-right"), &mut result.1);
            ui.separator();
            changed |= edit_split_axis(ui, axis, world);
            changed |= labeled_drag(
                ui,
                &t(world, "prop-position"),
                position,
                -500.0..=500.0,
                1.0,
            );
            changed |= labeled_drag(ui, &t(world, "prop-gap"), gap, 0.0..=200.0, 1.0);
            changed |= edit_gap_policy(ui, gap_policy, world);
            changed |= labeled_drag(
                ui,
                &t(world, "prop-duration-sec"),
                duration,
                0.0..=10.0,
                0.05,
            );
        }
        Chapter::MergeBattleBoxes {
            sources,
            result,
            gap_policy,
            duration,
            ..
        } => {
            changed |= labeled_text(ui, &t(world, "prop-source-a"), &mut sources.0);
            changed |= labeled_text(ui, &t(world, "prop-source-b"), &mut sources.1);
            ui.separator();
            changed |= labeled_text(ui, &t(world, "prop-result-box"), result);
            changed |= edit_gap_policy(ui, gap_policy, world);
            changed |= labeled_drag(
                ui,
                &t(world, "prop-duration-sec"),
                duration,
                0.0..=10.0,
                0.05,
            );
        }
        Chapter::Log { text, level } => {
            changed |= labeled_text(ui, &t(world, "prop-log-text"), text);
            changed |= edit_log_level(ui, level, world);
        }
    }

    changed
}

fn edit_split_axis(ui: &mut egui::Ui, axis: &mut SplitAxis, world: &World) -> bool {
    let axis_variants = ["Vertical", "Horizontal"];
    let current_axis = match axis {
        SplitAxis::Vertical => 0,
        SplitAxis::Horizontal => 1,
    };
    let mut selected = current_axis;
    egui::ComboBox::from_label(t(world, "prop-axis"))
        .selected_text(axis_variants[selected])
        .show_ui(ui, |ui| {
            for (i, label) in axis_variants.iter().enumerate() {
                ui.selectable_value(&mut selected, i, *label);
            }
        });
    if selected != current_axis {
        *axis = match selected {
            0 => SplitAxis::Vertical,
            _ => SplitAxis::Horizontal,
        };
        true
    } else {
        false
    }
}

fn edit_gap_policy(ui: &mut egui::Ui, gap_policy: &mut GapPolicy, world: &World) -> bool {
    let variants = ["Expands", "Includes"];
    let current = match gap_policy {
        GapPolicy::Expands => 0,
        GapPolicy::Includes => 1,
    };
    let mut selected = current;
    egui::ComboBox::from_label(t(world, "prop-gap-policy"))
        .selected_text(variants[selected])
        .show_ui(ui, |ui| {
            for (i, label) in variants.iter().enumerate() {
                ui.selectable_value(&mut selected, i, *label);
            }
        });
    if selected != current {
        *gap_policy = match selected {
            0 => GapPolicy::Expands,
            _ => GapPolicy::Includes,
        };
        true
    } else {
        false
    }
}

fn edit_log_level(ui: &mut egui::Ui, level: &mut LogLevel, world: &World) -> bool {
    let variants = ["Info", "Debug", "Warn", "Error"];
    let current = match level {
        LogLevel::Info => 0,
        LogLevel::Debug => 1,
        LogLevel::Warn => 2,
        LogLevel::Error => 3,
    };
    let mut selected = current;
    egui::ComboBox::from_label(t(world, "prop-log-level"))
        .selected_text(variants[selected])
        .show_ui(ui, |ui| {
            for (i, label) in variants.iter().enumerate() {
                ui.selectable_value(&mut selected, i, *label);
            }
        });
    if selected != current {
        *level = match selected {
            0 => LogLevel::Info,
            1 => LogLevel::Debug,
            2 => LogLevel::Warn,
            _ => LogLevel::Error,
        };
        true
    } else {
        false
    }
}

fn edit_element_selector(ui: &mut egui::Ui, selector: &mut ElementSelector, world: &World) -> bool {
    let mut changed = false;
    let variants = ["FullName", "LocalName", "Tag"];
    let current = match selector {
        ElementSelector::FullName(_) => 0,
        ElementSelector::LocalName(_) => 1,
        ElementSelector::Tag(_) => 2,
    };
    ui.horizontal(|ui| {
        ui.label(t(world, "inspector-selector"));
        for (i, name) in variants.iter().enumerate() {
            if ui.selectable_label(current == i, *name).clicked() && current != i {
                *selector = match i {
                    0 => ElementSelector::FullName(String::new()),
                    1 => ElementSelector::LocalName(String::new()),
                    _ => ElementSelector::Tag(String::new()),
                };
                changed = true;
            }
        }
    });
    let name = match selector {
        ElementSelector::FullName(n) | ElementSelector::LocalName(n) | ElementSelector::Tag(n) => n,
    };
    changed |= labeled_text(ui, &t(world, "prop-name"), name);
    ui.separator();
    changed
}

fn edit_element_modification(
    ui: &mut egui::Ui,
    modification: &mut ElementModification,
    world: &World,
) -> bool {
    let mut changed = false;
    let label = match modification {
        ElementModification::SetTexture(_) => "SetTexture",
        ElementModification::SetPosition(..) => "SetPosition",
        ElementModification::SetScale(..) => "SetScale",
        ElementModification::SetColor(..) => "SetColor",
        ElementModification::SetVisibility(_) => "SetVisibility",
        ElementModification::SetBoxSize(..) => "SetBoxSize",
        ElementModification::Undo => "Undo",
        ElementModification::Redo => "Redo",
        ElementModification::Reset => "Reset",
    };
    let mut args = FluentArgs::new();
    args.set("label", label.to_string());
    ui.label(t_args(world, "inspector-modify-type", &args));
    match modification {
        ElementModification::SetTexture(path) => {
            changed |= widgets::path_picker::edit_file_path(
                ui,
                &t(world, "prop-texture-path"),
                path,
                world,
            );
        }
        ElementModification::SetVisibility(val) => {
            changed |= widgets::val_editor::edit_val_bool(ui, &t(world, "prop-visibility"), val);
        }
        ElementModification::SetBoxSize(w, h) => {
            changed |= widgets::val_editor::edit_val_f32(ui, &t(world, "prop-width"), w);
            changed |= widgets::val_editor::edit_val_f32(ui, &t(world, "prop-height"), h);
        }
        ElementModification::SetPosition(x, y, z) => {
            changed |= widgets::val_editor::edit_val_f32(ui, "X", x);
            changed |= widgets::val_editor::edit_val_f32(ui, "Y", y);
            changed |= widgets::val_editor::edit_val_f32(ui, "Z", z);
        }
        ElementModification::SetScale(x, y, z) => {
            changed |= widgets::val_editor::edit_val_f32(ui, &t(world, "prop-scale-x"), x);
            changed |= widgets::val_editor::edit_val_f32(ui, &t(world, "prop-scale-y"), y);
            changed |= widgets::val_editor::edit_val_f32(ui, &t(world, "prop-scale-z"), z);
        }
        ElementModification::SetColor(r, g, b, a) => {
            changed |= widgets::val_editor::edit_val_f32(ui, "R", r);
            changed |= widgets::val_editor::edit_val_f32(ui, "G", g);
            changed |= widgets::val_editor::edit_val_f32(ui, "B", b);
            changed |= widgets::val_editor::edit_val_f32(ui, "A", a);
        }
        ElementModification::Undo | ElementModification::Redo | ElementModification::Reset => {}
    }
    changed
}
