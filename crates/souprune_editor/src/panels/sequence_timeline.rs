//! # 序列时间线面板
//!
//! 编辑器的核心面板，显示当前序列的章节列表。

use bevy::prelude::*;
use bevy_workbench::prelude::*;

use crate::data::EditorSequence;
use crate::widgets;

/// 当前编辑器正在编辑的序列状态。
#[derive(Resource, Default)]
pub struct EditorSequenceState {
    /// 当前打开的序列。
    pub current: Option<EditorSequence>,
    /// 选中的章节索引。
    pub selected_chapter: Option<usize>,
    /// 剪贴板（复制的章节）。
    pub clipboard: Option<souprune::core::sequencer::chapter_schema::Chapter>,
    /// 章节面板是否打开。
    pub palette_open: bool,
    /// 章节面板状态。
    pub palette_state: widgets::chapter_palette::ChapterPaletteState,
    /// 拖拽状态：正在拖拽的章节索引。
    pub drag_source: Option<usize>,
    /// 拖拽悬停目标索引。
    pub drag_target: Option<usize>,
    /// 折叠状态：已展开的章节索引集合。
    pub expanded: std::collections::HashSet<usize>,
    /// 自动保存倒计时（秒），属性修改时重置为 0.5 秒。
    pub save_timer: Option<f32>,
}

/// 序列时间线面板 — 编辑器核心工作区。
pub struct SequenceTimelinePanel;

impl SequenceTimelinePanel {
    pub fn new() -> Self {
        Self
    }
}

impl WorkbenchPanel for SequenceTimelinePanel {
    fn id(&self) -> &str {
        "sequence_timeline"
    }

    fn title(&self) -> String {
        "Sequence Timeline".to_string()
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
        // 工具栏
        render_toolbar(ui, world);

        ui.separator();

        // 章节面板（弹出式）
        render_palette_popup(ui, world);

        // 章节列表
        render_chapter_list(ui, world);
    }

    fn ui(&mut self, ui: &mut egui::Ui) {
        ui.label("需要 World 访问权限");
    }
}

fn render_toolbar(ui: &mut egui::Ui, world: &mut World) {
    ui.horizontal(|ui| {
        // 打开文件
        if ui.button("📂 打开").clicked()
            && let Some(path) = rfd_open_sequence()
        {
            match crate::data::load_sequence_from_file(&path) {
                Ok(seq) => {
                    let mut state = world.resource_mut::<EditorSequenceState>();
                    state.current = Some(seq);
                    state.selected_chapter = None;
                }
                Err(e) => {
                    warn!("打开序列失败: {e}");
                }
            }
        }

        // 保存
        let has_seq = world.resource::<EditorSequenceState>().current.is_some();
        if ui.add_enabled(has_seq, egui::Button::new("💾 保存")).clicked() {
            let state = world.resource::<EditorSequenceState>();
            if let Some(seq) = &state.current {
                if let Err(e) = crate::data::save_sequence_to_file(seq) {
                    warn!("保存序列失败: {e}");
                } else {
                    world.resource_mut::<EditorSequenceState>().current.as_mut().unwrap().dirty = false;
                    info!("序列已保存");
                }
            }
        }

        ui.separator();

        // 添加/删除/复制
        let has_selection = world.resource::<EditorSequenceState>().selected_chapter.is_some();

        if ui.add_enabled(has_seq, egui::Button::new("➕ 添加")).clicked() {
            let mut state = world.resource_mut::<EditorSequenceState>();
            state.palette_open = !state.palette_open;
        }

        if ui.add_enabled(has_selection, egui::Button::new("🗑 删除")).clicked() {
            let mut state = world.resource_mut::<EditorSequenceState>();
            if let Some(idx) = state.selected_chapter
                && let Some(seq) = &mut state.current
            {
                seq.remove_chapter(idx);
                state.selected_chapter = if seq.chapters.is_empty() {
                    None
                } else {
                    Some(idx.min(seq.chapters.len() - 1))
                };
            }
        }

        if ui.add_enabled(has_selection, egui::Button::new("📋 复制")).clicked() {
            let mut state = world.resource_mut::<EditorSequenceState>();
            if let Some(idx) = state.selected_chapter
                && let Some(seq) = &state.current
                && let Some(ch) = seq.chapters.get(idx)
            {
                state.clipboard = Some(ch.clone());
            }
        }

        let has_clipboard = world.resource::<EditorSequenceState>().clipboard.is_some();
        if ui.add_enabled(has_clipboard && has_seq, egui::Button::new("📌 粘贴")).clicked() {
            let mut state = world.resource_mut::<EditorSequenceState>();
            let ch = state.clipboard.clone();
            let idx = state.selected_chapter.unwrap_or(0);
            if let (Some(ch), Some(seq)) = (ch, &mut state.current) {
                seq.insert_chapter(idx, ch);
            }
            let new_len = state.current.as_ref().map_or(0, |s| s.chapters.len());
            state.selected_chapter = Some((idx + 1).min(new_len.saturating_sub(1)));
        }

        // 脏标记
        let dirty = world.resource::<EditorSequenceState>()
            .current.as_ref().is_some_and(|s| s.dirty);
        if dirty {
            ui.label(egui::RichText::new("● 已修改").color(egui::Color32::YELLOW));
        }
    });
}

