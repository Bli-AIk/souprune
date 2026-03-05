//! View 可视化编辑器面板 — 三面板布局：Node Tree + Preview + Inspector。

use std::path::{Path, PathBuf};

use bevy::prelude::*;
use bevy_workbench::prelude::*;
use souprune::core::view::layout::{
    DataRequirement, InitialFactValue, RepeatDef, SpriteDef, TextDef, ViewBoxLogicDef,
    ViewLayoutAsset, ViewNodeDef,
};

use super::view_preview::ViewPreviewState;
use crate::editors::SubEditorManager;
use crate::i18n::{t, t_args};
use crate::widgets::property_editors::{edit_option_string, labeled_text};
use crate::widgets::view_widgets::{
    edit_color, edit_expression, edit_font_def, edit_option_color, edit_option_transform,
    edit_option_vec2_tuple, edit_string_string_map, edit_tag_list, edit_vec2, edit_vec3,
};

/// View 编辑器状态资源。
#[derive(Resource, Default)]
pub struct ViewEditorState {
    pub file_path: Option<PathBuf>,
    pub layout: Option<ViewLayoutAsset>,
    pub parse_error: Option<String>,
    pub selected_node: Option<Vec<usize>>,
    pub dirty: bool,
    /// Incremented on each load/layout change; used for dirty-checking in preview rebuild.
    pub generation: u64,
}

impl ViewEditorState {
    pub fn load(&mut self, path: &Path) {
        match std::fs::read_to_string(path) {
            Ok(content) => {
                self.file_path = Some(path.to_path_buf());
                self.dirty = false;
                self.selected_node = None;
                self.parse_error = None;
                self.generation += 1;
                match ron::from_str::<ViewLayoutAsset>(&content) {
                    Ok(layout) => self.layout = Some(layout),
                    Err(e) => {
                        self.parse_error = Some(format!("{e}"));
                        self.layout = None;
                    }
                }
            }
            Err(e) => {
                warn!("加载 View 文件失败: {e}");
                self.parse_error = Some(format!("Failed to read file: {e}"));
            }
        }
    }
}

pub struct ViewEditorPanel {
    cached_title: String,
}

impl ViewEditorPanel {
    pub fn new() -> Self {
        Self {
            cached_title: "View Editor".to_string(),
        }
    }
}

impl WorkbenchPanel for ViewEditorPanel {
    fn id(&self) -> &str {
        "view_editor_preview"
    }
    fn title(&self) -> String {
        self.cached_title.clone()
    }
    fn ui(&mut self, ui: &mut egui::Ui) {
        ui.label("Requires World access");
    }
    fn ui_world(&mut self, ui: &mut egui::Ui, world: &mut World) {
        self.cached_title = t(world, "panel-view-editor");
        check_pending_open(world);

        let has_layout = world
            .get_resource::<ViewEditorState>()
            .is_some_and(|s| s.layout.is_some());

        if !has_layout {
            let err = world
                .get_resource::<ViewEditorState>()
                .and_then(|s| s.parse_error.clone());
            if let Some(err) = err {
                let mut args = bevy_workbench::i18n::FluentArgs::new();
                args.set("err", err);
                ui.colored_label(
                    egui::Color32::RED,
                    t_args(world, "label-parse-error", &args),
                );
            } else {
                ui.label(t(world, "label-no-view-open"));
            }
            return;
        }

        render_title_bar(ui, world);
        ui.separator();

        let available = ui.available_size();
        egui::SidePanel::left("view_node_tree")
            .resizable(true)
            .default_width(available.x * 0.2)
            .min_width(120.0)
            .show_inside(ui, |ui| render_node_tree(ui, world));

        egui::SidePanel::right("view_inspector")
            .resizable(true)
            .default_width(available.x * 0.35)
            .min_width(200.0)
            .show_inside(ui, |ui| render_inspector(ui, world));

        let preview_not_init = t(world, "label-preview-not-init");
        egui::CentralPanel::default().show_inside(ui, |ui| {
            let labels = {
                let zoom_val = world
                    .get_resource::<ViewPreviewState>()
                    .map_or(1.0, |s| s.zoom);
                let mut args = bevy_workbench::i18n::FluentArgs::new();
                args.set("percent", format!("{:.0}", zoom_val * 100.0));
                super::view_preview::PreviewLabels {
                    stop: t(world, "preview-stop"),
                    play: t(world, "preview-play"),
                    zoom: t_args(world, "preview-zoom", &args),
                    reset: t(world, "preview-reset"),
                    input_active: t(world, "preview-input-active"),
                }
            };
            let mut preview = world.get_resource_mut::<ViewPreviewState>();
            match preview.as_mut() {
                Some(state) => super::view_preview::render_preview_ui(ui, state, &labels),
                None => {
                    ui.label(&preview_not_init);
                }
            }
        });
    }
    fn needs_world(&self) -> bool {
        true
    }

