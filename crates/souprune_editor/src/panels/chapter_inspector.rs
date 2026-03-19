//! # Chapter Inspector Panel
//!
//! # 章节属性检查器面板
//!
//! ## Module Overview
//!
//! ## 模块概述
//!
//! Chapter inspector panel for the editor.
//! Displays editable properties of the selected chapter.
//!
//! 编辑器的章节属性检查器面板。
//! 显示选中章节的可编辑属性。

use bevy::prelude::*;
use bevy_workbench::i18n::FluentArgs;
use bevy_workbench::prelude::*;
use souprune::core::sequencer::chapter_schema::{Chapter, ElementModification, ElementSelector, LogLevel};

use super::sequence_timeline::EditorSequenceState;
use crate::data::ModifyChapterAction;
use crate::i18n::{t, t_args};
use crate::widgets;
use crate::widgets::property_editors::*;

/// 章节属性检查器面板。
pub struct ChapterInspectorPanel {
    cached_title: String,
}

impl ChapterInspectorPanel {
    pub fn new() -> Self {
        Self {
            cached_title: "Chapter Inspector".to_string(),
        }
    }
}

impl WorkbenchPanel for ChapterInspectorPanel {
    fn id(&self) -> &str {
        "chapter_inspector"
    }

    fn title(&self) -> String {
        self.cached_title.clone()
    }

    fn closable(&self) -> bool {
        true
    }

    fn default_visible(&self) -> bool {
        true
    }

    fn needs_world(&self) -> bool {
        true
    }

    fn ui_world(&mut self, ui: &mut egui::Ui, world: &mut World) {
        self.cached_title = t(world, "panel-chapter-inspector");

        if !world.contains_resource::<EditorSequenceState>() {
            ui.label(t(world, "label-not-initialized"));
            return;
        }

        let state = world.resource::<EditorSequenceState>();
        let Some(seq) = &state.current else {
            ui.label(t(world, "label-no-sequence-open"));
            return;
        };
        let Some(idx) = state.selected_chapter else {
            ui.centered_and_justified(|ui| {
                ui.label(t(world, "label-select-chapter"));
            });
            return;
        };
        let Some(chapter) = seq.chapters.get(idx) else {
            ui.label(t(world, "label-invalid-chapter"));
            return;
        };
        let chapter = chapter.clone();
        let icon = crate::widgets::chapter_icon(&chapter);
        let type_name = chapter_type_label(&chapter);

        ui.heading(format!("{icon} {type_name}"));
        ui.separator();

        // 根据章节类型渲染属性编辑器
        let mut edited_chapter = chapter.clone();
        let changed = render_chapter_properties(ui, &mut edited_chapter, world);

        if changed {
            apply_chapter_edit(world, idx, &edited_chapter);
            world.resource_mut::<UndoStack>().push(ModifyChapterAction {
                index: idx,
                old_chapter: chapter,
                new_chapter: edited_chapter,
            });
        }
    }

    fn ui(&mut self, ui: &mut egui::Ui) {
        ui.label("Requires World access");
    }
}

fn apply_chapter_edit(world: &mut World, idx: usize, edited: &Chapter) {
    let mut state = world.resource_mut::<EditorSequenceState>();
    if let Some(seq) = &mut state.current
        && let Some(ch) = seq.chapters.get_mut(idx)
    {
        *ch = edited.clone();
        seq.dirty = true;
    }
    state.save_timer = Some(0.5);
}

