//! # 资产浏览器面板
//!
//! 显示项目文件树，支持文件过滤、打开、新建和交叉引用。

use std::collections::HashSet;
use std::path::{Path, PathBuf};

use bevy::prelude::*;
use bevy_workbench::prelude::*;

use crate::data::load_sequence_from_file;
use crate::editors::SubEditorManager;
use crate::icons::EditorIcons;
use crate::panels::sequence_timeline::EditorSequenceState;

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
            Self::Sequence => "序列",
            Self::View => "视图",
            Self::Rule => "规则",
            Self::Performance => "弹幕",
            Self::Config => "配置",
            Self::Other => "其他",
            Self::Directory => "目录",
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
pub struct AssetBrowserPanel;

impl AssetBrowserPanel {
    pub fn new() -> Self {
        Self
    }
}

impl WorkbenchPanel for AssetBrowserPanel {
    fn id(&self) -> &str {
        "asset_browser"
    }

    fn title(&self) -> String {
        "资产浏览器".to_string()
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
        ui.label("需要 World 访问权限");
    }
}

fn find_project_root() -> Option<PathBuf> {
    let config_path = PathBuf::from("projects/config.toml");
    if config_path.exists() {
        let content = std::fs::read_to_string(&config_path).ok()?;
        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("mod_name")
                && let Some(val) = trimmed.split('=').nth(1)
            {
                let name = val.trim().trim_matches('"');
                let mod_path = PathBuf::from(format!("projects/{name}"));
                if mod_path.exists() {
                    return Some(mod_path);
                }
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
        ui.label("未找到项目目录");
        if ui.button("刷新").clicked() {
            world.resource_mut::<AssetBrowserState>().file_tree = None;
        }
    }
}

fn render_browser_toolbar(ui: &mut egui::Ui, world: &mut World) {
    ui.horizontal(|ui| {
        // 刷新按钮
        if ui.small_button("↻").on_hover_text("刷新文件树").clicked() {
            world.resource_mut::<AssetBrowserState>().file_tree = None;
        }

        // 搜索框
        let mut search = world.resource::<AssetBrowserState>().search_query.clone();
        ui.add(
            egui::TextEdit::singleline(&mut search)
                .hint_text("搜索…")
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
    });
}

fn render_file_tree(
    ui: &mut egui::Ui,
    world: &mut World,
    node: &FileNode,
    filters: &HashSet<AssetFileType>,
    expanded: &HashSet<PathBuf>,
    search: &str,
) {
    // 搜索过滤
    if !search.is_empty() && !node_matches_search(node, search) {
        return;
    }

    if node.file_type == AssetFileType::Directory {
        let is_expanded = expanded.contains(&node.path);
        let icon_name = if is_expanded { "folder_open" } else { "folder" };

        let id = ui.make_persistent_id(egui::Id::new(&node.path));
        let state = egui::collapsing_header::CollapsingState::load_with_default_open(
            ui.ctx(),
            id,
            is_expanded,
        );
        let tex_id = world
            .get_resource::<EditorIcons>()
            .and_then(|i| i.get(icon_name))
            .map(|h| h.id());
        let dir_color = AssetFileType::Directory.color();
        let header_resp = state.show_header(ui, |ui| {
            ui.spacing_mut().item_spacing.x = 4.0;
            if let Some(id) = tex_id {
                let size = egui::vec2(16.0, 16.0);
                ui.add(egui::Image::new(egui::load::SizedTexture::new(id, size)).tint(dir_color));
            }
            ui.label(&node.name);
        });

        let header_open = header_resp.is_open();
        let (toggle_resp, _header_inner, _body) = header_resp.body(|ui| {
            for child in &node.children {
                render_file_tree(ui, world, child, filters, expanded, search);
            }
        });

        // 同步展开状态
        if header_open != is_expanded {
            let mut state = world.resource_mut::<AssetBrowserState>();
            if header_open {
                state.expanded_dirs.insert(node.path.clone());
            } else {
                state.expanded_dirs.remove(&node.path);
            }
        }

        // 右键菜单
        toggle_resp.context_menu(|ui| {
            render_dir_context_menu(ui, world, &node.path);
        });
    } else {
        // 文件类型过滤
        if !filters.contains(&node.file_type) {
            return;
        }

        let file_tex_id = world
            .get_resource::<EditorIcons>()
            .and_then(|i| i.get(node.file_type.icon_name()))
            .map(|h| h.id());
        let file_color = node.file_type.color();
        let resp = ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 4.0;
            if let Some(id) = file_tex_id {
                let size = egui::vec2(16.0, 16.0);
                ui.add(egui::Image::new(egui::load::SizedTexture::new(id, size)).tint(file_color));
            }
            ui.selectable_label(false, &node.name)
        });
        let resp = resp.inner;

        // 双击打开文件
        if resp.double_clicked() {
            open_asset_file(world, &node.path, node.file_type);
        }

        // 拖拽支持（桌面端）
        #[cfg(not(target_os = "android"))]
        {
            let drag_resp = resp.interact(egui::Sense::drag());
            if drag_resp.drag_started() {
                ui.ctx().set_cursor_icon(egui::CursorIcon::Grabbing);
            }
        }

        // 右键菜单
        resp.context_menu(|ui| {
            render_file_context_menu(ui, world, &node.path, node.file_type);
        });
    }
}

fn node_matches_search(node: &FileNode, search: &str) -> bool {
    let search_lower = search.to_lowercase();
    if node.name.to_lowercase().contains(&search_lower) {
        return true;
    }
    if node.file_type == AssetFileType::Directory {
        return node.children.iter().any(|c| node_matches_search(c, search));
    }
    false
}

