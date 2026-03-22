//! Initializes editor-only resources and supporting systems that do not belong in individual panels.
//!
//! 初始化编辑器专用资源，以及那些不应塞进某个单独面板里的支撑系统。
//!
//! This file groups the shared editor services: i18n registration, debug
//! overlays, toolbar toggles, editor state resources, and background systems
//! such as autosave. These pieces support many panels at once, so keeping them
//! here avoids scattering startup-side editor state across unrelated modules.
//!
//! 这个文件收拢的是多个面板共用的编辑器服务：国际化注册、调试覆盖层、
//! 工具栏开关、编辑器状态资源，以及自动保存这样的后台系统。它们不是某个面板
//! 的私产，因此集中在这里可以避免启动期的编辑器状态分散到无关模块里。

use bevy::prelude::*;
use bevy_workbench::prelude::*;
use souprune::editor_api as api;

use crate::{data, editors, i18n, panels};

pub(super) fn configure_i18n(app: &mut App) {
    app.add_systems(Startup, register_i18n);
}

pub(super) fn configure_debug_tools(app: &mut App) {
    api::debug::setup_collider_debug(app);
    app.init_resource::<api::debug::RuleTriggerHistory>();
    app.init_resource::<panels::fre_panel::EditorFactEventHistory>();
    app.insert_resource(bevy_workbench::game_view::GameViewToolbar {
        toggles: vec![bevy_workbench::game_view::ToolbarToggle {
            id: "gizmos".into(),
            label: "Gizmos".into(),
            enabled: false,
        }],
    });
}

pub(super) fn configure_editor_resources(app: &mut App) {
    app.init_resource::<panels::sequence_timeline::EditorSequenceState>();
    app.init_resource::<panels::playback::PlaybackState>();
    app.init_resource::<editors::SubEditorManager>();
}

pub(super) fn configure_editor_systems(app: &mut App) {
    app.add_systems(Update, data::auto_save_system);
    app.add_systems(GameSchedule, panels::playback::playback_sync_system);
}

fn register_i18n(mut i18n: ResMut<bevy_workbench::i18n::I18n>) {
    i18n::register_editor_i18n(&mut i18n);
}
