//! # 章节属性检查器面板
//!
//! 显示选中章节的可编辑属性。

use bevy::prelude::*;
use bevy_workbench::prelude::*;
use souprune::core::sequencer::chapter_schema::Chapter;

use super::sequence_timeline::EditorSequenceState;
use crate::widgets::property_editors::*;

/// 章节属性检查器面板。
pub struct ChapterInspectorPanel;

impl ChapterInspectorPanel {
    pub fn new() -> Self {
        Self
    }
}

impl WorkbenchPanel for ChapterInspectorPanel {
    fn id(&self) -> &str {
        "chapter_inspector"
    }

    fn title(&self) -> String {
        "Chapter Inspector".to_string()
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
        if !world.contains_resource::<EditorSequenceState>() {
            ui.label("未初始化");
            return;
        }

        let state = world.resource::<EditorSequenceState>();
        let Some(seq) = &state.current else {
            ui.label("未打开序列");
            return;
        };
        let Some(idx) = state.selected_chapter else {
            ui.centered_and_justified(|ui| {
                ui.label("选择一个章节以查看属性");
            });
            return;
        };
        let Some(chapter) = seq.chapters.get(idx) else {
            ui.label("无效的章节索引");
            return;
        };
        let chapter = chapter.clone();
        let icon = crate::widgets::chapter_icon(&chapter);
        let type_name = chapter_type_label(&chapter);

        ui.heading(format!("{icon} {type_name}"));
        ui.separator();

        // 根据章节类型渲染属性编辑器
        let mut edited_chapter = chapter.clone();
        let changed = render_chapter_properties(ui, &mut edited_chapter);

        if changed {
            let mut state = world.resource_mut::<EditorSequenceState>();
            if let Some(seq) = &mut state.current
                && let Some(ch) = seq.chapters.get_mut(idx)
            {
                *ch = edited_chapter;
                seq.dirty = true;
            }
            state.save_timer = Some(0.5);
        }
    }

    fn ui(&mut self, ui: &mut egui::Ui) {
        ui.label("需要 World 访问权限");
    }
}

/// 渲染章节属性编辑器。返回 true 如果有修改。
fn render_chapter_properties(ui: &mut egui::Ui, chapter: &mut Chapter) -> bool {
    let mut changed = false;

    match chapter {
        Chapter::Wait(duration) => {
            changed |= labeled_drag(ui, "持续时间 (秒)", duration, 0.0..=f32::MAX, 0.1);
        }

        Chapter::SpawnView {
            view_layout,
            bindings,
        } => {
            changed |= labeled_text(ui, "视图布局文件", view_layout);
            ui.separator();
            changed |= edit_hashmap_string(ui, "绑定", bindings);
        }

        Chapter::AwaitFact { condition, local } => {
            changed |= labeled_text(ui, "条件表达式", condition);
            if ui.checkbox(local, "使用局部 Facts").changed() {
                changed = true;
            }
        }

        Chapter::SetViewFact { key, value } => {
            changed |= labeled_text(ui, "Fact 键", key);
            changed |= edit_fact_value_match(ui, "值", value);
        }

        Chapter::DanmakuPerformance {
            performance,
            translation,
        } => {
            changed |= labeled_text(ui, "演出文件", performance);
            changed |= edit_option_vec2(ui, "位置", translation);
        }

        Chapter::AmPerformance {
            amproj_path,
            am_config,
            wait_for_completion,
        } => {
            changed |= labeled_text(ui, "AMPROJ 文件", amproj_path);
            changed |= edit_option_string(ui, "AM 配置", am_config);
            if ui.checkbox(wait_for_completion, "等待完成").changed() {
                changed = true;
            }
        }

        Chapter::TweenViewElement {
            duration,
            wait_for_completion,
            ..
        } => {
            changed |= labeled_drag(ui, "持续时间 (秒)", duration, 0.0..=f32::MAX, 0.1);
            if ui.checkbox(wait_for_completion, "等待完成").changed() {
                changed = true;
            }
        }

        Chapter::Sequence(children) | Chapter::Parallel(children) => {
            ui.label(format!("子章节数: {}", children.len()));
        }

        Chapter::SetPlayer(action) => {
            changed |= edit_player_action(ui, action);
        }

        Chapter::SetUI(action) => {
            changed |= edit_ui_action(ui, action);
        }

        Chapter::SetCamera(action) => {
            changed |= edit_camera_action(ui, action);
        }

        Chapter::ModifyViewElement { .. } => {
            ui.label("视图元素修改 (编辑器开发中)");
        }

        Chapter::Conditional { condition, .. } => {
            changed |= edit_fact_condition(ui, condition);
        }

        Chapter::FactSwitch {
            fact_key,
            cases,
            default,
        } => {
            changed |= labeled_text(ui, "Fact 键", fact_key);
            ui.label(format!("分支数: {}", cases.len()));
            for (i, (value, _)) in cases.iter_mut().enumerate() {
                ui.push_id(i, |ui| {
                    changed |= edit_fact_value_match(ui, &format!("分支 {i}"), value);
                });
            }
            let has_default = default.is_some();
            let mut enable_default = has_default;
            if ui.checkbox(&mut enable_default, "默认分支").changed() {
                if enable_default && !has_default {
                    *default = Some(Box::new(Chapter::Wait(0.0)));
                } else if !enable_default {
                    *default = None;
                }
                changed = true;
            }
        }

        Chapter::EmitFactEvent { event_id, data } => {
            changed |= labeled_text(ui, "事件 ID", event_id);
            ui.separator();
            changed |= edit_hashmap_string_flat(ui, "数据", data);
        }

        Chapter::ModifyFact { modifications } => {
            changed |= edit_fact_modifications(ui, modifications);
        }

        Chapter::LoadFre { files, .. } => {
            changed |= edit_string_list(ui, "FRE 文件", files);
        }

        Chapter::RunSequence {
            path,
            path_fact,
            params,
        } => {
            changed |= edit_option_string(ui, "序列路径", path);
            changed |= edit_option_string(ui, "动态路径 Fact", path_fact);
            if !params.is_empty() {
                ui.separator();
                ui.label(format!("参数: {} 个", params.len()));
            }
        }

        Chapter::LoadMap {
            path,
            generate_collision,
            process_objects,
            setup_camera_bounds,
        } => {
            changed |= labeled_text(ui, "地图路径", path);
            if ui.checkbox(generate_collision, "生成碰撞").changed() {
                changed = true;
            }
            if ui.checkbox(process_objects, "处理对象").changed() {
                changed = true;
            }
            if ui.checkbox(setup_camera_bounds, "设置摄像机边界").changed() {
                changed = true;
            }
        }

        Chapter::SetBgm { path, fade_in } => {
            changed |= edit_option_string(ui, "BGM 路径", path);
            changed |= edit_option_f32(ui, "淡入时间 (秒)", fade_in);
        }

        Chapter::Custom {
            action_type,
            params,
        } => {
            changed |= labeled_text(ui, "动作类型", action_type);
            ui.separator();
            changed |= edit_hashmap_string_flat(ui, "参数", params);
        }
    }

    changed
}