    fn closable(&self) -> bool {
        true
    }

    fn default_visible(&self) -> bool {
        true
    }
}

fn check_pending_open(world: &mut World) {
    let pending_path = {
        let mgr = world.resource::<SubEditorManager>();
        if mgr.active_editor.as_deref() != Some("view_editor") {
            return;
        }
        mgr.nav_stack.last().map(|e| PathBuf::from(&e.file_path))
    };
    if let Some(path) = pending_path {
        let already_open = world
            .get_resource::<ViewEditorState>()
            .is_some_and(|s| s.file_path.as_deref() == Some(path.as_path()));
        if !already_open {
            world.get_resource_or_init::<ViewEditorState>().load(&path);
        }
    }
}

fn render_title_bar(ui: &mut egui::Ui, world: &mut World) {
    let (title, is_dirty) = {
        let state = world.resource::<ViewEditorState>();
        let title = state
            .file_path
            .as_ref()
            .and_then(|p| p.file_name())
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "View".to_string());
        (title, state.dirty)
    };
    let dirty = if is_dirty { " *" } else { "" };
    let save_text = t(world, "action-save");
    ui.horizontal(|ui| {
        ui.heading(format!("{title}{dirty}"));
        if is_dirty && ui.button(&save_text).clicked() {
            save_view(world);
        }
    });
}

fn save_view(world: &mut World) {
    let serialized = {
        let state = world.resource::<ViewEditorState>();
        match (&state.file_path, &state.layout) {
            (Some(path), Some(layout)) => {
                let config = ron::ser::PrettyConfig::default()
                    .struct_names(true)
                    .enumerate_arrays(false);
                match ron::ser::to_string_pretty(layout, config) {
                    Ok(content) => Some((path.clone(), content)),
                    Err(e) => {
                        warn!("序列化失败: {e}");
                        None
                    }
                }
            }
            _ => None,
        }
    };
    if let Some((path, content)) = serialized {
        match std::fs::write(&path, &content) {
            Ok(()) => {
                info!("已保存 View: {}", path.display());
                world.resource_mut::<ViewEditorState>().dirty = false;
            }
            Err(e) => warn!("保存失败: {e}"),
        }
    }
}

// ─── Node Tree ──────────────────────────────────────────────

fn render_node_tree(ui: &mut egui::Ui, world: &mut World) {
    let heading_text = t(world, "view-node-tree");
    let add_root_text = t(world, "view-add-root");
    ui.horizontal(|ui| {
        ui.heading(&heading_text);
        if ui.small_button("+").on_hover_text(&add_root_text).clicked() {
            let mut state = world.resource_mut::<ViewEditorState>();
            if let Some(layout) = &mut state.layout {
                layout.roots.push(new_empty_node("new_root"));
                state.dirty = true;
            }
        }
    });
    ui.separator();

    let (layout, selected) = {
        let s = world.resource::<ViewEditorState>();
        (s.layout.clone(), s.selected_node.clone())
    };
    let Some(layout) = layout else { return };

    // 收集树操作（右键菜单触发的操作，延迟执行避免借用冲突）
    let mut tree_action: Option<TreeAction> = None;

    egui::ScrollArea::vertical().show(ui, |ui| {
        for (i, root) in layout.roots.iter().enumerate() {
            render_tree_node(ui, root, &[i], &selected, &mut tree_action, world);
        }
    });

    // 执行延迟操作
    if let Some(action) = tree_action {
        apply_tree_action(world, action);
    }
}

