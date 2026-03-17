//! # Chapter Type Palette
//!
//! # 章节类型选择面板
//!
//! ## Module Overview
//!
//! ## 模块概述
//!
//! Chapter type palette widget for the editor.
//! Displays all Chapter types by category for adding new chapters.
//!
//! 编辑器的章节类型选择面板组件。
//! 按分类展示所有 Chapter 类型，用于添加新章节。

use bevy_tween::interpolation::EaseKind;
use egui::Color32;
use souprune::core::sequencer::chapter_schema::{
    CameraAction, Chapter, ElementModification, ElementSelector, FactCondition, FactValueMatch,
    PlayerAction, TweenTarget, UIAction, Value,
};

use super::chapter_card::ChapterCategory;

/// 章节分类定义。
struct CategoryDef {
    name: &'static str,
    color: Color32,
    templates: Vec<ChapterTemplate>,
}

/// 章节模板 — 创建新章节的默认值。
struct ChapterTemplate {
    name: &'static str,
    icon: &'static str,
    create: fn() -> Chapter,
}

/// 获取所有分类及其章节模板。
fn all_categories() -> Vec<CategoryDef> {
    vec![
        CategoryDef {
            name: "Flow Control",
            color: ChapterCategory::Flow.color(),
            templates: vec![
                ChapterTemplate {
                    name: "Wait",
                    icon: "[W]",
                    create: || Chapter::Wait(1.0),
                },
                ChapterTemplate {
                    name: "Sequence",
                    icon: "[S]",
                    create: || Chapter::Sequence(vec![]),
                },
                ChapterTemplate {
                    name: "Parallel",
                    icon: "[P]",
                    create: || Chapter::Parallel(vec![]),
                },
                ChapterTemplate {
                    name: "RunSequence",
                    icon: "[RS]",
                    create: || Chapter::RunSequence {
                        path: Some(String::new()),
                        path_fact: None,
                        params: Default::default(),
                    },
                },
            ],
        },
        CategoryDef {
            name: "Scene",
            color: ChapterCategory::Scene.color(),
            templates: vec![
                ChapterTemplate {
                    name: "LoadMap",
                    icon: "[M]",
                    create: || Chapter::LoadMap {
                        path: String::new(),
                        generate_collision: true,
                        process_objects: true,
                        setup_camera_bounds: true,
                    },
                },
                ChapterTemplate {
                    name: "SetPlayer",
                    icon: "[SP]",
                    create: || Chapter::SetPlayer(PlayerAction::SetActive(true)),
                },
                ChapterTemplate {
                    name: "SetCamera",
                    icon: "[C]",
                    create: || Chapter::SetCamera(CameraAction::FollowPlayer(true)),
                },
            ],
        },
        CategoryDef {
            name: "View",
            color: ChapterCategory::View.color(),
            templates: vec![
                ChapterTemplate {
                    name: "SpawnView",
                    icon: "[V]",
                    create: || Chapter::SpawnView {
                        view_layout: String::new(),
                        bindings: Default::default(),
                    },
                },
                ChapterTemplate {
                    name: "SetViewFact",
                    icon: "[VF]",
                    create: || Chapter::SetViewFact {
                        key: String::new(),
                        value: FactValueMatch::String(String::new()),
                    },
                },
                ChapterTemplate {
                    name: "ModifyViewElement",
                    icon: "[MV]",
                    create: || Chapter::ModifyViewElement {
                        selector: ElementSelector::LocalName(String::new()),
                        modification: ElementModification::SetVisibility(Value::Static(true)),
                    },
                },
                ChapterTemplate {
                    name: "TweenViewElement",
                    icon: "[~]",
                    create: || Chapter::TweenViewElement {
                        selector: ElementSelector::LocalName(String::new()),
                        target: TweenTarget::Alpha {
                            from: None,
                            to: Value::Static(1.0),
                        },
                        duration: 0.5,
                        easing: EaseKind::Linear,
                        wait_for_completion: true,
                    },
                },
                ChapterTemplate {
                    name: "SetUI",
                    icon: "[UI]",
                    create: || Chapter::SetUI(UIAction::Show(String::new())),
                },
            ],
        },
        CategoryDef {
            name: "Logic",
            color: ChapterCategory::Logic.color(),
            templates: vec![
                ChapterTemplate {
                    name: "Conditional",
                    icon: "[IF]",
                    create: || Chapter::Conditional {
                        condition: FactCondition::Always,
                        then_branch: Box::new(Chapter::Wait(0.0)),
                        else_branch: None,
                    },
                },
                ChapterTemplate {
                    name: "FactSwitch",
                    icon: "[SW]",
                    create: || Chapter::FactSwitch {
                        fact_key: String::new(),
                        cases: vec![],
                        default: None,
                    },
                },
                ChapterTemplate {
                    name: "AwaitFact",
                    icon: "[?]",
                    create: || Chapter::AwaitFact {
                        condition: String::new(),
                        local: true,
                    },
                },
                ChapterTemplate {
                    name: "EmitFactEvent",
                    icon: "[EV]",
                    create: || Chapter::EmitFactEvent {
                        event_id: String::new(),
                        data: Default::default(),
                    },
                },
                ChapterTemplate {
                    name: "ModifyFact",
                    icon: "[MF]",
                    create: || Chapter::ModifyFact {
                        modifications: vec![],
                    },
                },
                ChapterTemplate {
                    name: "LoadFre",
                    icon: "[FR]",
                    create: || Chapter::LoadFre {
                        files: vec![String::new()],
                        aggregate: Default::default(),
                    },
                },
            ],
        },
        CategoryDef {
            name: "Combat",
            color: ChapterCategory::Combat.color(),
            templates: vec![
                ChapterTemplate {
                    name: "DanmakuPerformance",
                    icon: "[D]",
                    create: || Chapter::DanmakuPerformance {
                        performance: String::new(),
                        translation: None,
                    },
                },
                ChapterTemplate {
                    name: "AlightMotionPerformance",
                    icon: "[AM]",
                    create: || Chapter::AlightMotionPerformance {
                        amproj_path: String::new(),
                        alight_motion_config: None,
                        wait_for_completion: true,
                    },
                },
            ],
        },
        CategoryDef {
            name: "Audio",
            color: ChapterCategory::Audio.color(),
            templates: vec![ChapterTemplate {
                name: "SetBgm",
                icon: "[B]",
                create: || Chapter::SetBgm {
                    path: None,
                    fade_in: None,
                },
            }],
        },
        CategoryDef {
            name: "Extension",
            color: ChapterCategory::Flow.color(),
            templates: vec![ChapterTemplate {
                name: "Custom",
                icon: "[X]",
                create: || Chapter::Custom {
                    action_type: String::new(),
                    params: Default::default(),
                },
            }],
        },
    ]
}

