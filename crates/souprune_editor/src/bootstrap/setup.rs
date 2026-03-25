//! Assembles the full Souprune editor application on top of a caller-provided Bevy `App`.
//!
//! 在调用方提供的 Bevy `App` 上装配完整的 Souprune 编辑器应用。
//!
//! Acts as the concrete startup recipe for the editor crate. It installs
//! the workbench shell, aligns the editor with Souprune's runtime schedules and
//! resources, wires preview support, and then layers editor-specific panels and
//! systems on top of the same game plugins used by the actual runtime.
//!
//! 编辑器 crate 的具体启动配方。它先安装 workbench 外壳，再把编辑器
//! 对齐到 Souprune 的运行时调度与资源模型上，随后接入预览支持，并在真实游戏
//! 运行时使用的同一套插件之上继续叠加编辑器专用的面板与系统。

use crate::{
    bootstrap::{config, mode, panels as editor_panels, preview, resources},
    platform,
};
use bevy::ecs::schedule::ScheduleLabel;
use bevy::prelude::*;
use bevy_workbench::prelude::*;
use souprune::editor_api as api;

pub(crate) fn build_editor_app(app: &mut App) {
    app.add_plugins(WorkbenchPlugin {
        config: WorkbenchConfig {
            show_menu_bar: true,
            show_toolbar: true,
            show_console: true,
            enable_game_view: true,
            ..default()
        },
    });

    app.add_plugins(platform::PlatformPlugin);
    app.insert_resource(souprune::GameUpdateSchedule(GameSchedule.intern()));

    let (resolution_scale, base_w, base_h) = config::load_resolution_config(app);
    app.world_mut()
        .resource_mut::<bevy_workbench::game_view::GameViewState>()
        .resolution = UVec2::new(base_w * resolution_scale, base_h * resolution_scale);

    souprune::init_game_state(app);
    app.insert_resource(api::input::TouchOverlayEnabled(false))
        .insert_resource(api::app::ResolutionScale(resolution_scale));

    souprune::insert_input_resources(app);
    preview::insert_preview_key_map(app);
    souprune::insert_font_resources(app);

    app.add_plugins(souprune::get_third_plugins());
    app.add_plugins(souprune::get_game_plugins());
    app.add_plugins(souprune::get_file_importer_plugins());

    editor_panels::register_panels(app);
    preview::configure_view_preview(app);
    resources::configure_i18n(app);
    resources::configure_debug_tools(app);
    resources::configure_editor_resources(app);
    resources::configure_editor_systems(app);
    mode::add_mode_systems(app);
}