/// 节点树操作。
enum TreeAction {
    Select(Vec<usize>),
    AddChild(Vec<usize>),
    Duplicate(Vec<usize>),
    Delete(Vec<usize>),
    MoveUp(Vec<usize>),
    MoveDown(Vec<usize>),
}

fn render_tree_node(
    ui: &mut egui::Ui,
    node: &ViewNodeDef,
    path: &[usize],
    selected: &Option<Vec<usize>>,
    action: &mut Option<TreeAction>,
    world: &World,
) {
    let is_sel = selected.as_ref().is_some_and(|s| s == path);
    let icon = if node.sprite.is_some() || node.state_sprite.is_some() {
        "S "
    } else if !node.texts.is_empty() {
        "T "
    } else if node.view_box.is_some() {
        "B "
    } else {
        "  "
    };
    let label = format!("{icon}{}", node.name);
    let path_id = path
        .iter()
        .map(|i| i.to_string())
        .collect::<Vec<_>>()
        .join("_");

    let show_ctx = |ui: &mut egui::Ui, path: &[usize], action: &mut Option<TreeAction>| {
        ui.menu_button("...", |ui| {
            let p = path.to_vec();
            for (label, act) in [
                (t(world, "tree-add-child"), TreeAction::AddChild(p.clone())),
                (t(world, "action-copy"), TreeAction::Duplicate(p.clone())),
                (t(world, "tree-move-up"), TreeAction::MoveUp(p.clone())),
                (t(world, "tree-move-down"), TreeAction::MoveDown(p.clone())),
            ] {
                if ui.button(label).clicked() {
                    *action = Some(act);
                    ui.close();
                }
            }
            ui.separator();
            if ui
                .button(egui::RichText::new(t(world, "action-delete")).color(egui::Color32::RED))
                .clicked()
            {
                *action = Some(TreeAction::Delete(path.to_vec()));
                ui.close();
            }
        });
    };

    if node.children.is_empty() {
        ui.horizontal(|ui| {
            if ui.selectable_label(is_sel, &label).clicked() {
                *action = Some(TreeAction::Select(path.to_vec()));
            }
            show_ctx(ui, path, action);
        });
    } else {
        let id = egui::Id::new(format!("vn_{path_id}"));
        let cs =
            egui::collapsing_header::CollapsingState::load_with_default_open(ui.ctx(), id, true);
        cs.show_header(ui, |ui| {
            if ui.selectable_label(is_sel, &label).clicked() {
                *action = Some(TreeAction::Select(path.to_vec()));
            }
            show_ctx(ui, path, action);
        })
        .body(|ui| {
            for (i, child) in node.children.iter().enumerate() {
                let mut cp = path.to_vec();
                cp.push(i);
                render_tree_node(ui, child, &cp, selected, action, world);
            }
        });
    }
}

fn new_empty_node(name: &str) -> ViewNodeDef {
    ViewNodeDef {
        name: name.to_string(),
        tags: Vec::new(),
        style: Default::default(),
        visible_when: None,
        background_color: None,
        border_color: None,
        image: None,
        sprite: None,
        state_sprite: None,
        texts: Vec::new(),
        view_box: None,
        children: Vec::new(),
        camera_anchored: false,
        repeat: None,
    }
}