/// 章节类型选择面板的持久状态。
#[derive(Default)]
pub struct ChapterPaletteState {
    /// 当前选中的分类索引。
    pub selected_category: usize,
}

fn render_template_card(
    col: &mut egui::Ui,
    template: &ChapterTemplate,
    cat: &CategoryDef,
    result: &mut Option<Chapter>,
) {
    let frame = egui::Frame::new()
        .fill(Color32::from_gray(45))
        .stroke(egui::Stroke::new(1.0, cat.color.gamma_multiply(0.5)))
        .corner_radius(6.0)
        .inner_margin(10.0);

    let resp = frame
        .show(col, |ui| {
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new(template.icon).size(20.0));
                ui.label(egui::RichText::new(template.name).strong().color(cat.color));
            });
        })
        .response;

    if resp.clicked() {
        *result = Some((template.create)());
    }
    col.add_space(4.0);
}

fn render_template_columns(cols: &mut [egui::Ui], cat: &CategoryDef, result: &mut Option<Chapter>) {
    for (i, template) in cat.templates.iter().enumerate() {
        let col = &mut cols[i % 2];
        render_template_card(col, template, cat, result);
    }
}

/// 渲染章节选择面板。
///
/// 返回 `Some(Chapter)` 如果用户选择了一个类型。
pub fn render_chapter_palette(
    ui: &mut egui::Ui,
    state: &mut ChapterPaletteState,
) -> Option<Chapter> {
    let categories = all_categories();
    let mut result = None;

    // 分类 Tab 栏
    ui.horizontal_wrapped(|ui| {
        for (i, cat) in categories.iter().enumerate() {
            let text = cat.name;
            let is_selected = state.selected_category == i;
            let btn = egui::Button::new(egui::RichText::new(text).color(if is_selected {
                cat.color
            } else {
                Color32::from_gray(180)
            }));
            if ui.add(btn).clicked() {
                state.selected_category = i;
            }
        }
    });

    ui.separator();

    // 当前分类的章节模板网格
    if let Some(cat) = categories.get(state.selected_category) {
        egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                ui.columns(2, |cols| {
                    render_template_columns(cols, cat, &mut result);
                });
            });
    }

    result
}
