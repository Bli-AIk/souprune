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
