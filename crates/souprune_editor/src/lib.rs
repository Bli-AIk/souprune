//! # SoupRune Editor
//!
//! 基于 bevy_workbench 的序列驱动编辑器。
//! Sequence-driven editor built on bevy_workbench.

mod data;
mod editors;
mod i18n;
mod panels;
mod platform;
pub mod widgets;

use bevy::prelude::*;
use bevy_workbench::prelude::*;

/// SoupRune 编辑器主插件。
pub struct SoupRuneEditorPlugin;

impl Plugin for SoupRuneEditorPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(WorkbenchPlugin {
            config: WorkbenchConfig {
                show_menu_bar: true,
                show_toolbar: true,
                show_console: true,
                enable_game_view: true,
                ..default()
            },
        });

        // 平台适配
        app.add_plugins(platform::PlatformPlugin);

        // 注册编辑器面板
        app.register_panel(panels::AssetBrowserPanel::new());
        app.register_panel(panels::SequenceTimelinePanel::new());
        app.register_panel(panels::ChapterInspectorPanel::new());
        app.register_panel(panels::PlaybackPanel::new());

        // i18n 在 Startup 时注册（I18n 资源由 WorkbenchPlugin 创建）
        app.add_systems(Startup, register_i18n);

        // 自动保存系统
        app.add_systems(Update, data::auto_save_system);

        // 编辑器序列状态（必须在 Update 系统之前初始化）
        app.init_resource::<panels::sequence_timeline::EditorSequenceState>();

        // 子编辑器管理器
        app.init_resource::<editors::SubEditorManager>();
    }
}

fn register_i18n(mut i18n: ResMut<bevy_workbench::i18n::I18n>) {
    i18n::register_editor_i18n(&mut i18n);
}