fn apply_tree_action(world: &mut World, action: TreeAction) {
    match action {
        TreeAction::Select(path) => {
            world.resource_mut::<ViewEditorState>().selected_node = Some(path);
        }
        TreeAction::AddChild(path) => {
            let mut state = world.resource_mut::<ViewEditorState>();
            if let Some(layout) = &mut state.layout
                && let Some(parent) = find_node_by_path_mut(&mut layout.roots, &path)
            {
                parent.children.push(new_empty_node("new_node"));
                state.dirty = true;
            }
        }
        TreeAction::Duplicate(path) => {
            let mut state = world.resource_mut::<ViewEditorState>();
            if let Some(layout) = &mut state.layout {
                let cloned = find_node_by_path(&layout.roots, &path).cloned();
                if let Some(mut node) = cloned {
                    node.name = format!("{}_copy", node.name);
                    let parent_children = parent_children_mut(&mut layout.roots, &path);
                    if let Some((siblings, idx)) = parent_children {
                        siblings.insert(idx + 1, node);
                        state.dirty = true;
                    }
                }
            }
        }
        TreeAction::Delete(path) => {
            let mut state = world.resource_mut::<ViewEditorState>();
            if let Some(layout) = &mut state.layout
                && let Some((siblings, idx)) = parent_children_mut(&mut layout.roots, &path)
            {
                siblings.remove(idx);
                state.dirty = true;
                // 清除选中（可能指向已删除的节点）
                state.selected_node = None;
            }
        }
        TreeAction::MoveUp(path) => {
            let mut state = world.resource_mut::<ViewEditorState>();
            if let Some(layout) = &mut state.layout
                && let Some((siblings, idx)) = parent_children_mut(&mut layout.roots, &path)
                && idx > 0
            {
                siblings.swap(idx, idx - 1);
                state.dirty = true;
                let mut new_path = path.clone();
                *new_path.last_mut().unwrap() = idx - 1;
                state.selected_node = Some(new_path);
            }
        }
        TreeAction::MoveDown(path) => {
            let mut state = world.resource_mut::<ViewEditorState>();
            if let Some(layout) = &mut state.layout
                && let Some((siblings, idx)) = parent_children_mut(&mut layout.roots, &path)
                && idx + 1 < siblings.len()
            {
                siblings.swap(idx, idx + 1);
                state.dirty = true;
                let mut new_path = path.clone();
                *new_path.last_mut().unwrap() = idx + 1;
                state.selected_node = Some(new_path);
            }
        }
    }
}

// ─── Inspector ──────────────────────────────────────────────

fn render_inspector(ui: &mut egui::Ui, world: &mut World) {
    ui.heading(t(world, "view-properties"));
    ui.separator();

    // Clone layout + selected path to avoid borrow conflicts
    let (layout_opt, sel) = {
        let s = world.resource::<ViewEditorState>();
        (s.layout.clone(), s.selected_node.clone())
    };
    let Some(mut layout) = layout_opt else {
        ui.label(t(world, "label-no-data"));
        return;
    };

    // Node property editing (requires selected node)
    if let Some(path) = sel {
        if find_node_by_path(&layout.roots, &path).is_none() {
            ui.colored_label(egui::Color32::RED, t(world, "label-node-path-invalid"));
        } else {
            let mut changed = false;

            egui::ScrollArea::vertical().show(ui, |ui| {
                let mut node = find_node_by_path(&layout.roots, &path).unwrap().clone();

                changed |= edit_node_basics(world, ui, &mut node);
                changed |= edit_node_sprite(world, ui, &mut node.sprite);
                changed |= edit_node_state_sprite(world, ui, &mut node.state_sprite);
                changed |= edit_node_texts(world, ui, &mut node.texts);
                changed |= edit_node_view_box(world, ui, &mut node.view_box);
                changed |= edit_node_repeat(world, ui, &mut node.repeat);

                if changed {
                    *find_node_by_path_mut(&mut layout.roots, &path).unwrap() = node;
                }

                // Layout-level sections (requires, facts)
                changed |= edit_data_requirements(world, ui, &mut layout.requires);
                changed |= edit_initial_facts(world, ui, &mut layout.facts);
            });

            if changed {
                let mut state = world.resource_mut::<ViewEditorState>();
                state.layout = Some(layout.clone());
                state.dirty = true;
            }
        }
    } else {
        ui.label(t(world, "label-select-node"));
    }
    ui.separator();

    // FRE section — always sync regardless of node selection
    let view_path = world.resource::<ViewEditorState>().file_path.clone();
    if let Some(vp) = &view_path {
        world
            .get_resource_or_init::<super::view_fre_panel::ViewFreState>()
            .sync_for_view(vp, &layout.requires);
    }

    // Check if we're in Play mode and find the ViewRoot entity
    let preview_state = world.get_resource::<super::view_preview::ViewPreviewState>();
    let view_root_entity = preview_state.filter(|ps| ps.playing).and_then(|ps| {
        ps.preview_entities
            .iter()
            .find(|e| {
                world
                    .get::<souprune::core::view::components::ViewRoot>(**e)
                    .is_some()
            })
            .copied()
    });

    if let Some(entity) = view_root_entity {
        // Play mode: render FRE section first (rules only), then live facts separately
        {
            let mut fre = world.get_resource_or_init::<super::view_fre_panel::ViewFreState>();
            super::view_fre_panel::render_view_fre_section(ui, &mut fre, None);
        }
        // Live fact simulator bound to ViewRoot.local_facts
        if let Some(mut view_root) =
            world.get_mut::<souprune::core::view::components::ViewRoot>(entity)
        {
            super::view_fre_panel::render_live_facts_section(ui, &mut view_root.local_facts);
        }
    } else {
        let mut fre = world.get_resource_or_init::<super::view_fre_panel::ViewFreState>();
        super::view_fre_panel::render_view_fre_section(ui, &mut fre, None);
    }
}

