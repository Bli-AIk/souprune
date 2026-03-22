use std::collections::HashSet;
use std::path::{Path, PathBuf};

use bevy::prelude::*;

use super::{AssetBrowserState, AssetFileType, FileNode};
use crate::icons::EditorIcons;
use crate::panels::asset_browser::file_actions::{
    find_and_show_references, open_asset_file, start_new_file_dialog,
};

pub(super) fn render_file_tree(
    ui: &mut egui::Ui,
    world: &mut World,
    node: &FileNode,
    filters: &HashSet<AssetFileType>,
    expanded: &HashSet<PathBuf>,
    search: &str,
) {
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

        if header_open != is_expanded {
            let mut state = world.resource_mut::<AssetBrowserState>();
            if header_open {
                state.expanded_dirs.insert(node.path.clone());
            } else {
                state.expanded_dirs.remove(&node.path);
            }
        }

        toggle_resp.context_menu(|ui| {
            render_dir_context_menu(ui, world, &node.path);
        });
    } else {
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

        if resp.double_clicked() {
            open_asset_file(world, &node.path, node.file_type);
        }

        #[cfg(not(target_os = "android"))]
        {
            let drag_resp = resp.interact(egui::Sense::drag());
            if drag_resp.drag_started() {
                ui.ctx().set_cursor_icon(egui::CursorIcon::Grabbing);
            }
        }

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
    use crate::i18n::t;

    let new_seq = t(world, "browser-new-sequence");
    let new_view = t(world, "browser-new-view");
    let new_rule = t(world, "browser-new-rule");
    let new_folder = t(world, "browser-new-folder");

    if ui.button(new_seq).clicked() {
        start_new_file_dialog(world, dir_path, AssetFileType::Sequence);
        ui.close();
    }
    if ui.button(new_view).clicked() {
        start_new_file_dialog(world, dir_path, AssetFileType::View);
        ui.close();
    }
    if ui.button(new_rule).clicked() {
        start_new_file_dialog(world, dir_path, AssetFileType::Rule);
        ui.close();
    }
    if ui.button(new_folder).clicked() {
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
    use crate::i18n::t;

    let action_open = t(world, "action-open");
    let action_find_refs = t(world, "action-find-refs");

    if ui.button(action_open).clicked() {
        open_asset_file(world, file_path, file_type);
        ui.close();
    }
    if ui.button(action_find_refs).clicked() {
        find_and_show_references(ui, world, file_path);
        ui.close();
    }
}
