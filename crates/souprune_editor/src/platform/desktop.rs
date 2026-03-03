//! 桌面平台适配 — 快捷键、文件拖放。

use std::path::Path;

use bevy::ecs::message::MessageReader;
use bevy::prelude::*;
use bevy::window::FileDragAndDrop;
use bevy_workbench::prelude::*;

use crate::data::save_sequence_to_file;
use crate::panels::sequence_timeline::EditorSequenceState;

/// 桌面平台插件 — 快捷键、拖放等桌面特有功能。
pub struct DesktopPlatformPlugin;

impl Plugin for DesktopPlatformPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (desktop_shortcuts_system, desktop_file_drop_system));
    }
}

/// 桌面快捷键系统。
fn desktop_shortcuts_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut state: ResMut<EditorSequenceState>,
    mut undo_stack: ResMut<UndoStack>,
) {
    let ctrl = keyboard.any_pressed([KeyCode::ControlLeft, KeyCode::ControlRight]);

    // Ctrl+S 保存
    if ctrl
        && keyboard.just_pressed(KeyCode::KeyS)
        && let Some(seq) = &mut state.current
        && seq.dirty
    {
        match save_sequence_to_file(seq) {
            Ok(()) => {
                seq.dirty = false;
                info!("序列已保存");
            }
            Err(e) => warn!("保存失败: {e}"),
        }
    }

    // Ctrl+Z 撤销
    if ctrl && keyboard.just_pressed(KeyCode::KeyZ) {
        undo_stack.undo_requested = true;
    }

    // Ctrl+Y 重做
    if ctrl && keyboard.just_pressed(KeyCode::KeyY) {
        undo_stack.redo_requested = true;
    }

    // Delete 删除选中章节
    if keyboard.just_pressed(KeyCode::Delete)
        && let Some(idx) = state.selected_chapter
    {
        if let Some(seq) = &mut state.current
            && idx < seq.chapters.len()
        {
            seq.chapters.remove(idx);
            seq.dirty = true;
            state.save_timer = Some(0.5);
        }
        state.selected_chapter = None;
    }

    // 方向键导航
    if keyboard.just_pressed(KeyCode::ArrowUp)
        && let Some(idx) = state.selected_chapter
        && idx > 0
    {
        state.selected_chapter = Some(idx - 1);
    }
    if keyboard.just_pressed(KeyCode::ArrowDown)
        && let Some(idx) = state.selected_chapter
    {
        let max = state
            .current
            .as_ref()
            .map(|s| s.chapters.len().saturating_sub(1))
            .unwrap_or(0);
        if idx < max {
            state.selected_chapter = Some(idx + 1);
        }
    }
}

// ─── P7.5 桌面文件拖放 ───────────────────────────────────

/// 桌面文件拖放系统 — 外部文件拖入编辑器窗口时复制到项目资源目录。
fn desktop_file_drop_system(mut dnd_events: MessageReader<FileDragAndDrop>) {
    for event in dnd_events.read() {
        if let FileDragAndDrop::DroppedFile { path_buf, .. } = event {
            let ext = path_buf.extension().and_then(|e| e.to_str()).unwrap_or("");
            match ext {
                "ron" | "toml" | "png" | "jpg" | "ogg" | "wav" | "tmx" | "tsx" => {
                    if let Some(dest) = resolve_project_asset_dir() {
                        let file_name = path_buf
                            .file_name()
                            .unwrap_or_default()
                            .to_string_lossy()
                            .to_string();
                        let target = dest.join(&file_name);
                        match std::fs::copy(path_buf, &target) {
                            Ok(_) => info!("已复制文件到项目: {}", target.display()),
                            Err(e) => warn!("复制文件失败: {e}"),
                        }
                    }
                }
                _ => {
                    info!("忽略不支持的文件类型: .{ext}");
                }
            }
        }
    }
}

/// 读取项目资源根目录。
fn resolve_project_asset_dir() -> Option<std::path::PathBuf> {
    let config_str = std::fs::read_to_string("projects/config.toml").ok()?;
    let mod_name = config_str
        .lines()
        .find(|l| l.starts_with("mod_name"))?
        .split('=')
        .nth(1)?
        .trim()
        .trim_matches('"');
    let dir = Path::new("projects").join(mod_name).join("assets");
    if dir.is_dir() {
        Some(dir)
    } else {
        std::fs::create_dir_all(&dir).ok()?;
        Some(dir)
    }
}