fn edit_node_basics(world: &World, ui: &mut egui::Ui, node: &mut ViewNodeDef) -> bool {
    let mut changed = false;
    egui::CollapsingHeader::new(t(world, "view-basics"))
        .default_open(true)
        .show(ui, |ui| {
            changed |= labeled_text(ui, &t(world, "prop-name"), &mut node.name);
            changed |= edit_tag_list(ui, &t(world, "prop-tags"), &mut node.tags);
            changed |= edit_expression(ui, "visible_when", &mut node.visible_when);
            ui.horizontal(|ui| {
                ui.label("camera_anchored:");
                changed |= ui.checkbox(&mut node.camera_anchored, "").changed();
            });
        });
    changed
}

fn edit_node_sprite(world: &World, ui: &mut egui::Ui, sprite_opt: &mut Option<SpriteDef>) -> bool {
    let mut changed = false;
    let mut has = sprite_opt.is_some();
    if ui.checkbox(&mut has, "Sprite").changed() {
        if has {
            *sprite_opt = Some(SpriteDef {
                visual: souprune::core::visual::Visual::default(),
                initial_state: None,
                color: None,
                flip_x: false,
                flip_y: false,
                transform: None,
                custom_shader: None,
                shader_params: None,
                pivot: None,
                frame_duration: None,
                visible_when: None,
                #[allow(deprecated)]
                hp_bar_source: None,
                material: None,
            });
        } else {
            *sprite_opt = None;
        }
        changed = true;
    }
    let Some(sprite) = sprite_opt else {
        return changed;
    };
    egui::CollapsingHeader::new("  Sprite")
        .default_open(true)
        .show(ui, |ui| {
            // Visual path
            ui.horizontal(|ui| {
                ui.label("visual:");
                if ui.text_edit_singleline(&mut sprite.visual.0).changed() {
                    changed = true;
                }
            });
            // Color
            changed |= edit_option_color(ui, &t(world, "view-color"), &mut sprite.color);
            // Flip
            ui.horizontal(|ui| {
                if ui.checkbox(&mut sprite.flip_x, "flip_x").changed() {
                    changed = true;
                }
                if ui.checkbox(&mut sprite.flip_y, "flip_y").changed() {
                    changed = true;
                }
            });
            // Transform
            changed |= edit_option_transform(ui, "Transform", &mut sprite.transform);
            // Pivot
            changed |= edit_option_vec2_tuple(ui, "Pivot", &mut sprite.pivot);
            // visible_when
            changed |= edit_expression(ui, "visible_when", &mut sprite.visible_when);
            // initial_state
            changed |= edit_option_string(ui, "initial_state", &mut sprite.initial_state);
        });
    changed
}

fn edit_node_state_sprite(
    world: &World,
    ui: &mut egui::Ui,
    ss_opt: &mut Option<souprune::core::view::layout::StateSpriteConfig>,
) -> bool {
    let Some(ss) = ss_opt else { return false };
    let mut changed = false;
    egui::CollapsingHeader::new("State Sprite")
        .default_open(true)
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label("default:");
                if ui.text_edit_singleline(&mut ss.default).changed() {
                    changed = true;
                }
            });
            changed |= edit_string_string_map(ui, &t(world, "prop-variants"), &mut ss.variants);
            changed |= edit_option_transform(ui, "Transform", &mut ss.transform);
            changed |= edit_expression(ui, "visible_when", &mut ss.visible_when);
        });
    changed
}

