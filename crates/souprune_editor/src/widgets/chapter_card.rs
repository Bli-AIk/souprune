//! # Chapter Card Rendering
//!
//! # 章节卡片渲染组件
//!
//! ## Module Overview
//!
//! ## 模块概述
//!
//! Chapter card rendering widget for the editor.
//! Each Chapter type has a unique icon, color, and summary.
//!
//! 编辑器的章节卡片渲染组件。
//! 每种 Chapter 类型对应独特的图标、颜色和摘要。

use egui::Color32;
use souprune_schema::sequence::Chapter;

/// 章节分类颜色。
#[derive(Debug, Clone, Copy)]
pub enum ChapterCategory {
    /// 场景构建（绿色系）
    Scene,
    /// UI / 视图（紫色系）
    View,
    /// 流程控制（蓝色系）
    Flow,
    /// 逻辑 / FRE（黄色系）
    Logic,
    /// 战斗 / 弹幕（红色系）
    Combat,
    /// 音频（青色系）
    Audio,
}

impl ChapterCategory {
    pub fn color(&self) -> Color32 {
        match self {
            ChapterCategory::Scene => Color32::from_rgb(76, 175, 80),
            ChapterCategory::View => Color32::from_rgb(156, 39, 176),
            ChapterCategory::Flow => Color32::from_rgb(33, 150, 243),
            ChapterCategory::Logic => Color32::from_rgb(255, 193, 7),
            ChapterCategory::Combat => Color32::from_rgb(244, 67, 54),
            ChapterCategory::Audio => Color32::from_rgb(0, 188, 212),
        }
    }
}

/// 获取章节对应的分类颜色。
pub fn chapter_color(chapter: &Chapter) -> Color32 {
    chapter_category(chapter).color()
}

fn chapter_category(chapter: &Chapter) -> ChapterCategory {
    match chapter {
        Chapter::LoadMap { .. } => ChapterCategory::Scene,
        Chapter::SetPlayer(_) => ChapterCategory::Scene,
        Chapter::SetCamera(_) => ChapterCategory::Scene,

        Chapter::SpawnView { .. } => ChapterCategory::View,
        Chapter::SetViewFact { .. } => ChapterCategory::View,
        Chapter::SetViewElement { .. } => ChapterCategory::View,
        Chapter::ModifyViewElement { .. } => ChapterCategory::View,
        Chapter::SetUI(_) => ChapterCategory::View,

        Chapter::Wait(_) => ChapterCategory::Flow,
        Chapter::Sequence(_) => ChapterCategory::Flow,
        Chapter::Parallel(_) => ChapterCategory::Flow,
        Chapter::RunSequence { .. } => ChapterCategory::Flow,

        Chapter::Conditional { .. } => ChapterCategory::Logic,
        Chapter::FactSwitch { .. } => ChapterCategory::Logic,
        Chapter::AwaitFact { .. } => ChapterCategory::Logic,
        Chapter::EmitFactEvent { .. } => ChapterCategory::Logic,
        Chapter::ModifyFact { .. } => ChapterCategory::Logic,
        Chapter::LoadFre { .. } => ChapterCategory::Logic,

        Chapter::DanmakuPerformance { .. } => ChapterCategory::Combat,
        Chapter::AlightMotionPerformance { .. } => ChapterCategory::Combat,

        Chapter::SetBgm { .. } => ChapterCategory::Audio,
        Chapter::Custom { .. } => ChapterCategory::Flow,
        Chapter::LoadEnemies { .. } => ChapterCategory::Combat,
        Chapter::SplitBattleBox { .. } => ChapterCategory::Combat,
        Chapter::MergeBattleBoxes { .. } => ChapterCategory::Combat,
        Chapter::Log { .. } => ChapterCategory::Flow,
    }
}

/// 获取章节的短标签（替代 emoji 图标）。
pub fn chapter_icon(chapter: &Chapter) -> &'static str {
    match chapter {
        Chapter::SpawnView { .. } => "[V]",
        Chapter::AwaitFact { .. } => "[?]",
        Chapter::SetViewFact { .. } => "[VF]",
        Chapter::DanmakuPerformance { .. } => "[D]",
        Chapter::AlightMotionPerformance { .. } => "[AM]",
        Chapter::SetViewElement { .. } => "[~]",
        Chapter::Wait(_) => "[W]",
        Chapter::Sequence(_) => "[S]",
        Chapter::Parallel(_) => "[P]",
        Chapter::SetPlayer(_) => "[SP]",
        Chapter::SetUI(_) => "[UI]",
        Chapter::ModifyViewElement { .. } => "[MV]",
        Chapter::SetCamera(_) => "[C]",
        Chapter::Conditional { .. } => "[IF]",
        Chapter::FactSwitch { .. } => "[SW]",
        Chapter::EmitFactEvent { .. } => "[EV]",
        Chapter::ModifyFact { .. } => "[MF]",
        Chapter::LoadFre { .. } => "[FR]",
        Chapter::RunSequence { .. } => "[RS]",
        Chapter::LoadMap { .. } => "[M]",
        Chapter::SetBgm { .. } => "[B]",
        Chapter::Custom { .. } => "[X]",
        Chapter::LoadEnemies { .. } => "[LE]",
        Chapter::SplitBattleBox { .. } => "[SB]",
        Chapter::MergeBattleBoxes { .. } => "[MB]",
        Chapter::Log { .. } => "[L]",
    }
}

