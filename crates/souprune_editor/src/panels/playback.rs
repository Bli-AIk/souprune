//! # 回放控制面板
//!
//! Play/Pause/Stop 工具栏和序列回放状态管理。

use bevy::prelude::*;
use bevy_workbench::prelude::*;

use crate::panels::sequence_timeline::EditorSequenceState;
use crate::widgets;
use souprune::core::sequencer::SequenceContext;

/// 回放状态资源 — 用于 UI 显示同步。
#[derive(Resource, Default)]
pub struct PlaybackState {
    /// 当前执行到的章节索引（从 SequenceContext 同步）。
    pub current_chapter: usize,
    /// 序列总章节数。
    pub total_chapters: usize,
    /// 是否处于单步模式。
    pub step_mode: bool,
    /// 从指定章节开始播放（右键 "从这里播放"）。
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
        let mode = *world.resource::<State<EditorMode>>().get();
        let has_sequence = world.resource::<EditorSequenceState>().current.is_some();

        ui.horizontal(|ui| {
            match mode {
                EditorMode::Edit => {
                    let play_btn = ui.add_enabled(has_sequence, egui::Button::new("▶ 播放"));
                    if play_btn.clicked() {
                        let mut pb = world.resource_mut::<PlaybackState>();
                        pb.current_chapter = 0;
                        pb.step_mode = false;
                        world
                            .resource_mut::<NextState<EditorMode>>()
                            .set(EditorMode::Play);
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
                }
            }

            ui.separator();
            let mode_label = match mode {
                EditorMode::Edit => "编辑",
                EditorMode::Play => "▶ 播放中",
                EditorMode::Pause => "⏸ 已暂停",
            };
            ui.label(mode_label);

            if mode != EditorMode::Edit {
                let pb = world.resource::<PlaybackState>();
                let remaining = world.resource::<SequenceContext>().chapters.len();
                let total = pb.total_chapters;
                let processed = total.saturating_sub(remaining);

                ui.label(format!("章节 {processed}/{total}"));

                // 显示当前章节信息
                if let Some(seq) = &world.resource::<EditorSequenceState>().current
                    && let Some(ch) = seq.chapters.get(processed)
                {
                    let type_name = widgets::chapter_type_name(ch);
                    let summary = widgets::chapter_summary(ch);
                    ui.separator();
                    ui.label(
                        egui::RichText::new(format!("{type_name}: {summary}"))
                            .color(widgets::chapter_color(ch)),
                    );
                }
            }
        });
    }

    fn ui(&mut self, ui: &mut egui::Ui) {
        ui.label("需要 World 访问权限");
    }
}

/// 同步 PlaybackState 与真实 SequenceContext（在 GameSchedule 中运行）。
pub fn playback_sync_system(
    context: Res<SequenceContext>,
    mut playback: ResMut<PlaybackState>,
    mut next_mode: ResMut<NextState<EditorMode>>,
    editor_seq: Res<EditorSequenceState>,
) {
    if let Some(seq) = &editor_seq.current {
        playback.total_chapters = seq.chapters.len();
        let new_chapter = seq.chapters.len().saturating_sub(context.chapters.len());
        // 单步模式：章节前进时自动暂停
        if playback.step_mode && new_chapter > playback.current_chapter {
            next_mode.set(EditorMode::Pause);
        }
        playback.current_chapter = new_chapter;
    }
}