fn edit_node_texts(world: &World, ui: &mut egui::Ui, texts: &mut Vec<TextDef>) -> bool {
    if texts.is_empty() {
        return false;
    }
    let mut changed = false;
    egui::CollapsingHeader::new("Text")
        .default_open(true)
        .show(ui, |ui| {
            let mut to_remove = None;
            for (i, text) in texts.iter_mut().enumerate() {
                ui.push_id(i, |ui| {
                    ui.group(|ui| {
                        ui.horizontal(|ui| {
                            ui.label("id:");
                            if ui.text_edit_singleline(&mut text.id).changed() {
                                changed = true;
                            }
                            if ui.small_button("x").clicked() {
                                to_remove = Some(i);
                            }
                        });
                        changed |= edit_option_string(ui, "content", &mut text.content);
                        changed |= edit_color(ui, &t(world, "view-color"), &mut text.color);
                        changed |= edit_font_def(ui, &t(world, "view-font"), &mut text.font);
                        changed |= edit_vec2(ui, "world_scale", &mut text.world_scale);
                        changed |= edit_expression(ui, "visible_when", &mut text.visible_when);
                    });
                });
            }
            if let Some(i) = to_remove {
                texts.remove(i);
                changed = true;
            }
        });
    changed
}

fn edit_node_view_box(
    world: &World,
    ui: &mut egui::Ui,
    vb_opt: &mut Option<ViewBoxLogicDef>,
) -> bool {
    let Some(vb) = vb_opt else { return false };
    let mut changed = false;
    egui::CollapsingHeader::new("ViewBox")
        .default_open(true)
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(t(world, "view-width"));
                changed |= ui
                    .add(egui::DragValue::new(&mut vb.width).speed(1.0))
                    .changed();
                ui.label(t(world, "view-height"));
                changed |= ui
                    .add(egui::DragValue::new(&mut vb.height).speed(1.0))
                    .changed();
            });
            ui.horizontal(|ui| {
                ui.label("border:");
                changed |= ui
                    .add(egui::DragValue::new(&mut vb.border_width).speed(0.5))
                    .changed();
            });
            changed |= edit_vec3(ui, "offset", &mut vb.offset);
            changed |= edit_option_color(ui, "fill_color", &mut vb.fill_color);
            changed |= edit_option_string(ui, "structure_file", &mut vb.structure_file);
        });
    changed
}

fn edit_node_repeat(world: &World, ui: &mut egui::Ui, repeat_opt: &mut Option<RepeatDef>) -> bool {
    let Some(repeat) = repeat_opt else {
        return false;
    };
    let mut changed = false;
    egui::CollapsingHeader::new(t(world, "view-repeat"))
        .default_open(true)
        .show(ui, |ui| {
            changed |= labeled_text(ui, "source", &mut repeat.source);
            // limit
            let mut has_limit = repeat.limit.is_some();
            ui.horizontal(|ui| {
                if ui.checkbox(&mut has_limit, "limit").changed() {
                    repeat.limit = if has_limit { Some(10) } else { None };
                    changed = true;
                }
                if let Some(limit) = &mut repeat.limit {
                    let mut v = *limit as f64;
                    if ui
                        .add(egui::DragValue::new(&mut v).speed(1.0).range(0.0..=1000.0))
                        .changed()
                    {
                        *limit = v as usize;
                        changed = true;
                    }
                }
            });
            changed |= edit_option_string(ui, "index_var", &mut repeat.index_var);
            changed |= edit_option_string(ui, "item_var", &mut repeat.item_var);
        });
    changed
}

