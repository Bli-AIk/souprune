//! # Asset Browser Panel
//!
//! # 资产浏览器面板
//!
//! ## Module Overview
//!
//! ## 模块概述
//!
//! Asset browser panel for the editor.
//! Displays project file tree with support for file filtering, opening, creating, and cross-referencing.
//!
//! 编辑器的资产浏览器面板。
//! 显示项目文件树，支持文件过滤、打开、新建和交叉引用。

use std::collections::HashSet;
use std::path::{Path, PathBuf};

use bevy::prelude::*;
use bevy_workbench::prelude::*;

mod file_actions;
mod tree_render;

use crate::icons::EditorIcons;
use file_actions::render_new_file_dialog;
use tree_render::render_file_tree;

/// 项目文件类型。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AssetFileType {
    Sequence,
    View,
    Rule,
    Performance,
    Config,
    Other,
    Directory,
}

impl AssetFileType {
    fn icon_name(self) -> &'static str {
        match self {
            Self::Sequence => "sequence",
            Self::View => "view",
            Self::Rule => "rule",
            Self::Performance => "performance",
            Self::Config => "config",
            Self::Other => "file",
            Self::Directory => "folder",
        }
    }

    fn color(self) -> egui::Color32 {
        match self {
            Self::Sequence => egui::Color32::from_rgb(255, 200, 60),
            Self::View => egui::Color32::from_rgb(100, 220, 100),
            Self::Rule => egui::Color32::from_rgb(100, 160, 255),
            Self::Performance => egui::Color32::from_rgb(220, 100, 255),
            Self::Config => egui::Color32::from_rgb(100, 220, 220),
            Self::Other => egui::Color32::from_rgb(160, 160, 160),
            Self::Directory => egui::Color32::from_rgb(200, 200, 200),
        }
    }

    fn label(self) -> &'static str {
        match self {
            Self::Sequence => "Sequence",
            Self::View => "View",
            Self::Rule => "Rule",
            Self::Performance => "Performance",
            Self::Config => "Config",
            Self::Other => "Other",
            Self::Directory => "Directory",
        }
    }

    fn from_path(path: &Path) -> Self {
        let name = path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default();
        if path.is_dir() {
            return Self::Directory;
        }
        if name.ends_with(".sequence.ron") {
            Self::Sequence
        } else if name.ends_with(".view.ron") || name.ends_with(".view_layout.ron") {
            Self::View
        } else if name.ends_with(".fre.ron") {
            Self::Rule
        } else if name.ends_with(".performance.ron") {
            Self::Performance
        } else if name == "states.ron"
            || name == "mod.toml"
            || name.ends_with(".toml")
            || name == "config.ron"
        {
            Self::Config
        } else {
            Self::Other
        }
    }
}

/// 文件树节点。
#[derive(Debug, Clone)]
pub struct FileNode {
    pub name: String,
    pub path: PathBuf,
    pub file_type: AssetFileType,
    pub children: Vec<FileNode>,
}

impl FileNode {
    fn scan(path: &Path) -> Option<Self> {
        let name = path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default();

        // 跳过隐藏文件和 target 目录
        if name.starts_with('.') || name == "target" || name == "code" {
            return None;
        }

        if path.is_dir() {
            let mut children: Vec<FileNode> = std::fs::read_dir(path)
                .ok()?
                .filter_map(|entry| entry.ok())
                .filter_map(|entry| Self::scan(&entry.path()))
                .collect();
            children.sort_by(|a, b| {
                let dir_order = b
                    .file_type
                    .eq(&AssetFileType::Directory)
                    .cmp(&a.file_type.eq(&AssetFileType::Directory));
                dir_order.then_with(|| a.name.cmp(&b.name))
            });
            Some(FileNode {
                name,
                path: path.to_path_buf(),
                file_type: AssetFileType::Directory,
                children,
            })
        } else {
            Some(FileNode {
                name,
                path: path.to_path_buf(),
                file_type: AssetFileType::from_path(path),
                children: vec![],
            })
        }
    }
}

/// 资产浏览器面板状态。
#[derive(Resource)]
pub struct AssetBrowserState {
    /// 项目根目录的文件树。
    pub file_tree: Option<FileNode>,
    /// 过滤器：启用的文件类型。
    pub active_filters: HashSet<AssetFileType>,
    /// 展开的目录路径。
    pub expanded_dirs: HashSet<PathBuf>,
    /// 搜索关键词。
    pub search_query: String,
    /// 新建文件对话框状态。
    pub new_file_dialog: Option<NewFileDialog>,
}

impl Default for AssetBrowserState {
    fn default() -> Self {
        let mut filters = HashSet::new();
        filters.insert(AssetFileType::Sequence);
        filters.insert(AssetFileType::View);
        filters.insert(AssetFileType::Rule);
        filters.insert(AssetFileType::Performance);
        filters.insert(AssetFileType::Config);
        filters.insert(AssetFileType::Other);
        filters.insert(AssetFileType::Directory);
        Self {
            file_tree: None,
            active_filters: filters,
            expanded_dirs: HashSet::new(),
            search_query: String::new(),
            new_file_dialog: None,
        }
    }
}

/// 新建文件对话框状态。
pub struct NewFileDialog {
    pub parent_dir: PathBuf,
    pub file_name: String,
    pub file_type: AssetFileType,
}

/// 资产浏览器面板。
pub struct AssetBrowserPanel {
    cached_title: String,
}

