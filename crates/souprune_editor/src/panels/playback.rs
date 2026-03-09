//! # Playback Control Panel
//!
//! # 回放控制面板
//!
//! ## Module Overview
//!
//! ## 模块概述
//!
//! Playback control panel for the editor.
//! Provides Play/Pause/Stop toolbar and sequence playback state management.
//!
//! 编辑器的回放控制面板。
//! 提供播放/暂停/停止工具栏和序列回放状态管理。

use bevy::prelude::*;
use bevy_workbench::prelude::*;

use crate::i18n::{t, t_args};
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

fn draw_edit_mode_buttons(ui: &mut egui::Ui, world: &mut World, has_sequence: bool) {
    let play_btn = ui.add_enabled(has_sequence, egui::Button::new(t(world, "playback-play")));
    if play_btn.clicked() {
        let mut pb = world.resource_mut::<PlaybackState>();
        pb.current_chapter = 0;
        pb.step_mode = false;
        world
            .resource_mut::<NextState<EditorMode>>()
            .set(EditorMode::Play);
    }

    let step_btn = ui.add_enabled(has_sequence, egui::Button::new(t(world, "playback-step")));
    if step_btn.clicked() {
        let mut pb = world.resource_mut::<PlaybackState>();
        pb.step_mode = true;
        pb.current_chapter = 0;
        world
            .resource_mut::<NextState<EditorMode>>()
            .set(EditorMode::Play);
    }
}

fn draw_play_mode_buttons(ui: &mut egui::Ui, world: &mut World) {
    if ui.button(t(world, "playback-pause")).clicked() {
        world
            .resource_mut::<NextState<EditorMode>>()
            .set(EditorMode::Pause);
    }
    if ui.button(t(world, "playback-stop")).clicked() {
        world
            .resource_mut::<NextState<EditorMode>>()
            .set(EditorMode::Edit);
    }
}

fn draw_pause_mode_buttons(ui: &mut egui::Ui, world: &mut World) {
    if ui.button(t(world, "playback-resume")).clicked() {
        world
            .resource_mut::<NextState<EditorMode>>()
            .set(EditorMode::Play);
    }
    if ui.button(t(world, "playback-stop")).clicked() {
        world
            .resource_mut::<NextState<EditorMode>>()
            .set(EditorMode::Edit);
    }
}

fn show_playback_progress(ui: &mut egui::Ui, world: &World) {
    let pb = world.resource::<PlaybackState>();
    let remaining = world.resource::<SequenceContext>().chapters.len();
    let total = pb.total_chapters;
    let processed = total.saturating_sub(remaining);

    let mut args = bevy_workbench::i18n::FluentArgs::new();
    args.set("processed", processed as i64);
    args.set("total", total as i64);
    ui.label(t_args(world, "playback-chapter-progress", &args));

    let seq_state = world.resource::<EditorSequenceState>();
    let Some(seq) = &seq_state.current else {
        return;
    };
    let Some(ch) = seq.chapters.get(processed) else {
        return;
    };
    let type_name = widgets::chapter_type_name(ch);
    let summary = widgets::chapter_summary(ch);
    ui.separator();
    ui.label(
        egui::RichText::new(format!("{type_name}: {summary}")).color(widgets::chapter_color(ch)),
    );
}

/// 回放控制面板。
pub struct PlaybackPanel {
    cached_title: String,
}

impl PlaybackPanel {
    pub fn new() -> Self {
        Self {
            cached_title: "Playback Control".to_string(),
        }
    }
}

impl WorkbenchPanel for PlaybackPanel {
    fn id(&self) -> &str {
        "playback_control"
    }

    fn title(&self) -> String {
        self.cached_title.clone()
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
        self.cached_title = t(world, "panel-playback");
        let mode = *world.resource::<State<EditorMode>>().get();
        let has_sequence = world.resource::<EditorSequenceState>().current.is_some();

        ui.horizontal(|ui| {
            match mode {
                EditorMode::Edit => draw_edit_mode_buttons(ui, world, has_sequence),
                EditorMode::Play => draw_play_mode_buttons(ui, world),
                EditorMode::Pause => draw_pause_mode_buttons(ui, world),
            }

            ui.separator();
            let mode_label = match mode {
                EditorMode::Edit => t(world, "playback-mode-edit"),
                EditorMode::Play => t(world, "playback-mode-playing"),
                EditorMode::Pause => t(world, "playback-mode-paused"),
            };
            ui.label(mode_label);

            if mode != EditorMode::Edit {
                show_playback_progress(ui, world);
            }
        });
    }

    fn ui(&mut self, ui: &mut egui::Ui) {
        ui.label("Requires World access");
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