fn edit_data_requirements(
    world: &World,
    ui: &mut egui::Ui,
    requires: &mut Vec<DataRequirement>,
) -> bool {
    if requires.is_empty() {
        return false;
    }
    let mut changed = false;
    egui::CollapsingHeader::new(t(world, "view-data-requirements"))
        .default_open(false)
        .show(ui, |ui| {
            let mut to_remove = None;
            for (i, req) in requires.iter_mut().enumerate() {
                ui.push_id(i, |ui| {
                    ui.horizontal(|ui| {
                        match req {
                            DataRequirement::File(path) => {
                                ui.label("File:");
                                if ui.text_edit_singleline(path).changed() {
                                    changed = true;
                                }
                            }
                            DataRequirement::Interface { interface, expects } => {
                                ui.label("Interface:");
                                if ui.text_edit_singleline(interface).changed() {
                                    changed = true;
                                }
                                ui.label(format!("({})", expects.join(", ")));
                            }
                        }
                        if ui.small_button("x").clicked() {
                            to_remove = Some(i);
                        }
                    });
                });
            }
            if let Some(i) = to_remove {
                requires.remove(i);
                changed = true;
            }
        });
    changed
}

fn edit_initial_facts(
    world: &World,
    ui: &mut egui::Ui,
    facts_opt: &mut Option<std::collections::HashMap<String, InitialFactValue>>,
) -> bool {
    let Some(facts) = facts_opt else {
        return false;
    };
    let mut changed = false;
    egui::CollapsingHeader::new(t(world, "view-initial-facts"))
        .default_open(false)
        .show(ui, |ui| {
            let keys: Vec<String> = facts.keys().cloned().collect();
            for key in &keys {
                if let Some(val) = facts.get_mut(key) {
                    changed |= edit_initial_fact_value(ui, key, val);
                }
            }
        });
    changed
}

fn edit_initial_fact_value(ui: &mut egui::Ui, key: &str, val: &mut InitialFactValue) -> bool {
    let mut changed = false;
    ui.horizontal(|ui| {
        ui.label(format!("{key}:"));
        match val {
            InitialFactValue::Int(v) => {
                let mut f = *v as f64;
                if ui.add(egui::DragValue::new(&mut f).speed(1.0)).changed() {
                    *v = f as i64;
                    changed = true;
                }
            }
            InitialFactValue::Float(v) => {
                if ui.add(egui::DragValue::new(v).speed(0.1)).changed() {
                    changed = true;
                }
            }
            InitialFactValue::Bool(v) => {
                if ui.checkbox(v, "").changed() {
                    changed = true;
                }
            }
            InitialFactValue::String(s) => {
                if ui.text_edit_singleline(s).changed() {
                    changed = true;
                }
            }
            InitialFactValue::StringList(list) => {
                ui.label(format!("[{}]", list.join(", ")));
            }
            InitialFactValue::IntList(list) => {
                ui.label(format!("{list:?}"));
            }
        }
    });
    changed
}

// ─── Helpers ────────────────────────────────────────────────
fn find_node_by_path<'a>(roots: &'a [ViewNodeDef], path: &[usize]) -> Option<&'a ViewNodeDef> {
    if path.is_empty() {
        return None;
    }
    let mut current = roots.get(path[0])?;
    for &idx in &path[1..] {
        current = current.children.get(idx)?;
    }
    Some(current)
}

fn find_node_by_path_mut<'a>(
    roots: &'a mut [ViewNodeDef],
    path: &[usize],
) -> Option<&'a mut ViewNodeDef> {
    if path.is_empty() {
        return None;
    }
    let mut current = roots.get_mut(path[0])?;
    for &idx in &path[1..] {
        current = current.children.get_mut(idx)?;
    }
    Some(current)
}

/// 获取路径所在父节点的 children 切片和该节点的索引。
fn parent_children_mut<'a>(
    roots: &'a mut Vec<ViewNodeDef>,
    path: &[usize],
) -> Option<(&'a mut Vec<ViewNodeDef>, usize)> {
    if path.is_empty() {
        return None;
    }
    if path.len() == 1 {
        return Some((roots, path[0]));
    }
    let parent_path = &path[..path.len() - 1];
    let idx = *path.last().unwrap();
    let parent = find_node_by_path_mut(roots, parent_path)?;
    Some((&mut parent.children, idx))
}