impl AssetBrowserPanel {
    pub fn new() -> Self {
        Self {
            cached_title: "Asset Browser".to_string(),
        }
    }
}

impl WorkbenchPanel for AssetBrowserPanel {
    fn id(&self) -> &str {
        "asset_browser"
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
        use crate::i18n::t;

        self.cached_title = t(world, "panel-asset-browser");
        // 确保资源存在
        if !world.contains_resource::<AssetBrowserState>() {
            world.insert_resource(AssetBrowserState::default());
        }

        // 初始化图标纹理
        if !world.contains_resource::<EditorIcons>() {
            let icons = crate::icons::init_icons(ui.ctx());
            world.insert_resource(icons);
        }

        // 首次加载文件树
        let needs_scan = world.resource::<AssetBrowserState>().file_tree.is_none();
        if needs_scan {
            let project_root = find_project_root();
            if let Some(root) = project_root {
                let tree = FileNode::scan(&root);
                world.resource_mut::<AssetBrowserState>().file_tree = tree;
            }
        }

        render_asset_browser(ui, world);
    }

    fn ui(&mut self, ui: &mut egui::Ui) {
        ui.label("Requires World access");
    }
}

fn find_project_root() -> Option<PathBuf> {
    let config_path = PathBuf::from("projects/config.toml");
    if config_path.exists() {
        let content = std::fs::read_to_string(&config_path).ok()?;
        for line in content.lines() {
            let trimmed = line.trim();
            if !trimmed.starts_with("mod_name") {
                continue;
            }
            let Some(val) = trimmed.split('=').nth(1) else {
                continue;
            };
            let name = val.trim().trim_matches('"');
            let mod_path = PathBuf::from(format!("projects/{name}"));
            if mod_path.exists() {
                return Some(mod_path);
            }
        }
    }
    // 回退到 projects 目录
    let projects = PathBuf::from("projects");
    if projects.exists() {
        return Some(projects);
    }
    None
}

fn render_asset_browser(ui: &mut egui::Ui, world: &mut World) {
    use crate::i18n::t;

    // 渲染工具栏
    render_browser_toolbar(ui, world);
    ui.separator();

    // 渲染新建文件对话框
    render_new_file_dialog(ui, world);

    // 渲染文件树
    let state = world.resource::<AssetBrowserState>();
    let tree = state.file_tree.clone();
    let filters = state.active_filters.clone();
    let expanded = state.expanded_dirs.clone();
    let search = state.search_query.clone();

    if let Some(root) = tree {
        egui::ScrollArea::vertical().show(ui, |ui| {
            render_file_tree(ui, world, &root, &filters, &expanded, &search);
        });
    } else {
        let label_no_project = t(world, "label-no-project");
        let action_refresh = t(world, "action-refresh");
        ui.label(label_no_project);
        if ui.button(action_refresh).clicked() {
            world.resource_mut::<AssetBrowserState>().file_tree = None;
        }
    }
}

fn render_filter_button(
    ui: &mut egui::Ui,
    ft: AssetFileType,
    active: bool,
    tex_id: Option<egui::TextureId>,
    color: egui::Color32,
    world: &mut World,
) {
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 2.0;
        if let Some(id) = tex_id {
            let size = egui::vec2(16.0, 16.0);
            ui.add(egui::Image::new(egui::load::SizedTexture::new(id, size)).tint(color));
        }
        let label = egui::RichText::new(ft.label()).color(color);
        if ui.selectable_label(active, label).clicked() {
            let mut state = world.resource_mut::<AssetBrowserState>();
            if active {
                state.active_filters.remove(&ft);
            } else {
                state.active_filters.insert(ft);
            }
        }
    });
}

fn render_browser_toolbar(ui: &mut egui::Ui, world: &mut World) {
    use crate::i18n::t;

    let refresh_hover = t(world, "browser-refresh-tree");
    let search_hint = t(world, "browser-search-hint");

    ui.horizontal(|ui| {
        // 刷新按钮
        if ui.small_button("↻").on_hover_text(refresh_hover).clicked() {
            world.resource_mut::<AssetBrowserState>().file_tree = None;
        }

        // 搜索框
        let mut search = world.resource::<AssetBrowserState>().search_query.clone();
        ui.add(
            egui::TextEdit::singleline(&mut search)
                .hint_text(search_hint)
                .desired_width(120.0),
        );
        world.resource_mut::<AssetBrowserState>().search_query = search;
    });

    // 过滤器按钮
    ui.horizontal_wrapped(|ui| {
        let filter_types = [
            AssetFileType::Sequence,
            AssetFileType::View,
            AssetFileType::Rule,
            AssetFileType::Performance,
            AssetFileType::Config,
            AssetFileType::Other,
        ];

        // 预先收集图标纹理 ID 和过滤状态，避免借用冲突
        let icon_data: Vec<_> = {
            let icons = world.get_resource::<EditorIcons>();
            let state = world.resource::<AssetBrowserState>();
            filter_types
                .iter()
                .map(|ft| {
                    let active = state.active_filters.contains(ft);
                    let tex_id = icons.and_then(|i| i.get(ft.icon_name())).map(|h| h.id());
                    (*ft, active, tex_id)
                })
                .collect()
        };

        for (ft, active, tex_id) in icon_data {
            let color = if active {
                ft.color()
            } else {
                egui::Color32::from_rgb(80, 80, 80)
            };
            render_filter_button(ui, ft, active, tex_id, color, world);
        }
    });
}
