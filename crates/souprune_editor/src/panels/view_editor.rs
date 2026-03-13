//! View 可视化编辑器面板 — 三面板布局：Node Tree + Preview + Inspector。
mod node_props;
use super::view_preview::ViewPreviewState;
use crate::editors::SubEditorManager;
use crate::i18n::{t, t_args};
use bevy::prelude::*;
use bevy_workbench::prelude::*;
use node_props::{
    edit_data_requirements, edit_initial_facts, edit_node_basics, edit_node_repeat,
    edit_node_sprite, edit_node_state_sprite, edit_node_texts, edit_node_view_box,
    find_node_by_path, find_node_by_path_mut, parent_children_mut,
};
use souprune::core::view::layout::{ViewLayoutAsset, ViewNodeDef};
use std::path::{Path, PathBuf};

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

fn render_context_menu_items(
    ui: &mut egui::Ui,
    path: &[usize],
    action: &mut Option<TreeAction>,
    world: &World,
) {
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
            render_context_menu_items(ui, path, action, world);
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
            let Some(layout) = &mut state.layout else {
                return;
            };
            let cloned = find_node_by_path(&layout.roots, &path).cloned();
            let Some(mut node) = cloned else { return };
            node.name = format!("{}_copy", node.name);
            let parent_children = parent_children_mut(&mut layout.roots, &path);
            let Some((siblings, idx)) = parent_children else {
                return;
            };
            siblings.insert(idx + 1, node);
            state.dirty = true;
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

fn render_node_inspector(
    ui: &mut egui::Ui,
    world: &World,
    layout: &mut ViewLayoutAsset,
    path: &[usize],
) -> bool {
    let mut changed = false;
    let mut node = find_node_by_path(&layout.roots, path).unwrap().clone();

    changed |= edit_node_basics(world, ui, &mut node);
    changed |= edit_node_sprite(world, ui, &mut node.sprite);
    changed |= edit_node_state_sprite(world, ui, &mut node.state_sprite);
    changed |= edit_node_texts(world, ui, &mut node.texts);
    changed |= edit_node_view_box(world, ui, &mut node.view_box);
    changed |= edit_node_repeat(world, ui, &mut node.repeat);

    if changed {
        *find_node_by_path_mut(&mut layout.roots, path).unwrap() = node;
    }

    // Layout-level sections (requires, facts)
    changed |= edit_data_requirements(world, ui, &mut layout.requires);
    changed |= edit_initial_facts(world, ui, &mut layout.facts);
    changed
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
                changed |= render_node_inspector(ui, world, &mut layout, &path);
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