fn render_dir_context_menu(ui: &mut egui::Ui, world: &mut World, dir_path: &Path) {
    if ui.button("新建序列").clicked() {
        start_new_file_dialog(world, dir_path, AssetFileType::Sequence);
        ui.close();
    }
    if ui.button("新建视图").clicked() {
        start_new_file_dialog(world, dir_path, AssetFileType::View);
        ui.close();
    }
    if ui.button("新建规则").clicked() {
        start_new_file_dialog(world, dir_path, AssetFileType::Rule);
        ui.close();
    }
    if ui.button("新建目录").clicked() {
        start_new_file_dialog(world, dir_path, AssetFileType::Directory);
        ui.close();
    }
}

fn render_file_context_menu(
    ui: &mut egui::Ui,
    world: &mut World,
    file_path: &Path,
    file_type: AssetFileType,
) {
    if ui.button("打开").clicked() {
        open_asset_file(world, file_path, file_type);
        ui.close();
    }
    if ui.button("查找引用").clicked() {
        find_and_show_references(ui, file_path);
        ui.close();
    }
}

fn start_new_file_dialog(world: &mut World, parent_dir: &Path, file_type: AssetFileType) {
    let default_name = match file_type {
        AssetFileType::Sequence => "untitled.sequence.ron".to_string(),
        AssetFileType::View => "untitled.view.ron".to_string(),
        AssetFileType::Rule => "untitled.fre.ron".to_string(),
        AssetFileType::Performance => "untitled.performance.ron".to_string(),
        AssetFileType::Directory => "new_directory".to_string(),
        _ => "untitled.ron".to_string(),
    };
    world.resource_mut::<AssetBrowserState>().new_file_dialog = Some(NewFileDialog {
        parent_dir: parent_dir.to_path_buf(),
        file_name: default_name,
        file_type,
    });
}

fn render_new_file_dialog(ui: &mut egui::Ui, world: &mut World) {
    let dialog = world
        .resource::<AssetBrowserState>()
        .new_file_dialog
        .as_ref()
        .map(|d| (d.parent_dir.clone(), d.file_name.clone(), d.file_type));

    let Some((parent_dir, mut file_name, file_type)) = dialog else {
        return;
    };

    egui::Window::new("新建文件")
        .collapsible(false)
        .resizable(false)
        .show(ui.ctx(), |ui| {
            ui.horizontal(|ui| {
                ui.label("名称:");
                ui.text_edit_singleline(&mut file_name);
            });
            ui.label(format!("目录: {}", parent_dir.display()));

            ui.horizontal(|ui| {
                if ui.button("创建").clicked() {
                    create_new_file(world, &parent_dir, &file_name, file_type);
                    world.resource_mut::<AssetBrowserState>().new_file_dialog = None;
                    // 刷新文件树
                    world.resource_mut::<AssetBrowserState>().file_tree = None;
                }
                if ui.button("取消").clicked() {
                    world.resource_mut::<AssetBrowserState>().new_file_dialog = None;
                }
            });
        });

    if let Some(d) = world
        .resource_mut::<AssetBrowserState>()
        .new_file_dialog
        .as_mut()
    {
        d.file_name = file_name;
    }
}

fn create_new_file(world: &mut World, parent_dir: &Path, name: &str, file_type: AssetFileType) {
    let full_path = parent_dir.join(name);

    if file_type == AssetFileType::Directory {
        let _ = std::fs::create_dir_all(&full_path);
        return;
    }

    let template = match file_type {
        AssetFileType::Sequence => "SequenceAsset(\n    chapters: [],\n)\n".to_string(),
        AssetFileType::View => "// View layout\n()".to_string(),
        AssetFileType::Rule => "// FRE rules\n(\n    rules: [],\n)\n".to_string(),
        _ => String::new(),
    };

    if let Err(e) = std::fs::write(&full_path, &template) {
        warn!("创建文件失败: {e}");
        return;
    }

    // 自动打开新文件
    open_asset_file(world, &full_path, file_type);
}

fn open_asset_file(world: &mut World, path: &Path, file_type: AssetFileType) {
    match file_type {
        AssetFileType::Sequence => match load_sequence_from_file(path) {
            Ok(seq) => {
                let mut state = world.resource_mut::<EditorSequenceState>();
                state.current = Some(seq);
                state.selected_chapter = None;
                info!("已打开序列: {}", path.display());
            }
            Err(e) => {
                warn!("打开序列失败: {e}");
            }
        },
        AssetFileType::View => {
            world
                .resource_mut::<SubEditorManager>()
                .open("view_editor", &path.display().to_string());
            info!("打开 View: {}", path.display());
        }
        AssetFileType::Rule => {
            world
                .resource_mut::<SubEditorManager>()
                .open("fre_editor", &path.display().to_string());
            info!("打开 FRE: {}", path.display());
        }
        _ => {
            world
                .resource_mut::<SubEditorManager>()
                .open("ron_source_editor", &path.display().to_string());
            info!("打开文件: {} ({})", path.display(), file_type.label());
        }
    }
}

fn find_and_show_references(ui: &mut egui::Ui, asset_path: &Path) {
    let path_str = asset_path.display().to_string();
    ui.label(format!("查找 '{path_str}' 的引用…"));
    // TODO: 扫描所有 .sequence.ron 文件查找引用
    ui.label("(交叉引用功能将在后续版本实现)");
}