/// 获取章节类型名称（用于 i18n key）。
#[allow(dead_code)]
pub(crate) fn chapter_i18n_key(chapter: &Chapter) -> &'static str {
    match chapter {
        Chapter::SpawnView { .. } => "chapter-spawn-view",
        Chapter::AwaitFact { .. } => "chapter-await-fact",
        Chapter::SetViewFact { .. } => "chapter-set-view-fact",
        Chapter::DanmakuPerformance { .. } => "chapter-danmaku-performance",
        Chapter::AlightMotionPerformance { .. } => "chapter-am-performance",
        Chapter::SetViewElement { .. } => "chapter-tween-view-element",
        Chapter::Wait(_) => "chapter-wait",
        Chapter::Sequence(_) => "chapter-sequence",
        Chapter::Parallel(_) => "chapter-parallel",
        Chapter::SetPlayer(_) => "chapter-set-player",
        Chapter::SetUI(_) => "chapter-set-ui",
        Chapter::ModifyViewElement { .. } => "chapter-modify-view-element",
        Chapter::SetCamera(_) => "chapter-set-camera",
        Chapter::Conditional { .. } => "chapter-conditional",
        Chapter::FactSwitch { .. } => "chapter-fact-switch",
        Chapter::EmitFactEvent { .. } => "chapter-emit-fact-event",
        Chapter::ModifyFact { .. } => "chapter-modify-fact",
        Chapter::LoadFre { .. } => "chapter-load-fre",
        Chapter::RunSequence { .. } => "chapter-run-sequence",
        Chapter::LoadMap { .. } => "chapter-load-map",
        Chapter::SetBgm { .. } => "chapter-set-bgm",
        Chapter::Custom { .. } => "chapter-custom",
        Chapter::LoadEnemies { .. } => "chapter-load-enemies",
        Chapter::SplitBattleBox { .. } => "chapter-split-battle-box",
        Chapter::MergeBattleBoxes { .. } => "chapter-merge-battle-boxes",
        Chapter::Log { .. } => "chapter-log",
    }
}

/// 获取章节的 1 行摘要。
pub fn chapter_summary(chapter: &Chapter) -> String {
    match chapter {
        Chapter::SpawnView { view_layout, .. } => view_layout.clone(),
        Chapter::AwaitFact { condition, .. } => condition.clone(),
        Chapter::SetViewFact { key, .. } => key.clone(),
        Chapter::DanmakuPerformance { performance, .. } => performance.clone(),
        Chapter::AlightMotionPerformance { amproj_path, .. } => amproj_path.clone(),
        Chapter::SetViewElement { duration, .. } => {
            duration.map_or("instant".to_string(), |d| format!("{d}s"))
        }
        Chapter::Wait(secs) => format!("{secs}s"),
        Chapter::Sequence(children) => format!("{} chapters", children.len()),
        Chapter::Parallel(children) => format!("{} chapters", children.len()),
        Chapter::SetPlayer(action) => format!("{action:?}"),
        Chapter::SetUI(action) => format!("{action:?}"),
        Chapter::ModifyViewElement { selector, .. } => format!("{selector:?}"),
        Chapter::SetCamera(action) => format!("{action:?}"),
        Chapter::Conditional { condition, .. } => format!("{condition:?}"),
        Chapter::FactSwitch {
            fact_key, cases, ..
        } => {
            format!("{fact_key} ({} cases)", cases.len())
        }
        Chapter::EmitFactEvent { event_id, .. } => event_id.clone(),
        Chapter::ModifyFact { modifications } => format!("{} mods", modifications.len()),
        Chapter::LoadFre { files, .. } => format!("{} files", files.len()),
        Chapter::RunSequence {
            path, path_fact, ..
        } => {
            if let Some(p) = path {
                p.clone()
            } else if let Some(f) = path_fact {
                format!("${f}")
            } else {
                "(dynamic)".to_string()
            }
        }
        Chapter::LoadMap { path, .. } => path.clone(),
        Chapter::SetBgm { path, .. } => path.as_deref().unwrap_or("(stop)").to_string(),
        Chapter::Custom { action_type, .. } => action_type.clone(),
        Chapter::LoadEnemies { enemies } => format!("{} enemies", enemies.len()),
        Chapter::SplitBattleBox { source, result, .. } => {
            format!("{source} → {} + {}", result.0, result.1)
        }
        Chapter::MergeBattleBoxes {
            sources, result, ..
        } => {
            format!("{} + {} → {result}", sources.0, sources.1)
        }
        Chapter::Log { text, .. } => text.clone(),
    }
}