fn render_palette_popup(ui: &mut egui::Ui, world: &mut World) {
    let is_open = world.resource::<EditorSequenceState>().palette_open;
    if !is_open {
        return;
    }

    let frame = egui::Frame::new()
        .fill(egui::Color32::from_gray(30))
        .stroke(egui::Stroke::new(1.0, egui::Color32::from_gray(80)))
        .corner_radius(6.0)
        .inner_margin(8.0);

    let mut created = None;
    frame.show(ui, |ui| {
        ui.set_max_height(300.0);
        let mut palette_state = world.resource::<EditorSequenceState>().palette_state.selected_category;
        let mut ps = widgets::chapter_palette::ChapterPaletteState {
            selected_category: palette_state,
        };
        if let Some(ch) = widgets::chapter_palette::render_chapter_palette(ui, &mut ps) {
            created = Some(ch);
        }
        palette_state = ps.selected_category;
        world.resource_mut::<EditorSequenceState>().palette_state.selected_category = palette_state;
    });

    if let Some(chapter) = created {
        let mut state = world.resource_mut::<EditorSequenceState>();
        let idx = state.selected_chapter.unwrap_or(0);
        if let Some(seq) = &mut state.current {
            seq.insert_chapter(idx, chapter);
        }
        let new_len = state.current.as_ref().map_or(0, |s| s.chapters.len());
        state.selected_chapter = Some((idx + 1).min(new_len.saturating_sub(1)));
        state.palette_open = false;
    }

    ui.separator();
}

