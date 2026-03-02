//! # 回放控制面板
//!
//! Play/Pause/Stop 工具栏和序列回放状态管理。

use bevy::prelude::*;
use bevy_workbench::prelude::*;

use crate::panels::sequence_timeline::EditorSequenceState;

/// 回放状态资源。
#[derive(Resource, Default)]
pub struct PlaybackState {
    /// 当前执行到的章节索引。
    pub current_chapter: usize,
    /// 是否处于单步模式。
    pub step_mode: bool,
    /// 从指定章节开始播放（右键 "从这里播放"）。
    #[allow(dead_code)]
    pub start_from: Option<usize>,
}

/// 回放控制面板。
pub struct PlaybackPanel;

impl PlaybackPanel {
    pub fn new() -> Self {
        Self
    }
}

impl WorkbenchPanel for PlaybackPanel {
    fn id(&self) -> &str {
        "playback_control"
    }

    fn title(&self) -> String {
        "回放控制".to_string()
    }

    fn closable(&self) -> bool {
        false
    }

    fn default_visible(&self) -> bool {
        true
    }

    fn needs_world(&self) -> bool {
        true
    }

    fn ui_world(&mut self, ui: &mut egui::Ui, world: &mut World) {
        if !world.contains_resource::<PlaybackState>() {
            world.insert_resource(PlaybackState::default());
        }

        let mode = *world.resource::<State<EditorMode>>().get();
        let has_sequence = world
            .resource::<EditorSequenceState>()
            .current
            .is_some();

        ui.horizontal(|ui| {
            match mode {
                EditorMode::Edit => {
                    let play_btn = ui.add_enabled(has_sequence, egui::Button::new("▶ 播放"));
                    if play_btn.clicked() {
                        world
                            .resource_mut::<NextState<EditorMode>>()
                            .set(EditorMode::Play);
                        world.resource_mut::<PlaybackState>().current_chapter = 0;
                    }

                    let step_btn = ui.add_enabled(has_sequence, egui::Button::new("⏭ 单步"));
                    if step_btn.clicked() {
                        let mut pb = world.resource_mut::<PlaybackState>();
                        pb.step_mode = true;
                        pb.current_chapter = 0;
                        world
                            .resource_mut::<NextState<EditorMode>>()
                            .set(EditorMode::Play);
                    }
                }
                EditorMode::Play => {
                    if ui.button("⏸ 暂停").clicked() {
                        world
                            .resource_mut::<NextState<EditorMode>>()
                            .set(EditorMode::Pause);
                    }
                    if ui.button("⏹ 停止").clicked() {
                        world
                            .resource_mut::<NextState<EditorMode>>()
                            .set(EditorMode::Edit);
                    }
                    if ui.button("⏭ 下一章").clicked() {
                        world.resource_mut::<PlaybackState>().current_chapter += 1;
                    }
                }
                EditorMode::Pause => {
                    if ui.button("▶ 继续").clicked() {
                        world
                            .resource_mut::<NextState<EditorMode>>()
                            .set(EditorMode::Play);
                    }
                    if ui.button("⏹ 停止").clicked() {
                        world
                            .resource_mut::<NextState<EditorMode>>()
                            .set(EditorMode::Edit);
                    }
                    if ui.button("⏭ 下一章").clicked() {
                        world.resource_mut::<PlaybackState>().current_chapter += 1;
                    }
                }
            }

            // 显示当前状态
            ui.separator();
            let mode_label = match mode {
                EditorMode::Edit => "✏ 编辑",
                EditorMode::Play => "▶ 播放中",
                EditorMode::Pause => "⏸ 已暂停",
            };
            ui.label(mode_label);

            if mode != EditorMode::Edit {
                let chapter_idx = world.resource::<PlaybackState>().current_chapter;
                let total = world
                    .resource::<EditorSequenceState>()
                    .current
                    .as_ref()
                    .map(|s| s.chapters.len())
                    .unwrap_or(0);
                ui.label(format!("章节 {}/{total}", chapter_idx + 1));
            }
        });
    }

    fn ui(&mut self, ui: &mut egui::Ui) {
        ui.label("需要 World 访问权限");
    }
}