/// 渲染章节属性编辑器。返回 true 如果有修改。
fn render_chapter_properties(ui: &mut egui::Ui, chapter: &mut Chapter, world: &World) -> bool {
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
            if ui
                .checkbox(local, t(world, "inspector-use-local-facts"))
                .changed()
            {
                changed = true;
            }
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
            if ui
                .checkbox(wait_for_completion, t(world, "inspector-wait-completion"))
                .changed()
            {
                changed = true;
            }
        }

        Chapter::TweenViewElement {
            selector,
            duration,
            wait_for_completion,
            ..
        } => {
            changed |= edit_element_selector(ui, selector, world);
            changed |= labeled_drag(
                ui,
                &t(world, "prop-duration-sec"),
                duration,
                0.0..=f32::MAX,
                0.1,
            );
            if ui
                .checkbox(wait_for_completion, t(world, "inspector-wait-completion"))
                .changed()
            {
                changed = true;
            }
        }

        Chapter::Sequence(children) | Chapter::Parallel(children) => {
            let mut args = FluentArgs::new();
            args.set("count", children.len() as i64);
            ui.label(t_args(world, "label-sub-chapters", &args));
        }

        Chapter::SetPlayer(action) => {
            changed |= edit_player_action(ui, action, world);
        }

        Chapter::SetUI(action) => {
            changed |= edit_ui_action(ui, action, world);
        }

        Chapter::SetCamera(action) => {
            changed |= edit_camera_action(ui, action, world);
        }

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
            {
                let mut args = FluentArgs::new();
                args.set("count", cases.len() as i64);
                ui.label(t_args(world, "label-branch-count", &args));
            }
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
            setup_camera_bounds,
        } => {
            changed |=
                widgets::path_picker::edit_file_path(ui, &t(world, "prop-map-path"), path, world);
            if ui
                .checkbox(generate_collision, t(world, "inspector-generate-collision"))
                .changed()
            {
                changed = true;
            }
            if ui
                .checkbox(process_objects, t(world, "inspector-process-objects"))
                .changed()
            {
                changed = true;
            }
            if ui
                .checkbox(
                    setup_camera_bounds,
                    t(world, "inspector-setup-camera-bounds"),
                )
                .changed()
            {
                changed = true;
            }
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
        } => {
            changed |= labeled_text(ui, &t(world, "prop-source-box"), source);
            ui.separator();
            changed |= labeled_text(ui, &t(world, "prop-result-left"), &mut result.0);
            changed |= labeled_text(ui, &t(world, "prop-result-right"), &mut result.1);
            ui.separator();
            let axis_variants = ["Vertical", "Horizontal"];
            let current_axis = match axis {
                souprune::app_state::battle::collision::SplitAxis::Vertical => 0,
                souprune::app_state::battle::collision::SplitAxis::Horizontal => 1,
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
                    0 => souprune::app_state::battle::collision::SplitAxis::Vertical,
                    _ => souprune::app_state::battle::collision::SplitAxis::Horizontal,
                };
                changed = true;
            }
            changed |= labeled_drag(
                ui,
                &t(world, "prop-position"),
                position,
                -500.0..=500.0,
                1.0,
            );
            changed |= labeled_drag(ui, &t(world, "prop-gap"), gap, 0.0..=200.0, 1.0);
            let gap_policy_variants = ["Expands", "Includes"];
            let current_gap_policy = match gap_policy {
                souprune::app_state::battle::collision::GapPolicy::Expands => 0,
                souprune::app_state::battle::collision::GapPolicy::Includes => 1,
            };
            let mut selected_gap_policy = current_gap_policy;
            egui::ComboBox::from_label(t(world, "prop-gap-policy"))
                .selected_text(gap_policy_variants[selected_gap_policy])
                .show_ui(ui, |ui| {
                    for (i, label) in gap_policy_variants.iter().enumerate() {
                        ui.selectable_value(&mut selected_gap_policy, i, *label);
                    }
                });
            if selected_gap_policy != current_gap_policy {
                *gap_policy = match selected_gap_policy {
                    0 => souprune::app_state::battle::collision::GapPolicy::Expands,
                    _ => souprune::app_state::battle::collision::GapPolicy::Includes,
                };
                changed = true;
            }
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
            duration,
        } => {
            changed |= labeled_text(ui, &t(world, "prop-source-a"), &mut sources.0);
            changed |= labeled_text(ui, &t(world, "prop-source-b"), &mut sources.1);
            ui.separator();
            changed |= labeled_text(ui, &t(world, "prop-result-box"), result);
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
            let level_variants = ["Info", "Debug", "Warn", "Error"];
            let current_level = match level {
                LogLevel::Info => 0,
                LogLevel::Debug => 1,
                LogLevel::Warn => 2,
                LogLevel::Error => 3,
            };
            let mut selected = current_level;
            egui::ComboBox::from_label(t(world, "prop-log-level"))
                .selected_text(level_variants[selected])
                .show_ui(ui, |ui| {
                    for (i, label) in level_variants.iter().enumerate() {
                        ui.selectable_value(&mut selected, i, *label);
                    }
                });
            if selected != current_level {
                *level = match selected {
                    0 => LogLevel::Info,
                    1 => LogLevel::Debug,
                    2 => LogLevel::Warn,
                    _ => LogLevel::Error,
                };
                changed = true;
            }
        }
    }

    changed
}

/// 渲染 ElementSelector 编辑器。
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

/// 渲染 ElementModification 编辑器。
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