fn render_chapter_list(ui: &mut egui::Ui, world: &mut World) {
    let state = world.resource::<EditorSequenceState>();

    let Some(seq) = &state.current else {
        ui.vertical_centered(|ui| {
            ui.add_space(40.0);
            ui.label("请打开 .sequence.ron 文件开始编辑。");
        });
        return;
    };

    let chapters = seq.chapters.clone();
    let selected = state.selected_chapter;
    let drag_source = state.drag_source;
    let chapter_count = chapters.len();

    // 标题
    ui.horizontal(|ui| {
        if let Some(path) = world.resource::<EditorSequenceState>().current.as_ref().map(|s| &s.file_path) {
            ui.label(egui::RichText::new(
                path.file_name().unwrap_or_default().to_string_lossy().to_string()
            ).strong());
        }
        ui.label(format!("— {chapter_count} 个章节"));
    });
    ui.add_space(4.0);

    // 滚动区域中渲染章节卡片（支持拖拽排序）
    let mut new_selection = selected;
    let mut drag_drop: Option<(usize, usize)> = None;
    let expanded = world.resource::<EditorSequenceState>().expanded.clone();
    let mut toggle_expand: Option<usize> = None;

    egui::ScrollArea::vertical()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            for (i, chapter) in chapters.iter().enumerate() {
                let is_selected = selected == Some(i);
                let is_drag_target = drag_source.is_some() && drag_source != Some(i);
                let is_expanded = expanded.contains(&i);
                let has_nest = widgets::has_children(chapter);

                // 拖拽目标指示线
                if world.resource::<EditorSequenceState>().drag_target == Some(i) && drag_source.is_some() {
                    let rect = ui.available_rect_before_wrap();
                    let line_y = rect.top();
                    ui.painter().line_segment(
                        [egui::pos2(rect.left(), line_y), egui::pos2(rect.right(), line_y)],
                        egui::Stroke::new(2.0, egui::Color32::from_rgb(100, 200, 255)),
                    );
                }

                // 卡片行：折叠图标 + 卡片
                ui.horizontal(|ui| {
                    if has_nest {
                        let arrow = if is_expanded { "▼" } else { "▶" };
                        if ui.small_button(arrow).clicked() {
                            toggle_expand = Some(i);
                        }
                    } else {
                        ui.add_space(20.0);
                    }

                    let response = widgets::render_chapter_card_response(ui, chapter, i, is_selected);

                    if response.clicked() {
                        new_selection = Some(i);
                    }
                    if response.drag_started() {
                        world.resource_mut::<EditorSequenceState>().drag_source = Some(i);
                    }
                    if is_drag_target && response.hovered() {
                        world.resource_mut::<EditorSequenceState>().drag_target = Some(i);
                    }
                    if response.drag_stopped() {
                        let state = world.resource::<EditorSequenceState>();
                        if let Some(src) = state.drag_source
                            && let Some(tgt) = state.drag_target
                            && src != tgt
                        {
                            drag_drop = Some((src, tgt));
                        }
                        let mut state = world.resource_mut::<EditorSequenceState>();
                        state.drag_source = None;
                        state.drag_target = None;
                    }
                });

                // 展开后显示子章节
                if has_nest && is_expanded {
                    let groups = widgets::get_children(chapter);
                    ui.indent(egui::Id::new(("nested", i)), |ui| {
                        for (label, children) in groups {
                            if !label.is_empty() {
                                ui.label(
                                    egui::RichText::new(label)
                                        .small()
                                        .color(egui::Color32::from_gray(140)),
                                );
                            }
                            for (ci, child) in children.iter().enumerate() {
                                widgets::render_chapter_card(ui, child, ci, false);
                                ui.add_space(1.0);
                            }
                        }
                    });
                }

                ui.add_space(2.0);
            }
        });

    // 切换折叠状态
    if let Some(idx) = toggle_expand {
        let mut state = world.resource_mut::<EditorSequenceState>();
        if state.expanded.contains(&idx) {
            state.expanded.remove(&idx);
        } else {
            state.expanded.insert(idx);
        }
    }

    // 执行拖拽排序
    if let Some((from, to)) = drag_drop {
        let mut state = world.resource_mut::<EditorSequenceState>();
        if let Some(seq) = &mut state.current {
            seq.move_chapter(from, to);
            state.selected_chapter = Some(to);
        }
    } else if new_selection != selected {
        world.resource_mut::<EditorSequenceState>().selected_chapter = new_selection;
    }
}

/// 打开文件对话框选择 .sequence.ron 文件。
/// 在 headless 模式下返回 None。
fn rfd_open_sequence() -> Option<std::path::PathBuf> {
    #[cfg(not(target_os = "android"))]
    {
        // 简单实现：使用 stdin 输入路径（rfd 需要额外依赖，先用 egui 内置方案）
        // TODO: 集成 rfd 或 egui file dialog
        None
    }
    #[cfg(target_os = "android")]
    {
        None
    }
}

