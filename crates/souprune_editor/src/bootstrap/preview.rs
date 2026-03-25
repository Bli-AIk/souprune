//! Wires the embedded View preview so edit mode can render and drive UI layouts live.
//!
//! 装配嵌入式 View 预览，让编辑态能够实时渲染并驱动界面布局。
//!
//! Owns the startup glue for the preview sandbox used by the view
//! editor. It initializes preview state, registers the rebuild and render sync
//! systems, and forwards preview input into the same FRE and View update paths
//! that the real game runtime uses.
//!
//! 负责 View 编辑器预览沙盒的启动期装配。它初始化预览状态，
//! 注册重建与渲染同步系统，并把预览输入送入游戏运行时同一套 FRE 与 View
//! 更新链路里，这样编辑器看到的行为才尽量接近真实运行效果。

use bevy::prelude::*;
use bevy_workbench::prelude::EditorMode;
use souprune::editor_api as api;

use crate::{bootstrap::config, panels};

pub(super) fn configure_view_preview(app: &mut App) {
    app.init_resource::<panels::view_editor::ViewEditorState>();
    app.init_resource::<panels::view_preview::ViewPreviewState>();
    app.init_resource::<panels::view_fre_panel::ViewFreState>();
    app.add_systems(Startup, panels::view_preview::setup_view_preview);
    app.add_systems(
        Update,
        (
            panels::view_preview::sync_preview_texture,
            panels::view_preview::rebuild_preview_entities,
            ApplyDeferred,
            panels::view_preview::sync_preview_camera,
            panels::view_preview::propagate_preview_render_layers,
            api::view::update_sdf_view_shape_system,
            api::view::show_text_when_ready_system,
        )
            .chain()
            .run_if(in_state(EditorMode::Edit)),
    );
    app.add_systems(
        Update,
        panels::view_preview::preview_play_control_system.run_if(in_state(EditorMode::Edit)),
    );
    app.add_systems(
        Update,
        (
            panels::view_preview::preview_input_to_fre_system,
            api::fre_bridge::process_view_actions_system,
            api::view::evaluate_visible_when_system,
            api::view::update_fact_dependent_ui_elements,
        )
            .chain()
            .run_if(in_state(EditorMode::Edit))
            .run_if(|state: Res<panels::view_preview::ViewPreviewState>| state.playing),
    );
}

pub(super) fn insert_preview_key_map(app: &mut App) {
    config::insert_preview_key_map(app);
}