/// 在 egui 中渲染一个章节卡片。
///
/// 返回 `true` 如果该卡片被点击（选中）。
pub fn render_chapter_card(
    ui: &mut egui::Ui,
    chapter: &Chapter,
    index: usize,
    selected: bool,
) -> bool {
    let icon = chapter_icon(chapter);
    let summary = chapter_summary(chapter);
    let color = chapter_color(chapter);
    let type_name = chapter_type_name(chapter);

    let frame = egui::Frame::new()
        .fill(if selected {
            color.gamma_multiply(0.3)
        } else {
            Color32::from_gray(40)
        })
        .stroke(if selected {
            egui::Stroke::new(2.0, color)
        } else {
            egui::Stroke::new(1.0, Color32::from_gray(80))
        })
        .corner_radius(4.0)
        .inner_margin(8.0);

    let response = frame
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                // 索引编号
                ui.label(
                    egui::RichText::new(format!("{index:>2}"))
                        .monospace()
                        .color(Color32::from_gray(120)),
                );

                // 图标
                ui.label(egui::RichText::new(icon).size(16.0));

                // 类型名称
                ui.label(egui::RichText::new(type_name).strong().color(color));

                // 摘要
                ui.label(
                    egui::RichText::new(truncate(&summary, 40)).color(Color32::from_gray(180)),
                );
            });
        })
        .response;

    response.clicked()
}

/// 渲染章节卡片并返回 Response（用于拖拽等交互）。
pub fn render_chapter_card_response(
    ui: &mut egui::Ui,
    chapter: &Chapter,
    index: usize,
    selected: bool,
) -> egui::Response {
    let icon = chapter_icon(chapter);
    let summary = chapter_summary(chapter);
    let color = chapter_color(chapter);
    let type_name = chapter_type_name(chapter);

    let frame = egui::Frame::new()
        .fill(if selected {
            color.gamma_multiply(0.3)
        } else {
            Color32::from_gray(40)
        })
        .stroke(if selected {
            egui::Stroke::new(2.0, color)
        } else {
            egui::Stroke::new(1.0, Color32::from_gray(80))
        })
        .corner_radius(4.0)
        .inner_margin(8.0);

    let response = frame
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new(format!("{index:>2}"))
                        .monospace()
                        .color(Color32::from_gray(120)),
                );
                ui.label(egui::RichText::new(icon).size(16.0));
                ui.label(egui::RichText::new(type_name).strong().color(color));
                ui.label(
                    egui::RichText::new(truncate(&summary, 40)).color(Color32::from_gray(180)),
                );
            });
        })
        .response;

    // Enable drag sensing
    response.interact(egui::Sense::drag())
}

pub fn chapter_type_name(chapter: &Chapter) -> &'static str {
    match chapter {
        Chapter::SpawnView { .. } => "SpawnView",
        Chapter::AwaitFact { .. } => "AwaitFact",
        Chapter::SetViewFact { .. } => "SetViewFact",
        Chapter::DanmakuPerformance { .. } => "DanmakuPerformance",
        Chapter::AlightMotionPerformance { .. } => "AlightMotionPerformance",
        Chapter::SetViewElement { .. } => "SetViewElement",
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

fn truncate(s: &str, max_len: usize) -> &str {
    if s.len() <= max_len {
        s
    } else {
        let end = s.char_indices().nth(max_len).map_or(s.len(), |(i, _)| i);
        &s[..end]
    }
}

/// 检查章节是否有子章节（可折叠）。
pub fn has_children(chapter: &Chapter) -> bool {
    match chapter {
        Chapter::Sequence(children) | Chapter::Parallel(children) => !children.is_empty(),
        Chapter::Conditional { .. } => true,
        Chapter::FactSwitch { cases, .. } => !cases.is_empty(),
        _ => false,
    }
}

/// 获取章节的直接子章节列表（用于折叠展示）。
pub fn get_children(chapter: &Chapter) -> Vec<(&str, &[Chapter])> {
    match chapter {
        Chapter::Sequence(children) => vec![("", children.as_slice())],
        Chapter::Parallel(children) => vec![("", children.as_slice())],
        Chapter::Conditional {
            then_branch,
            else_branch,
            ..
        } => {
            let mut result = vec![("then", std::slice::from_ref(then_branch.as_ref()))];
            if let Some(eb) = else_branch {
                result.push(("else", std::slice::from_ref(eb.as_ref())));
            }
            result
        }
        Chapter::FactSwitch { cases, .. } => cases
            .iter()
            .map(|(_, ch)| ("case", std::slice::from_ref(ch)))
            .collect(),
        _ => vec![],
    }
}
