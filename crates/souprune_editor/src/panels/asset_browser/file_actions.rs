use std::path::Path;

use bevy::prelude::*;

use super::{AssetBrowserState, AssetFileType, NewFileDialog};
use crate::data::load_sequence_from_file;
use crate::editors::SubEditorManager;
use crate::panels::sequence_timeline::EditorSequenceState;

pub(super) fn start_new_file_dialog(
    world: &mut World,
    parent_dir: &Path,
    file_type: AssetFileType,
) {
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

pub(super) fn render_new_file_dialog(ui: &mut egui::Ui, world: &mut World) {
    use crate::i18n::{t, t_args};

    let dialog = world
        .resource::<AssetBrowserState>()
        .new_file_dialog
        .as_ref()
        .map(|d| (d.parent_dir.clone(), d.file_name.clone(), d.file_type));

    let Some((parent_dir, mut file_name, file_type)) = dialog else {
        return;
    };

    let window_title = t(world, "browser-new-file");
    let label_name = t(world, "browser-name");
    let dir_label = {
        let mut args = bevy_workbench::i18n::FluentArgs::new();
        args.set("path", parent_dir.display().to_string());
        t_args(world, "browser-directory", &args)
    };
    let action_create = t(world, "action-create");
    let action_cancel = t(world, "action-cancel");

    egui::Window::new(window_title)
        .collapsible(false)
        .resizable(false)
        .show(ui.ctx(), |ui| {
            ui.horizontal(|ui| {
                ui.label(label_name);
                ui.text_edit_singleline(&mut file_name);
            });
            ui.label(dir_label);

            ui.horizontal(|ui| {
                if ui.button(action_create).clicked() {
                    create_new_file(world, &parent_dir, &file_name, file_type);
                    world.resource_mut::<AssetBrowserState>().new_file_dialog = None;
                    world.resource_mut::<AssetBrowserState>().file_tree = None;
                }
                if ui.button(action_cancel).clicked() {
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

    open_asset_file(world, &full_path, file_type);
}

pub(super) fn open_asset_file(world: &mut World, path: &Path, file_type: AssetFileType) {
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

pub(super) fn find_and_show_references(ui: &mut egui::Ui, world: &World, asset_path: &Path) {
    use crate::i18n::{t, t_args};

    let path_str = asset_path.display().to_string();
    let label = {
        let mut args = bevy_workbench::i18n::FluentArgs::new();
        args.set("path", path_str);
        t_args(world, "label-find-refs-for", &args)
    };
    ui.label(label);
    ui.label(t(world, "label-crossref-todo"));
}
